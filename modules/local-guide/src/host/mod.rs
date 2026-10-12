//! Host dashboard surface — the settings page (spec Bons plans et activités §2), mode « page ».
//!
//! Three `Section`s, the shell's cards, in the board's order: your picks, the partners' activities,
//! the public page. Your picks are two lists behind a segmented switch (`Tabs`): the addresses and
//! your activities, each a sortable `DataTable` (order = carousel order) whose row opens in a side
//! panel (`panel: "sheet"`), its fields named `<list>.<i>.<key>` at the rank the row is drawn. A
//! block that depends on a choice hides with `visibleWhen`. A rule broken shows under its field,
//! the same one `publishReadiness` reports; no Save button — the shell autosaves through
//! `updateConfig`.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{TabItem, Tone};
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, ChoiceList, DataTable, Field, FieldHint, Form, ImageUpload, InlineNotice,
    NumberInput, Page, PhoneInput, Section, Select, Stack, Tabs, TextArea, TextInput, Toggle,
    ToggleRow, WeeklyHours,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::affiliate::MAX_CURATED_LINKS;
use crate::config::{
    ActivityRow, HostActivityRow, ModuleConfig, SpotRow, CATEGORIES, MAX_GALLERY,
    MAX_HOST_ACTIVITIES, PARTNER_MAX_PRICE, PARTNER_RADIUS_KM, PRICE_LEVELS, PROVIDERS, PUBLIC_MAX,
    PUBLIC_MIN, SPOT_MAX_KM,
};
use crate::tiqets::TiqetsStatus;
use crate::viator::ViatorStatus;

pub use crate::config::MAX_SPOTS;

/// Les lignes ajoutées pas encore enregistrées, par liste : leurs id, séparés par des virgules.
const NEW_SPOTS: &str = "newSpots";
const NEW_ACTIVITIES: &str = "newActivities";
const NEW_LINKS: &str = "newLinks";

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::GuideEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::MapPin
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(picks_section(&config, &ctx))
                .child(partners_section(&config, &ctx))
                .child(public_section(&config, &ctx)),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 Vos bons plans ───────────────────────────────────────────────────────

/// « Adresses N / 30 » | « Vos activités N / 6 » : deux listes, un tableau chacune.
fn picks_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let spots = rows(
        ctx,
        NEW_SPOTS,
        &config.spots,
        SpotRow::is_blank,
        |i, spot| fallback_id(&spot.id, || format!("spot-{}", i + 1)),
    );
    let activities = rows(
        ctx,
        NEW_ACTIVITIES,
        &config.host_activities,
        HostActivityRow::is_blank,
        |i, activity| activity.route_id(i),
    );
    let tab = |id: &str, key: &str, count: usize, max: usize, content: Component| TabItem {
        id: Some(id.into()),
        label: Some(t!(key, count = count, max = max).unwrap_or_else(|_| format!("i18n:{key}"))),
        content: Some(Box::new(content)),
    };
    Section::new()
        .title("i18n:host.picks.title")
        .subtitle("i18n:host.picks.help")
        .child(Tabs::new().id("picks").tabs(vec![
            tab(
                "spots",
                "host.picks.tab.spots",
                spots.len(),
                MAX_SPOTS,
                spots_table(config, ctx, &spots),
            ),
            tab(
                "activities",
                "host.picks.tab.activities",
                activities.len(),
                MAX_HOST_ACTIVITIES,
                activities_table(config, ctx, &activities),
            ),
        ]))
        .into()
}

/// Les lignes d'une liste : celles enregistrées, puis celles qu'« Ajouter » a ouvertes. Une ligne
/// sans id (enregistrée avant le tableau) en reçoit un, stable d'un rendu à l'autre : la plateforme
/// la rapproche par sa position au premier enregistrement, et l'id reste. Les emplacements vides
/// de l'ancien formulaire ne sont pas des lignes.
fn rows<T: Clone + Default>(
    ctx: &HostContext,
    added: &str,
    stored: &[T],
    blank: impl Fn(&T) -> bool,
    id_of: impl Fn(usize, &T) -> String,
) -> Vec<(String, T)> {
    let mut rows: Vec<(String, T)> = stored
        .iter()
        .enumerate()
        .filter(|(_, row)| !blank(row))
        .map(|(index, row)| (id_of(index, row), row.clone()))
        .collect();
    for id in ctx.input_str(added).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty() && !rows.iter().any(|(known, _)| known == id) {
            rows.push((id.to_string(), T::default()));
        }
    }
    rows
}

fn fallback_id(id: &str, fallback: impl FnOnce() -> String) -> String {
    match id.trim() {
        "" => fallback(),
        id => id.to_string(),
    }
}

/// « Ajouter » : une ligne de plus, avec un id neuf, que le tableau de bord ouvre aussitôt.
fn add_row<T>(
    table: DataTable,
    rows: &[(String, T)],
    blank: impl Fn(&T) -> bool,
    key: &str,
    label: &str,
) -> DataTable {
    let mut added: Vec<String> = rows
        .iter()
        .filter(|(_, row)| blank(row))
        .map(|(id, _)| id.clone())
        .collect();
    added.push(Uuid::new_v4().to_string());
    let mut input = Map::new();
    input.insert(key.into(), Value::String(added.join(",")));
    table.addLabel(label).addAction(Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(Value::Object(input)),
    ))
}

/// Le message de « N au maximum », au-dessus du tableau.
fn list_error(config: &ModuleConfig, ctx: &HostContext, field: &str) -> Option<Component> {
    config.error_of(field).map(|error| {
        InlineNotice::new()
            .tone(Tone::Danger)
            .message(error.get(&ctx.locale).to_string())
            .into()
    })
}

fn spots_table(config: &ModuleConfig, ctx: &HostContext, rows: &[(String, SpotRow)]) -> Component {
    let today = portaki_sdk::host::time::now()
        .ok()
        .map(|now| now.date_naive());
    let home = home(ctx);
    let mut table = DataTable::new()
        .rowName("spots")
        .sortable(true)
        .panel("sheet")
        .columns(json!([
            { "key": "title", "label": "i18n:host.spot.name", "sub": "sub" },
            { "key": "category", "label": "i18n:host.spot.category" },
            { "key": "distance", "label": "i18n:host.spot.distance", "mono": true },
            { "key": "badge", "label": "", "kind": "pill" },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(id, spot)| {
                    let off_season = today.is_some_and(|day| !spot.in_season(day));
                    let badge = if off_season {
                        "i18n:host.spot.offSeason"
                    } else if spot.perk.is_blank() {
                        ""
                    } else {
                        "i18n:host.spot.badge.perk"
                    };
                    let perk = spot.perk.host_value(ctx);
                    let sub = if perk.trim().is_empty() {
                        spot.note
                            .as_ref()
                            .map(|note| short(note.host_value(ctx), 60))
                            .unwrap_or_default()
                    } else {
                        perk.to_string()
                    };
                    json!({
                        "id": id,
                        "title": spot.title.host_value(ctx),
                        "sub": sub,
                        "category": spot.category.as_deref().map(category_label).unwrap_or_default(),
                        "distance": distance_label(spot, home, &ctx.locale).unwrap_or_default(),
                        "badge": badge,
                    })
                })
                .collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (_, spot))| spot_panel(index, spot, ctx, home, today))
                .collect(),
        );
    if rows.len() < MAX_SPOTS {
        table = add_row(
            table,
            rows,
            SpotRow::is_blank,
            NEW_SPOTS,
            "i18n:host.spots.add",
        );
    }
    Stack::new()
        .gap(12.0)
        .children(
            list_error(config, ctx, "spots")
                .into_iter()
                .chain([table.into()])
                .collect(),
        )
        .into()
}

fn activities_table(
    config: &ModuleConfig,
    ctx: &HostContext,
    rows: &[(String, HostActivityRow)],
) -> Component {
    let mut table = DataTable::new()
        .rowName("host_activities")
        .sortable(true)
        .panel("sheet")
        .columns(json!([
            { "key": "title", "label": "i18n:host.hostActivities.name", "sub": "sub" },
            { "key": "duration", "label": "i18n:host.hostActivities.duration", "mono": true },
            { "key": "book", "label": "i18n:host.hostActivities.book" },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(id, activity)| {
                    let sub: Vec<&str> = [activity.provider.as_deref(), activity.price.as_deref()]
                        .into_iter()
                        .flatten()
                        .collect();
                    // Comment on réserve, tel que le livret le fera : le téléphone fait le bouton
                    // « Réserver », le site un lien ; sans eux, on passe par l'hôte.
                    let book = if activity.phone.is_some() {
                        "i18n:host.hostActivities.book.phone"
                    } else if activity.url.is_some() {
                        "i18n:host.hostActivities.book.link"
                    } else {
                        "i18n:host.hostActivities.book.host"
                    };
                    json!({
                        "id": id,
                        "title": activity.title.host_value(ctx),
                        "sub": sub.join(" · "),
                        "duration": activity.duration.as_deref().unwrap_or_default(),
                        "book": book,
                    })
                })
                .collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (_, activity))| activity_panel(index, activity, ctx))
                .collect(),
        );
    if rows.len() < MAX_HOST_ACTIVITIES {
        table = add_row(
            table,
            rows,
            HostActivityRow::is_blank,
            NEW_ACTIVITIES,
            "i18n:host.hostActivities.add",
        );
    }
    Stack::new()
        .gap(12.0)
        .children(
            list_error(config, ctx, "host_activities")
                .into_iter()
                .chain([FieldHint::new()
                    .text("i18n:host.hostActivities.subtitle")
                    .into()])
                .chain([table.into()])
                .collect(),
        )
        .into()
}

/// Un champ de ligne, avec l'erreur de sa sous-clé sous lui s'il y en a une.
fn row_field(
    list: &str,
    index: usize,
    key: &str,
    problems: &[(&'static str, I18nText)],
    ctx: &HostContext,
) -> Field {
    let field = Field::new().name(format!("{list}.{index}.{key}"));
    match problems.iter().find(|(name, _)| *name == key) {
        Some((_, error)) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// Le panneau d'une adresse (§2.1), dans l'ordre de la planche.
fn spot_panel(
    index: usize,
    spot: &SpotRow,
    ctx: &HostContext,
    home: Option<(f64, f64)>,
    today: Option<chrono::NaiveDate>,
) -> Component {
    let name = |key: &str| format!("spots.{index}.{key}");
    let problems = spot.problems();
    let field = |key: &str| row_field("spots", index, key, &problems, ctx);
    let text = |value: &Option<String>| value.as_deref().unwrap_or_default().to_string();

    // La catégorie et le prix en liste fermée ; une ancienne saisie libre reste choisie et
    // affichée telle quelle, plutôt que de disparaître au premier enregistrement.
    let category = text(&spot.category);
    let mut categories: Vec<ChoiceOption> = CATEGORIES
        .iter()
        .map(|(token, value)| ChoiceOption::new(*value, format!("i18n:host.category.{token}")))
        .collect();
    if !category.is_empty() && !CATEGORIES.iter().any(|(_, value)| *value == category) {
        categories.push(ChoiceOption::new(category.clone(), category.clone()));
    }
    let price = text(&spot.price);
    let mut prices = vec![ChoiceOption::new("", "i18n:host.spot.price.none")];
    prices.extend(
        PRICE_LEVELS
            .iter()
            .map(|level| ChoiceOption::new(*level, *level)),
    );
    if !price.is_empty() && !PRICE_LEVELS.contains(&price.as_str()) {
        prices.push(ChoiceOption::new(price.clone(), price.clone()));
    }

    // La position : sans elle, ni repère ni distance ; trop loin, sans doute mal placée.
    let mut picker = AddressMapPicker::new()
        .addressName(name("address"))
        .latName(name("lat"))
        .lngName(name("lng"))
        .address(text(&spot.address))
        .label("i18n:host.spot.address");
    if let Some(line) = distance_label(spot, home, &ctx.locale) {
        picker = picker.hint(line);
    }
    if let Some((lat, lng)) = spot.coords() {
        picker = picker.lat(lat).lng(lng);
    }
    let mut position = Field::new().name(name("lat")).child(picker);
    if let Some((lat, lng)) = spot.coords() {
        let far = home.is_some_and(|(home_lat, home_lng)| {
            !portaki_sdk::config::check::within_km(lat, lng, home_lat, home_lng, SPOT_MAX_KM)
        });
        if far {
            position = position.child(
                InlineNotice::new()
                    .tone(Tone::Warning)
                    .message("i18n:host.spot.position.far"),
            );
        }
    } else if !spot.is_blank() {
        position = position.error(
            crate::i18n::text("host.spot.position.required")
                .get(&ctx.locale)
                .to_string(),
        );
    }

    let mut perk = field("perk")
        .label("i18n:host.spot.perk")
        .child(
            TextInput::new()
                .name(name("perk"))
                .value(spot.perk.host_value(ctx))
                .placeholder("i18n:host.spot.perk.placeholder"),
        )
        .child(
            Toggle::new()
                .name(name("perk_agreement"))
                .label("i18n:host.spot.perkAgreement")
                .checked(spot.perk_agreement),
        );
    if spot.perk_without_agreement() {
        perk = perk.child(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message("i18n:host.spot.perkAgreement.warning"),
        );
    }

    // Les anciens horaires en texte libre restent affichés au voyageur tant que l'hôte n'a pas
    // saisi les nouveaux : on les lui rappelle sous le champ.
    let mut hours = Field::new()
        .name(name("weekly_hours"))
        .label("i18n:host.spot.hours")
        .child(
            WeeklyHours::new()
                .name(name("weekly_hours"))
                .value(spot.weekly_hours.trim()),
        );
    let legacy: Vec<&str> = [spot.hours.as_deref(), spot.opening.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !spot.has_weekly_hours() && !legacy.is_empty() {
        if let Ok(line) = t!("host.spot.hours.legacy", hours = legacy.join(" · ")) {
            hours = hours.child(FieldHint::new().text(line));
        }
    }

    let off_season = today.is_some_and(|day| !spot.in_season(day));
    let mut children: Vec<Component> = vec![
        field("title")
            .label("i18n:host.spot.name")
            .required(true)
            .child(
                TextInput::new()
                    .name(name("title"))
                    .value(spot.title.host_value(ctx))
                    .placeholder("i18n:host.spot.name.placeholder"),
            )
            .into(),
        field("category")
            .label("i18n:host.spot.category")
            .required(true)
            .child(
                Select::new()
                    .name(name("category"))
                    .options(categories)
                    .value(category),
            )
            .into(),
        field("price")
            .label("i18n:host.spot.price")
            .child(
                ChoiceList::new()
                    .name(name("price"))
                    .layout(ChoiceListLayout::Compact)
                    .choices(prices)
                    .value(price),
            )
            .into(),
        position.into(),
    ];
    children.extend(photo_fields(index, spot));
    children.extend([
        perk.into(),
        field("note")
            .label("i18n:host.spot.note")
            .child(
                TextArea::new()
                    .name(name("note"))
                    .value(
                        spot.note
                            .as_ref()
                            .map(|note| note.host_value(ctx))
                            .unwrap_or_default(),
                    )
                    .rows(3)
                    .placeholder("i18n:host.spot.note.placeholder"),
            )
            .child(FieldHint::new().text("i18n:host.spot.note.hint"))
            .into(),
        hours.into(),
        field("season_from")
            .label("i18n:host.spot.seasonFrom")
            .child(
                TextInput::new()
                    .name(name("season_from"))
                    .value(text(&spot.season_from))
                    .placeholder("01/04"),
            )
            .into(),
        Field::new()
            .name(name("season_to"))
            .label("i18n:host.spot.seasonTo")
            .child(
                TextInput::new()
                    .name(name("season_to"))
                    .value(text(&spot.season_to))
                    .placeholder("30/09"),
            )
            .child(FieldHint::new().text(if off_season {
                "i18n:host.spot.offSeason"
            } else {
                "i18n:host.spot.season.hint"
            }))
            .into(),
        field("parking")
            .label("i18n:host.spot.parking")
            .child(
                TextInput::new()
                    .name(name("parking"))
                    .value(text(&spot.parking))
                    .placeholder("i18n:host.spot.parking.placeholder"),
            )
            .into(),
        field("phone")
            .label("i18n:host.spot.phone")
            .child(
                PhoneInput::new()
                    .name(name("phone"))
                    .value(text(&spot.phone))
                    .defaultCountry(default_country(ctx)),
            )
            .into(),
        field("url")
            .label("i18n:host.spot.url")
            .child(
                TextInput::new()
                    .name(name("url"))
                    .value(text(&spot.url))
                    .placeholder("https://"),
            )
            .into(),
        field("warning")
            .label("i18n:host.spot.warning")
            .child(
                TextInput::new()
                    .name(name("warning"))
                    .value(spot.warning.host_value(ctx))
                    .placeholder("i18n:host.spot.warning.placeholder"),
            )
            .into(),
    ]);
    Stack::new().gap(14.0).children(children).into()
}

/// Les dépôts de photos d'une adresse : celles déjà là, plus une case libre (§2.12).
///
/// Une case de plus et non cinq : une rangée de cinq champs vides par adresse serait un formulaire
/// qu'on ne lit plus.
fn photo_fields(index: usize, spot: &SpotRow) -> Vec<Component> {
    let stored: Vec<String> = spot.photo_refs().into_iter().map(str::to_string).collect();
    let slots = (stored.len() + 1).clamp(1, MAX_GALLERY).max(stored.len());
    (0..slots)
        .map(|slot| {
            let label = if slot == 0 {
                "i18n:host.spot.photo".to_string()
            } else {
                t!("host.spot.photo.more", rank = &(slot + 1).to_string())
                    .unwrap_or_else(|_| "i18n:host.spot.photo".into())
            };
            Field::new()
                .name(format!("spots.{index}.photos.{slot}"))
                .label(label)
                .child(
                    ImageUpload::new()
                        .name(format!("spots.{index}.photos.{slot}"))
                        .value(stored.get(slot).cloned().unwrap_or_default()),
                )
                .into()
        })
        .collect()
}

/// Le panneau d'une activité : le modèle actuel, du nom à la photo.
fn activity_panel(index: usize, activity: &HostActivityRow, ctx: &HostContext) -> Component {
    let name = |key: &str| format!("host_activities.{index}.{key}");
    let problems = activity.problems();
    let field = |key: &str| row_field("host_activities", index, key, &problems, ctx);
    let text = |value: &Option<String>| value.as_deref().unwrap_or_default().to_string();
    let input = |key: &str, label: &str, value: String| -> Component {
        field(key)
            .label(label)
            .child(TextInput::new().name(name(key)).value(value))
            .into()
    };
    let area = |key: &str, label: &str, value: &str, placeholder: &str| -> Component {
        field(key)
            .label(label)
            .child(
                TextArea::new()
                    .name(name(key))
                    .value(value)
                    .rows(2)
                    .placeholder(placeholder),
            )
            .into()
    };
    Stack::new()
        .gap(14.0)
        .children(vec![
            field("title")
                .label("i18n:host.hostActivities.name")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("title"))
                        .value(activity.title.host_value(ctx)),
                )
                .into(),
            field("provider")
                .label("i18n:host.hostActivities.provider")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("provider"))
                        .value(text(&activity.provider)),
                )
                .child(FieldHint::new().text("i18n:host.hostActivities.provider.hint"))
                .into(),
            input(
                "duration",
                "i18n:host.hostActivities.duration",
                text(&activity.duration),
            ),
            input(
                "price",
                "i18n:host.hostActivities.price",
                text(&activity.price),
            ),
            field("phone")
                .label("i18n:host.hostActivities.phone")
                .child(
                    PhoneInput::new()
                        .name(name("phone"))
                        .value(text(&activity.phone))
                        .defaultCountry(default_country(ctx)),
                )
                .child(FieldHint::new().text("i18n:host.hostActivities.phone.hint"))
                .into(),
            field("url")
                .label("i18n:host.hostActivities.url")
                .child(
                    TextInput::new()
                        .name(name("url"))
                        .value(text(&activity.url))
                        .placeholder("https://"),
                )
                .into(),
            input(
                "meet",
                "i18n:host.hostActivities.meet",
                text(&activity.meet),
            ),
            input(
                "languages",
                "i18n:host.hostActivities.languages",
                text(&activity.languages),
            ),
            area(
                "cancel",
                "i18n:host.hostActivities.cancel",
                activity.cancel.host_value(ctx),
                "i18n:host.hostActivities.cancel.placeholder",
            ),
            area(
                "included",
                "i18n:host.hostActivities.included",
                activity.included.host_value(ctx),
                "i18n:host.hostActivities.included.placeholder",
            ),
            FieldHint::new()
                .text("i18n:host.hostActivities.included.hint")
                .into(),
            area(
                "tip",
                "i18n:host.hostActivities.tip",
                activity.tip.host_value(ctx),
                "i18n:host.hostActivities.tip.placeholder",
            ),
            Field::new()
                .name(name("photo"))
                .label("i18n:host.hostActivities.photo")
                .child(
                    ImageUpload::new()
                        .name(name("photo"))
                        .value(activity.photo.clone()),
                )
                .into(),
        ])
        .into()
}

// ── 2.2 Activités des partenaires ────────────────────────────────────────────

/// « Afficher le catalogue », puis, allumé, les fournisseurs, les filtres et « Vos choix ».
fn partners_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let providers = config.providers();
    let choices: Vec<ChoiceOption> = PROVIDERS
        .iter()
        .map(|provider| {
            ChoiceOption::new(*provider, format!("i18n:host.partners.provider.{provider}"))
        })
        .collect();
    let (radius_min, radius_max) = PARTNER_RADIUS_KM;
    let (price_min, price_max) = PARTNER_MAX_PRICE;
    let mut max_price = NumberInput::new()
        .name("partner_max_price")
        .min(f64::from(price_min))
        .max(f64::from(price_max));
    if let Some(price) = config.partner_max_price {
        max_price = max_price.value(f64::from(price));
    }
    let rating = config
        .partner_min_rating
        .unwrap_or(config.viator().min_rating)
        .to_string();

    let mut catalogue = Section::new().visibleWhen("partners_enabled=true").child(
        named(config, ctx, "providers")
            .label("i18n:host.partners.providers")
            .required(true)
            .child(
                ChoiceList::new()
                    .name("providers")
                    .multi(true)
                    .layout(ChoiceListLayout::Compact)
                    .choices(choices)
                    .value(serde_json::to_string(&providers).unwrap_or_default()),
            )
            // GetYourGuide n'a pas encore de connecteur : il est annoncé, pas proposé.
            .child(FieldHint::new().text("i18n:host.partners.provider.soon")),
    );
    for notice in status_notices(config, ctx, &providers) {
        catalogue = catalogue.child(notice);
    }
    catalogue = catalogue
        .child(
            named(config, ctx, "partner_radius_km")
                .label("i18n:host.partners.radius")
                .child(
                    NumberInput::new()
                        .name("partner_radius_km")
                        .min(f64::from(radius_min))
                        .max(f64::from(radius_max))
                        .value(f64::from(config.partner_radius())),
                )
                .child(FieldHint::new().text("i18n:host.partners.radius.hint")),
        )
        .child(
            named(config, ctx, "partner_max_price")
                .label("i18n:host.partners.maxPrice")
                .child(max_price),
        )
        .child(
            Field::new()
                .name("partner_min_rating")
                .label("i18n:host.partners.minRating")
                .child(
                    Select::new()
                        .name("partner_min_rating")
                        .options(
                            ["0", "3", "4"]
                                .iter()
                                .map(|value| {
                                    ChoiceOption::new(
                                        *value,
                                        format!("i18n:host.partners.minRating.{value}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(rating),
                ),
        )
        .child(choices_section(config, ctx));

    Section::new()
        .title("i18n:host.partners.title")
        .child(
            ToggleRow::new()
                .name("partners_enabled")
                .label("i18n:host.partners.enabled")
                .description("i18n:host.partners.enabled.help")
                .checked(config.partners_on()),
        )
        .child(catalogue)
        .into()
}

/// Pourquoi un fournisseur choisi ne montrera rien : pas de clé, logement sans position ou sans
/// ville. Rien pour un fournisseur prêt.
fn status_notices(config: &ModuleConfig, ctx: &HostContext, providers: &[&str]) -> Vec<Component> {
    let mut keys: Vec<&str> = Vec::new();
    if providers.contains(&"viator") {
        let viator = crate::config::ViatorConfig {
            enabled: true,
            ..config.viator()
        };
        keys.extend(match crate::viator::status(ctx, &viator) {
            ViatorStatus::MissingKey => Some("i18n:host.viator.status.missingKey"),
            ViatorStatus::MissingCity => Some("i18n:host.viator.status.missingCity"),
            ViatorStatus::Off | ViatorStatus::Ready => None,
        });
    }
    if providers.contains(&"tiqets") {
        let tiqets = crate::config::TiqetsConfig {
            enabled: true,
            ..config.tiqets()
        };
        keys.extend(match crate::tiqets::status(ctx, &tiqets) {
            TiqetsStatus::MissingKey => Some("i18n:host.tiqets.status.missingKey"),
            TiqetsStatus::MissingCoordinates => Some("i18n:host.tiqets.status.missingCoordinates"),
            TiqetsStatus::Off | TiqetsStatus::Ready => None,
        });
    }
    keys.into_iter()
        .map(|key| InlineNotice::new().tone(Tone::Warning).message(key).into())
        .collect()
}

/// « Vos choix dans le catalogue » : les liens collés par l'hôte, cinq au plus, chacun avec son
/// conseil, ouverts dans une fenêtre (`panel: "dialog"`).
fn choices_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows = rows(
        ctx,
        NEW_LINKS,
        &config.activities,
        ActivityRow::is_blank,
        |i, link| fallback_id(&link.id, || format!("link-{}", i + 1)),
    );
    let mut table = DataTable::new()
        .rowName("activities")
        .sortable(true)
        .panel("dialog")
        .columns(json!([
            { "key": "label", "label": "i18n:host.activities.link.label", "sub": "sub" },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(id, link)| {
                    let label = link.label.host_value(ctx);
                    let tip = link.tip.host_value(ctx);
                    let mut sub = provider_name(&link.url).to_string();
                    if !tip.trim().is_empty() {
                        sub = format!("{sub} · « {} »", short(tip, 60));
                    }
                    json!({
                        "id": id,
                        "label": if label.trim().is_empty() { link.url.trim() } else { label },
                        "sub": sub,
                    })
                })
                .collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (_, link))| link_panel(index, link, ctx))
                .collect(),
        );
    if rows.len() < MAX_CURATED_LINKS {
        table = add_row(
            table,
            &rows,
            ActivityRow::is_blank,
            NEW_LINKS,
            "i18n:host.choices.add",
        );
    }
    let mut section = Section::new()
        .title("i18n:host.choices.title")
        .subtitle(
            t!(
                "host.choices.count",
                count = rows.len(),
                max = MAX_CURATED_LINKS
            )
            .unwrap_or_else(|_| "i18n:host.choices.count".into()),
        )
        .child(FieldHint::new().text("i18n:host.choices.help"));
    if let Some(error) = list_error(config, ctx, "activities") {
        section = section.child(error);
    }
    section.child(table).into()
}

/// La fenêtre d'un lien : le lien, son intitulé, le conseil de l'hôte (≤ 200).
fn link_panel(index: usize, link: &ActivityRow, ctx: &HostContext) -> Component {
    let name = |key: &str| format!("activities.{index}.{key}");
    let problems = link.problems();
    let field = |key: &str| row_field("activities", index, key, &problems, ctx);
    Stack::new()
        .gap(14.0)
        .children(vec![
            field("url")
                .label("i18n:host.activities.link.url")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("url"))
                        .value(link.url.trim())
                        .placeholder("i18n:host.activities.link.url.placeholder"),
                )
                .into(),
            field("label")
                .label("i18n:host.activities.link.label")
                .child(
                    TextInput::new()
                        .name(name("label"))
                        .value(link.label.host_value(ctx))
                        .placeholder("i18n:host.activities.link.label.placeholder"),
                )
                .into(),
            field("tip")
                .label("i18n:host.activities.link.tip")
                .child(
                    TextArea::new()
                        .name(name("tip"))
                        .value(link.tip.host_value(ctx))
                        .rows(3)
                        .placeholder("i18n:host.activities.link.tip.placeholder"),
                )
                .into(),
        ])
        .into()
}

/// Le fournisseur d'un lien collé, par son domaine : des noms de marque, non traduits.
fn provider_name(url: &str) -> &'static str {
    let url = url.to_ascii_lowercase();
    if url.contains("viator.com") {
        "Viator"
    } else if url.contains("tiqets.com") {
        "Tiqets"
    } else if url.contains("getyourguide.com") || url.contains("gyg.me") {
        "GetYourGuide"
    } else {
        ""
    }
}

// ── 2.3 Sur votre page publique ──────────────────────────────────────────────

/// L'interrupteur du bloc (la page publique est sans séjour : rien n'y sort sans l'avoir décidé),
/// puis le choix de 3 à 6 adresses, en pastilles.
fn public_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let spots = config.parse_spots();
    let choices: Vec<ChoiceOption> = spots
        .iter()
        .map(|spot| ChoiceOption::new(spot.id.clone(), spot.title.host_value(ctx).to_string()))
        .collect();
    let hint = if spots.is_empty() {
        "i18n:host.public.none".to_string()
    } else {
        t!(
            "host.public.count",
            count = config.public_chosen().len(),
            min = PUBLIC_MIN,
            max = PUBLIC_MAX
        )
        .unwrap_or_else(|_| "i18n:host.public.count".into())
    };
    Section::new()
        .title("i18n:host.public.title")
        .subtitle("i18n:host.public.subtitle")
        .child(
            ToggleRow::new()
                .name("public_enabled")
                .label("i18n:host.public.enabled")
                .checked(config.public_enabled),
        )
        .child(
            Field::new()
                .name("public_spots")
                .label("i18n:host.public.spots")
                .visibleWhen("public_enabled=true")
                .child(
                    ChoiceList::new()
                        .name("public_spots")
                        .multi(true)
                        .layout(ChoiceListLayout::Compact)
                        .limit(PUBLIC_MAX as u32)
                        .choices(choices)
                        .value(serde_json::to_string(&config.public_spots).unwrap_or_default()),
                )
                .child(FieldHint::new().text(hint)),
        )
        .into()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    let field = Field::new().name(name);
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// La position du logement, `None` tant qu'il n'est pas géocodé.
fn home(ctx: &HostContext) -> Option<(f64, f64)> {
    let point = ctx.property.coordinates.as_ref()?;
    crate::config::valid_coords(point.lat, point.lng)
}

/// « 800 m », « 3,2 km » : calculée depuis le logement, jamais saisie (§4).
fn distance_label(spot: &SpotRow, home: Option<(f64, f64)>, locale: &str) -> Option<String> {
    let (home_lat, home_lng) = home?;
    let (lat, lng) = spot.coords()?;
    let metres = crate::guest::item::haversine_m(home_lat, home_lng, lat, lng);
    Some(if metres < 1_000.0 {
        format!("{} m", ((metres / 10.0).round() as i64) * 10)
    } else {
        let km = format!("{:.1} km", metres / 1_000.0);
        // La virgule décimale, sauf en anglais, chinois et japonais.
        match locale.split(['-', '_']).next().unwrap_or_default() {
            "en" | "zh" | "ja" => km,
            _ => km.replace('.', ","),
        }
    })
}

/// Le libellé traduit d'une catégorie de la liste ; une ancienne saisie libre, telle quelle.
fn category_label(category: &str) -> String {
    CATEGORIES
        .iter()
        .find(|(_, value)| *value == category.trim())
        .map(|(token, _)| format!("i18n:host.category.{token}"))
        .unwrap_or_else(|| category.trim().to_string())
}

/// Les `max` premiers caractères, une ellipse au-delà.
fn short(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max).collect::<String>().trim_end())
    }
}

/// L'indicatif proposé d'abord : la région de la langue du logement (`fr-FR` → `FR`), la France
/// sinon.
fn default_country(ctx: &HostContext) -> String {
    ctx.property
        .locale
        .split(['-', '_'])
        .nth(1)
        .filter(|region| region.len() == 2 && region.chars().all(|c| c.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| "FR".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_text_is_cut_on_a_character() {
        assert_eq!(
            short("  Prenez la bouillabaisse  ", 60),
            "Prenez la bouillabaisse"
        );
        assert_eq!(short("Crème brûlée", 5), "Crème…");
    }

    #[test]
    fn a_category_from_the_list_is_translated_a_free_one_kept() {
        assert_eq!(category_label("Bien-être"), "i18n:host.category.wellness");
        assert_eq!(category_label(" Coup de cœur "), "Coup de cœur");
    }

    #[test]
    fn a_link_names_its_provider() {
        assert_eq!(
            provider_name("https://www.viator.com/tours/x/d1-2P3"),
            "Viator"
        );
        assert_eq!(provider_name("https://www.tiqets.com/fr/x-p1/"), "Tiqets");
        assert_eq!(provider_name("https://gyg.me/aBcD12"), "GetYourGuide");
        assert_eq!(provider_name("https://example.com"), "");
    }
}
