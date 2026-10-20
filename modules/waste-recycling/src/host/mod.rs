//! Host dashboard surface — the settings page (spec Tri §2).
//!
//! Five `Section`s, the shell's cards, in the spec's order: collection, bins, bin room, drop-off
//! points, compost. The page chrome (title, switch, Publier) belongs to the dashboard. A rule
//! broken shows under its field (`Field::error`), the same one `publishReadiness` blocks on.
//!
//! A block behind a switch hides with `visibleWhen` (common rules §2.2: hidden, not greyed). The
//! form keys are the stored ones (`title`, `items`, `color`, `bin_room_*`, `compost_*`): the
//! platform merges a save key by key, and a renamed key would sit next to the old one forever.
//! A field the spec dropped stays on the page only while it still holds something, so the host
//! can read and clear it.

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Checkbox, ChoiceList, Field, FieldHint, Form, InlineNotice, Map, Page,
    RadioGroup, SecretInput, Section, Stack, StepList, TextArea, TextInput, TimeRange, Toggle,
    ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

pub use crate::config::MAX_BINS;
use crate::config::{BinRow, DropoffRow, ModuleConfig, BIN_SWATCHES, MAX_DROPOFF_POINTS, PUT_OUT};

const COLLECTION_ON: &str = "has_collection=true";
const BIN_ROOM_ON: &str = "bin_room_enabled=true";
const COMPOST_ON: &str = "compost_enabled=true";
const DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// Host configuration page.
// « Sortir les bacs » dans À venir, la veille d'une collecte : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Recycle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::WasteEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Recycle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // No Save button — the shell saves the form (`updateConfig`, taken by the platform).
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(collection_section(&config, &ctx))
                .child(bins_section(&config, &ctx))
                .child(bin_room_section(&config, &ctx))
                .child(dropoff_section(&config, &ctx))
                .child(compost_section(&config, &ctx)),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 Collecte ─────────────────────────────────────────────────────────────

fn collection_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("has_collection")
        .label("i18n:host.hasCollection.label")
        .description("i18n:host.hasCollection.hint")
        .checked(config.has_collection())
        .into()];
    children.push(
        field(
            config,
            ctx,
            "put_out",
            "i18n:host.putOut.label",
            RadioGroup::new()
                .name("put_out")
                .options(
                    PUT_OUT
                        .iter()
                        .map(|key| ChoiceOption::new(*key, format!("i18n:host.putOut.label.{key}")))
                        .collect(),
                )
                .value(config.put_out()),
            None,
        )
        .required(true)
        .visibleWhen(COLLECTION_ON)
        .into(),
    );
    children.push(
        field(
            config,
            ctx,
            "collection_schedule",
            "i18n:host.collectionNote.label",
            TextArea::new()
                .name("collection_schedule")
                .rows(2)
                .value(config.collection_schedule.host_value(ctx))
                .placeholder("i18n:host.collectionNote.placeholder"),
            None,
        )
        .visibleWhen(COLLECTION_ON)
        .into(),
    );
    // Les jours cochés pour tout le logement, d'avant les jours par bac : gardés lisibles (et
    // décochables) tant qu'il en reste un.
    let days = config.global_days();
    if days.iter().any(|(_, ticked)| *ticked) {
        children.push(
            Field::new()
                .label("i18n:host.collection.days")
                .visibleWhen(COLLECTION_ON)
                .child(FieldHint::new().text("i18n:host.collection.days.legacy"))
                .child(
                    Stack::new().gap(6.0).children(
                        days.into_iter()
                            .map(|(day, ticked)| {
                                let name = format!("collects_{day}");
                                Toggle::new()
                                    .name(name)
                                    .label(format!("i18n:host.collection.day.{day}"))
                                    .checked(ticked)
                                    .into()
                            })
                            .collect(),
                    ),
                )
                .into(),
        );
    }
    children.push(
        field(
            config,
            ctx,
            "host_reminder",
            "i18n:host.hostReminder.label",
            Toggle::new()
                .name("host_reminder")
                .checked(config.host_reminder),
            Some("i18n:host.hostReminder.hint"),
        )
        .visibleWhen(COLLECTION_ON)
        .into(),
    );
    // La consigne de sortie d'avant le message de collecte : lisible et effaçable, tant qu'elle
    // dit quelque chose.
    if !config.takeout_note.is_blank() {
        children.push(
            field(
                config,
                ctx,
                "takeout_note",
                "i18n:host.takeout.label",
                TextArea::new()
                    .name("takeout_note")
                    .rows(2)
                    .value(config.takeout_note.host_value(ctx)),
                Some("i18n:host.takeout.desc"),
            )
            .into(),
        );
    }
    section("collection", None, children)
}

// ── 2.2 Bacs ─────────────────────────────────────────────────────────────────

fn bins_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, "bins", config.bins.len(), MAX_BINS);
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| bin_row(index, config, ctx))
        .collect();

    let mut list = StepList::new()
        .addLabel("i18n:host.bins.add")
        .removeLabel("i18n:host.bins.remove")
        .emptyTitle("i18n:host.bins.emptyTitle")
        .emptyDescription("i18n:host.bins.emptyDescription")
        .itemKeyPrefix("bins")
        .children(rows);
    // À la borne, plus d'« Ajouter » : la phrase dit pourquoi (règles communes §2.4).
    if rows_count < MAX_BINS {
        list = list.addAction(emit_input(BinRowCount {
            bins_count: rows_count + 1,
        }));
    }
    let mut children: Vec<Component> = vec![list.into()];
    if rows_count >= MAX_BINS {
        children.push(FieldHint::new().text("i18n:host.bins.tooMany").into());
    }
    if let Some(error) = config.error_of("bins") {
        children.push(notice(error.get(&ctx.locale), Tone::Danger));
    }
    // « Le bac jaune n'a pas de jour de collecte. » : un avertissement, la publication passe.
    if let Some(bin) = config.bin_without_days() {
        children.push(notice(
            bin.no_days_warning().get(&ctx.locale),
            Tone::Warning,
        ));
    }
    section("bins", None, children)
}

fn bin_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let bin: Option<&BinRow> = config.bins.get(index);
    let text =
        |pick: fn(&BinRow) -> &I18nText| bin.map(|b| pick(b).host_value(ctx)).unwrap_or_default();
    let name = |key: &str| format!("bins.{index}.{key}");
    // A filled row sends its id, so a save merges into it (and keeps its other languages). A
    // blank slot has nothing to keep — and an id would make it count as filled.
    let id = bin
        .filter(|b| !b.is_blank())
        .map(|b| sdui::row_id("bins", index, Some(&b.id)));

    let mut children: Vec<Component> = id.into_iter().collect();
    children.push(
        field(
            config,
            ctx,
            &name("title"),
            "i18n:host.bin.title",
            TextInput::new()
                .name(name("title"))
                .value(text(|b| &b.title))
                .placeholder("i18n:host.bin.title.placeholder"),
            None,
        )
        .into(),
    );
    // Huit pastilles, chacune avec son nom : la couleur seule ne dit rien à qui la distingue mal.
    let color = bin.map_or("grey", BinRow::color_key);
    children.push(
        field(
            config,
            ctx,
            &name("color"),
            "i18n:host.bin.color",
            ChoiceList::new()
                .name(name("color"))
                .layout(ChoiceListLayout::Compact)
                .choices(
                    BIN_SWATCHES
                        .iter()
                        .map(|(key, _)| {
                            ChoiceOption::new(*key, format!("i18n:host.bin.color.{key}"))
                        })
                        .collect(),
                )
                .value(color),
            None,
        )
        .required(true)
        .into(),
    );
    children.push(
        field(
            config,
            ctx,
            &name("items"),
            "i18n:host.bin.items",
            TextArea::new()
                .name(name("items"))
                .rows(2)
                .value(text(|b| &b.items))
                .placeholder("i18n:host.bin.items.placeholder"),
            None,
        )
        .required(true)
        .into(),
    );
    // Sans ramassage (zone rurale), les jours n'ont pas d'objet.
    let chosen = bin.map(|b| b.days.clone()).unwrap_or_default();
    children.push(
        field(
            config,
            ctx,
            &name("days"),
            "i18n:host.bin.days",
            ChoiceList::new()
                .name(name("days"))
                .multi(true)
                .layout(ChoiceListLayout::Compact)
                .choices(
                    DAYS.iter()
                        .map(|day| {
                            ChoiceOption::new(*day, format!("i18n:host.collection.day.{day}"))
                        })
                        .collect(),
                )
                .value(serde_json::to_string(&chosen).unwrap_or_default()),
            None,
        )
        .visibleWhen(COLLECTION_ON)
        .into(),
    );
    children.push(
        field(
            config,
            ctx,
            &name("note"),
            "i18n:host.bin.note",
            TextInput::new()
                .name(name("note"))
                .value(text(|b| &b.note))
                .placeholder("i18n:host.bin.note.placeholder"),
            None,
        )
        .into(),
    );
    // « Où il se trouve », d'avant la spec : lisible et effaçable tant qu'il dit quelque chose.
    if bin.is_some_and(|b| !b.location.is_blank()) {
        children.push(
            field(
                config,
                ctx,
                &name("location"),
                "i18n:host.bin.location",
                TextInput::new()
                    .name(name("location"))
                    .value(text(|b| &b.location)),
                None,
            )
            .into(),
        );
    }

    Stack::new()
        .id(format!("bin-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

// ── 2.3 Local poubelles ──────────────────────────────────────────────────────

fn bin_room_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("bin_room_enabled")
        .label("i18n:host.binRoom.enabled")
        .checked(config.has_bin_room())
        .into()];
    children.push(
        field(
            config,
            ctx,
            "bin_room_where",
            "i18n:host.binRoom.where",
            TextInput::new()
                .name("bin_room_where")
                .value(config.bin_room_where.host_value(ctx))
                .placeholder("i18n:host.binRoom.where.placeholder"),
            None,
        )
        .required(true)
        .visibleWhen(BIN_ROOM_ON)
        .into(),
    );
    let mut picker = AddressMapPicker::new()
        .addressName("bin_room_address")
        .latName("bin_room_lat")
        .lngName("bin_room_lng")
        .address(config.bin_room_address.as_str())
        .label("i18n:host.binRoom.position")
        .hint("i18n:host.binRoom.position.hint");
    if let Some((lat, lng)) = config.bin_room_position() {
        picker = picker.lat(lat).lng(lng);
    }
    children.push(Field::new().visibleWhen(BIN_ROOM_ON).child(picker).into());
    if config.has_bin_room() {
        if let Some((_, far)) = warning_of(config, ctx, "bin_room_lat") {
            children.push(notice(far.get(&ctx.locale), Tone::Warning));
        }
    }
    let mut code = SecretInput::new()
        .name("bin_room_code")
        .value(String::new());
    // Jamais renvoyé : vide, la plateforme garde le code enregistré — le placeholder le dit.
    if !config.bin_room_code.trim().is_empty() {
        code = code.placeholder("i18n:host.binRoom.code.keep");
    }
    children.push(
        field(
            config,
            ctx,
            "bin_room_code",
            "i18n:host.binRoom.code",
            code,
            Some("i18n:host.binRoom.code.hint"),
        )
        .visibleWhen(BIN_ROOM_ON)
        .into(),
    );
    children.push(
        field(
            config,
            ctx,
            "bin_room_hours_range",
            "i18n:host.binRoom.hours",
            TimeRange::new()
                .name("bin_room_hours_range")
                .value(config.bin_room_hours_range.trim()),
            Some("i18n:host.binRoom.hours.hint"),
        )
        .visibleWhen(BIN_ROOM_ON)
        .into(),
    );
    // Les heures écrites en toutes lettres, d'avant la plage : lisibles et effaçables.
    if !config.bin_room_hours.is_blank() {
        children.push(
            field(
                config,
                ctx,
                "bin_room_hours",
                "i18n:host.binRoom.hours.legacy",
                TextInput::new()
                    .name("bin_room_hours")
                    .value(config.bin_room_hours.host_value(ctx)),
                None,
            )
            .visibleWhen(BIN_ROOM_ON)
            .into(),
        );
    }
    children.push(
        field(
            config,
            ctx,
            "bin_room_steps",
            "i18n:host.binRoom.label",
            TextArea::new()
                .name("bin_room_steps")
                .value(config.bin_room_steps.host_value(ctx))
                .rows(3)
                .placeholder("i18n:host.binRoom.placeholder"),
            Some("i18n:host.binRoom.hint"),
        )
        .visibleWhen(BIN_ROOM_ON)
        .into(),
    );
    section("binRoom", None, children)
}

// ── 2.4 Points d'apport ──────────────────────────────────────────────────────

/// Les points d'apport : une carte de tous les points, puis une ligne par point, dessinées à la
/// demande et bornées.
fn dropoff_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let stored = config.parse_dropoff_points().len();
    let rows_count = draft_rows(ctx, "dropoff_points", stored, MAX_DROPOFF_POINTS);
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| dropoff_row(index, config, ctx))
        .collect();

    let mut children: Vec<Component> = overview_map(config, ctx).into_iter().collect();
    let mut list = StepList::new()
        .addLabel("i18n:host.dropoff.add")
        .removeLabel("i18n:host.dropoff.remove")
        .emptyTitle("i18n:host.dropoff.emptyTitle")
        .emptyDescription("i18n:host.dropoff.emptyDescription")
        .itemKeyPrefix("dropoff_points")
        .children(rows);
    if rows_count < MAX_DROPOFF_POINTS {
        list = list.addAction(emit_input(RowCount {
            dropoff_points_count: rows_count + 1,
        }));
    }
    children.push(list.into());
    if rows_count >= MAX_DROPOFF_POINTS {
        children.push(FieldHint::new().text("i18n:host.dropoff.tooMany").into());
    }
    if let Some(error) = config.error_of("dropoff_points") {
        children.push(notice(error.get(&ctx.locale), Tone::Danger));
    }
    section("dropoff", Some("i18n:host.dropoff.subtitle"), children)
}

/// Le logement et tous les points situés, sur une carte — rien tant qu'aucun n'est placé.
fn overview_map(config: &ModuleConfig, ctx: &HostContext) -> Option<Component> {
    let points: Vec<MapMarker> = config
        .parse_dropoff_points()
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let (lat, lng) = point.coordinates()?;
            let marker =
                MapMarker::new(format!("dropoff-{index}"), lat, lng).kind(MapMarkerKind::Poi);
            let title = point.title.host_value(ctx);
            Some(if title.trim().is_empty() {
                marker
            } else {
                marker.label(title)
            })
        })
        .collect();
    let first = points.first()?;
    let (lat, lng) = ctx
        .property
        .coordinates
        .map_or((first.lat, first.lng), |home| (home.lat, home.lng));
    let home = ctx.property.coordinates.map(|home| {
        MapMarker::new("property", home.lat, home.lng)
            .label("i18n:guest.map.property")
            .kind(MapMarkerKind::Property)
    });
    Some(
        Map::new()
            .viewport(MapViewport::new(lat, lng, Some(13.0)))
            .markers(home.into_iter().chain(points).collect())
            .label("i18n:guest.map.label")
            .isStatic(true)
            .interactionMode(MapInteractionMode::None)
            .into(),
    )
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, key: &str, stored: usize, max: usize) -> usize {
    match ctx.input_u64(&format!("{key}_count")) {
        Some(asked) => (asked as usize).clamp(1, max),
        None => stored.clamp(1, max),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    dropoff_points_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct BinRowCount {
    bins_count: usize,
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn dropoff_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let point: Option<&DropoffRow> = config.dropoff_points.get(index);
    let name = |key: &str| format!("dropoff_points.{index}.{key}");
    let id = point
        .filter(|p| !p.is_blank())
        .map(|p| sdui::row_id("dropoff_points", index, Some(&p.id)));

    let mut picker = AddressMapPicker::new()
        .addressName(name("address"))
        .latName(name("lat"))
        .lngName(name("lng"))
        .address(point.and_then(|p| p.address.as_deref()).unwrap_or(""))
        .label("i18n:host.dropoff.position");
    if let Some((lat, lng)) = point.and_then(DropoffRow::coordinates) {
        picker = picker.lat(lat).lng(lng);
    }

    let mut children: Vec<Component> = id.into_iter().collect();
    children.push(
        field(
            config,
            ctx,
            &name("title"),
            "i18n:host.dropoff.pointTitle",
            TextInput::new()
                .name(name("title"))
                .value(point.map(|p| p.title.host_value(ctx)).unwrap_or_default())
                .placeholder("i18n:host.dropoff.pointTitle.placeholder"),
            None,
        )
        .required(true)
        .into(),
    );
    // Le sélecteur n'a pas d'erreur à lui : le champ qui l'entoure la porte, ou l'avertissement
    // d'une épingle loin du logement.
    let mut pin = Field::new().child(picker);
    if let Some(error) = config.error_of(&name("lat")) {
        pin = pin.error(error.get(&ctx.locale).to_string());
    } else if let Some((_, far)) = warning_of(config, ctx, &name("lat")) {
        pin = pin.child(FieldHint::new().text(far.get(&ctx.locale)));
    }
    children.push(pin.into());
    let checks: Vec<Component> = [
        (
            "accepts_household",
            point.is_some_and(|p| p.accepts_household),
        ),
        (
            "accepts_packaging",
            point.is_some_and(|p| p.accepts_packaging),
        ),
        ("accepts_glass", point.is_some_and(|p| p.accepts_glass)),
        ("accepts_paper", point.is_some_and(|p| p.accepts_paper)),
        ("accepts_textile", point.is_some_and(|p| p.accepts_textile)),
        (
            "accepts_batteries",
            point.is_some_and(|p| p.accepts_batteries),
        ),
    ]
    .into_iter()
    .map(|(key, ticked)| {
        Checkbox::new()
            .name(name(key))
            .label(format!("i18n:host.dropoff.{key}"))
            .checked(ticked)
            .into()
    })
    .collect();
    children.push(
        field(
            config,
            ctx,
            &name("accepts_household"),
            "i18n:host.dropoff.accepts",
            Stack::new().gap(6.0).children(checks),
            None,
        )
        .required(true)
        .into(),
    );
    children.push(
        field(
            config,
            ctx,
            &name("note"),
            "i18n:host.dropoff.note",
            TextInput::new()
                .name(name("note"))
                .value(point.map(|p| p.note.host_value(ctx)).unwrap_or_default())
                .placeholder("i18n:host.dropoff.note.placeholder"),
            None,
        )
        .into(),
    );

    Stack::new()
        .id(format!("dropoff-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

// ── 2.5 Composteur ───────────────────────────────────────────────────────────

fn compost_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("compost_enabled")
        .label("i18n:host.compost.enabled")
        .checked(config.compost_enabled)
        .into()];
    children.push(
        field(
            config,
            ctx,
            "compost_location",
            "i18n:host.compost.location",
            TextInput::new()
                .name("compost_location")
                .value(config.compost_location.host_value(ctx))
                .placeholder("i18n:host.compost.location.placeholder"),
            Some("i18n:host.compost.location.hint"),
        )
        .required(true)
        .visibleWhen(COMPOST_ON)
        .into(),
    );
    // Les deux listes, pré-remplies tant que le composteur n'a jamais été ouvert (§4) : le
    // premier enregistrement les garde, et une liste vidée ensuite reste vide.
    for (name, value) in [
        ("compost_accepted", &config.compost_accepted),
        ("compost_refused", &config.compost_refused),
    ] {
        let key = name.trim_start_matches("compost_");
        let mut lines = value.host_value(ctx).to_string();
        if !config.compost_enabled && lines.trim().is_empty() {
            let proposed = format!("host.compost.{key}.default");
            // Une clé absente revient telle quelle de l'hôte : jamais elle dans le formulaire.
            lines = t!(&proposed)
                .ok()
                .filter(|text| *text != proposed)
                .unwrap_or_else(|| bundle_text!(&proposed).get(&ctx.locale).to_string());
        }
        children.push(
            field(
                config,
                ctx,
                name,
                &format!("i18n:host.compost.{key}"),
                TextArea::new()
                    .name(name)
                    .rows(4)
                    .value(lines)
                    .placeholder("i18n:host.compost.lines.placeholder"),
                Some("i18n:host.compost.lines.hint"),
            )
            .visibleWhen(COMPOST_ON)
            .into(),
        );
    }
    section("compost", None, children)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// One card of the settings page (§2.2 of the common rules): a `Section`, never a `Card`.
fn section(key: &str, subtitle: Option<&str>, children: Vec<Component>) -> Component {
    let section = Section::new()
        .title(format!("i18n:host.section.{key}"))
        .children(children);
    match subtitle {
        Some(subtitle) => section.subtitle(subtitle),
        None => section,
    }
    .into()
}

/// Libellé, contrôle, l'aide dessous, et le message de [`ModuleConfig::error_of`] s'il y en a un.
fn field(
    config: &ModuleConfig,
    ctx: &HostContext,
    name: &str,
    label: &str,
    control: impl Into<Component>,
    hint: Option<&str>,
) -> Field {
    let mut field = Field::new().name(name).label(label).child(control);
    if let Some(hint) = hint {
        field = field.child(FieldHint::new().text(hint));
    }
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// L'avertissement de `name`, s'il y en a un (épingle loin du logement).
fn warning_of(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Option<(String, I18nText)> {
    config
        .warnings(ctx.property.coordinates)
        .into_iter()
        .find(|(field, _)| field == name)
}

fn notice(message: &str, tone: Tone) -> Component {
    InlineNotice::new().message(message).tone(tone).into()
}
