//! Host dashboard surface — les réglages « Randonnées » (spec Randonnées §2), mode page.
//!
//! Trois `Section`s, dans l'ordre de la planche : « Itinéraires », un tableau ordonnable (l'ordre
//! du livret, les deux premiers sur la carte d'accueil) dont chaque ligne s'ouvre dans le panneau
//! latéral (`panel: "sheet"`), ses champs nommés `trails.<rang>.<clé>` ; « Pour aller plus loin »,
//! le lien de la commune ; « Sur votre page publique ». Une erreur s'affiche sous son champ et
//! bloque « Publier » (`publishReadiness`), jamais l'enregistrement.

use portaki_sdk::contracts::i18n::I18nText;
mod stats;

pub use stats::{render_host_stats, stats_summary};

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Button, ChoiceList, DataTable, Field, FieldHint, FileUpload, Form,
    ImageUpload, InlineNotice, NumberInput, Page, Section, Stack, Text, TextArea, TextInput,
    ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::config::{
    ModuleConfig, TrailRow, FAR_START_METRES, LEVELS, MAX_TRAILS, PUBLIC_MAX, PUBLIC_MIN, SHAPES,
};
use crate::gpx::Track;

/// Les itinéraires ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_TRAILS: &str = "newTrails";
/// L'id de la ligne dont l'hôte demande « Recalculer depuis la trace ».
const RECOMPUTE: &str = "recomputeTrail";

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::Mountain
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let view = View::load(&ctx)?;

    // No Save button — the dashboard autosaves and owns « Publier ».
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(trails_section(&view, &ctx))
                .child(commune_section(&view, &ctx))
                .child(public_section(&view.config, &ctx)),
        ),
    )
    .with_id(MAIN))
}

/// Ce que la page lit : la config, ses erreurs, le logement, et les lignes du tableau.
struct View {
    config: ModuleConfig,
    problems: Vec<(String, I18nText)>,
    home: Option<(f64, f64)>,
    /// Aujourd'hui en `MMJJ`, pour le badge « Hors saison ».
    today: Option<u32>,
    /// Les lignes, dans l'ordre stocké : `(rang stocké, itinéraire)`, `None` pour une ajoutée.
    rows: Vec<(Option<usize>, TrailRow)>,
    /// Les ajoutées pas encore enregistrées, pour les garder d'un rendu à l'autre.
    added: Vec<String>,
    /// Une ligne remplie sans id : la plateforme ne la retrouve que par son rang, donc pas de
    /// glisser-déposer tant qu'elle existe — une ligne déplacée jamais ouverte n'envoie que son id,
    /// et le rang désignerait une autre ligne. Le premier enregistrement donne les id.
    legacy: bool,
}

impl View {
    fn load(ctx: &HostContext) -> Result<Self> {
        let config = ModuleConfig::load(ctx)?;
        let home = ctx.property.coordinates.map(|point| (point.lat, point.lng));
        let stored = &config.trails;

        // La plateforme fusionne une ligne reçue avec la ligne stockée de même id, sinon de même
        // rang. Une ligne remplie sans id ne se retrouve que par son rang : tant qu'il y en a une,
        // le tableau garde toutes les lignes enregistrées, vides comprises. L'id qu'elle reçoit
        // est son identifiant de route (`t<rang>`), celui que `public_trails` désigne déjà.
        let legacy = stored
            .iter()
            .any(|t| !t.is_blank() && t.id.trim().is_empty());
        let mut rows: Vec<(Option<usize>, TrailRow)> = stored
            .iter()
            .enumerate()
            .filter(|(_, t)| legacy || !t.is_blank())
            .map(|(index, t)| {
                let mut t = t.clone();
                t.id = t.route_id(index);
                (Some(index), t)
            })
            .collect();
        let added: Vec<String> = ctx
            .input_str(NEW_TRAILS)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty() && !rows.iter().any(|(_, t)| t.id == *id))
            .map(str::to_string)
            .collect();
        for id in &added {
            if rows.len() >= MAX_TRAILS {
                break;
            }
            rows.push((
                None,
                TrailRow {
                    id: id.clone(),
                    ..TrailRow::default()
                },
            ));
        }
        Ok(Self {
            problems: config.problems(home),
            config,
            home,
            today: portaki_sdk::host::time::now()
                .ok()
                .and_then(|now| now.format("%m%d").to_string().parse().ok()),
            rows,
            added,
            legacy,
        })
    }

    /// Le message sous `field`, dans la langue de l'hôte.
    fn error(&self, field: &str, ctx: &HostContext) -> Option<String> {
        self.problems
            .iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error.get(&ctx.locale).to_string())
    }
}

fn emit_input(input: Map<String, Value>) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(Value::Object(input)))
}

fn added_input(added: &[String]) -> Map<String, Value> {
    let mut input = Map::new();
    input.insert(NEW_TRAILS.into(), Value::String(added.join(",")));
    input
}

/// §2.1 Itinéraires : le tableau ordonnable et le panneau de chaque ligne.
fn trails_section(view: &View, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![Text::new()
        .text(format!("{} / {MAX_TRAILS}", view.rows.len()))
        .variant(TextVariant::Caption)
        .into()];
    // « Ajoutez au moins un itinéraire. » tant qu'aucun n'est complet ; « 30 au maximum » vient
    // de `problems`.
    let error = view.error("trails", ctx).or_else(|| {
        view.config.parse_trails().is_empty().then(|| {
            crate::i18n::text("publish.trails.hint")
                .get(&ctx.locale)
                .to_string()
        })
    });
    if let Some(error) = error {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    let mut table = DataTable::new()
        .rowName("trails")
        .sortable(!view.legacy)
        .panel("sheet")
        .columns(json!([
            { "key": "name", "label": "i18n:host.column.trail", "sub": "start" },
            { "key": "level", "label": "i18n:host.level.label" },
            { "key": "duration", "label": "i18n:host.column.duration", "mono": true },
            { "key": "distance", "label": "i18n:host.column.distance", "mono": true },
            { "key": "ascent", "label": "i18n:host.column.ascent", "mono": true },
            { "key": "badge", "label": "i18n:host.column.season", "kind": "pill" },
        ]))
        .rows(Value::Array(
            view.rows
                .iter()
                .map(|(_, trail)| row(view, trail, ctx))
                .collect(),
        ))
        .children(
            view.rows
                .iter()
                .enumerate()
                .map(|(rank, (stored, trail))| trail_panel(view, rank, *stored, trail, ctx))
                .collect(),
        );
    if view.rows.len() < MAX_TRAILS {
        let mut added = view.added.clone();
        added.push(Uuid::new_v4().to_string());
        table = table
            .addLabel("i18n:host.trails.add")
            .addAction(emit_input(added_input(&added)));
    }
    children.push(table.into());
    children.push(FieldHint::new().text("i18n:host.trails.add.hint").into());

    Section::new()
        .title("i18n:host.trails.title")
        .subtitle("i18n:host.trails.subtitle")
        .children(children)
        .into()
}

/// Une ligne du tableau : Itinéraire (départ dessous) · Niveau · Durée · Distance · D+ · saison.
fn row(view: &View, trail: &TrailRow, ctx: &HostContext) -> Value {
    let dash = || "—".to_string();
    let start = if trail.coordinates().is_some() && at_property(view, trail) {
        t!("guest.start.atProperty").unwrap_or_default()
    } else {
        match trail.address.as_deref().map(str::trim) {
            Some(place) if !place.is_empty() => {
                t!("guest.start", place = place).unwrap_or_default()
            }
            _ => String::new(),
        }
    };
    let off_season = view.today.is_some_and(|today| !trail.in_season(today));
    json!({
        "id": trail.id,
        "name": trail.title.host_value(ctx),
        "start": start,
        "level": trail.level_key().map(|level| format!("i18n:host.level.label.{level}")).unwrap_or_default(),
        "duration": trail.duration().map_or_else(dash, crate::format::duration),
        "distance": trail.distance().map_or_else(dash, crate::format::distance),
        "ascent": trail.elevation().map_or_else(dash, crate::format::elevation),
        "badge": if off_season { "i18n:guest.offSeason" } else { "" },
    })
}

/// Le départ est-il au logement : la réponse enregistrée, sinon ce que la position suggère.
fn at_property(view: &View, trail: &TrailRow) -> bool {
    trail
        .starts_at_property
        .unwrap_or_else(|| trail.near_property(view.home))
}

/// Ce que la trace de la ligne propose, quand il faut la lire.
///
/// Lue sur demande (« Recalculer depuis la trace ») ou tant qu'une valeur qu'elle calcule manque
/// encore — juste après le dépôt. Pas à chaque rendu : ce serait un appel par ligne, pour des
/// valeurs déjà là. Une trace refusée rend la clé de son message ; une trace que la plateforme ne
/// sert pas rend `None`.
fn read_track(trail: &TrailRow, forced: bool) -> Option<std::result::Result<Track, &'static str>> {
    let reference = trail.gpx_ref()?;
    let missing = trail.title.is_blank()
        || trail.distance().is_none()
        || trail.elevation().is_none()
        || trail.duration().is_none()
        || trail.shape_key().is_none()
        || trail.coordinates().is_none();
    if !forced && !missing {
        return None;
    }
    let bytes = portaki_sdk::host::files::read(reference).ok()?;
    Some(match crate::gpx::refusal(&bytes) {
        Some(refusal) => Err(refusal),
        None => crate::gpx::read(&String::from_utf8_lossy(&bytes)).ok_or("host.trails.gpx.invalid"),
    })
}

/// Les valeurs du panneau : celles de la ligne, complétées — ou remplacées sur « Recalculer » —
/// par ce que la trace calcule. Le titre et le départ ne sont jamais remplacés, seulement
/// proposés quand ils manquent ; le type de l'hôte reste le sien (une trace ne distingue pas un
/// aller-retour d'un aller simple).
struct Values {
    title: String,
    duration: Option<f64>,
    distance: Option<f64>,
    elevation: Option<f64>,
    shape: &'static str,
    start: Option<(f64, f64)>,
}

fn values(trail: &TrailRow, title: &str, track: Option<&Track>, forced: bool) -> Values {
    let pick = |stored: Option<f64>, computed: Option<f64>| {
        if forced {
            computed.or(stored)
        } else {
            stored.or(computed)
        }
    };
    Values {
        title: match track.and_then(|t| t.name.clone()) {
            Some(name) if title.trim().is_empty() => name,
            _ => title.to_string(),
        },
        duration: pick(
            trail
                .duration_total()
                .filter(|_| trail.duration().is_some()),
            track.map(|t| t.duration_min().round()),
        ),
        distance: pick(
            trail.distance(),
            track.map(|t| crate::format::round_tenth(t.distance_km)),
        ),
        elevation: pick(
            trail.elevation().map(f64::from),
            track
                .filter(|t| t.has_elevation)
                .map(|t| t.elevation_m.round()),
        ),
        shape: trail
            .shape_key()
            .or_else(|| track.map(|t| t.shape))
            .unwrap_or(""),
        start: trail.coordinates().or_else(|| {
            track
                .and_then(|t| t.points.first())
                .map(|point| (point.lat, point.lng))
        }),
    }
}

/// Le panneau d'une ligne, dans l'ordre de la planche. Les champs sont nommés au rang affiché ;
/// les erreurs se lisent au rang stocké, celui de `problems`.
fn trail_panel(
    view: &View,
    rank: usize,
    stored: Option<usize>,
    trail: &TrailRow,
    ctx: &HostContext,
) -> Component {
    let name = |key: &str| format!("trails.{rank}.{key}");
    let error =
        |key: &str| stored.and_then(|index| view.error(&format!("trails.{index}.{key}"), ctx));
    let field = |key: &str, label: &str| {
        let field = Field::new().name(name(key)).label(label);
        match error(key) {
            Some(error) => field.error(error),
            None => field,
        }
    };

    let forced = ctx.input_str(RECOMPUTE) == Some(trail.id.as_str());
    let read = read_track(trail, forced);
    let track = read.as_ref().and_then(|read| read.as_ref().ok());
    let values = values(trail, trail.title.host_value(ctx), track, forced);

    let mut children: Vec<Component> = Vec::new();

    // La trace en premier : elle calcule le reste.
    let mut gpx = field("gpx_file", "i18n:host.trails.gpx");
    if let Some(Err(refusal)) = &read {
        gpx = gpx.error(crate::i18n::text(refusal).get(&ctx.locale).to_string());
    }
    children.push(
        gpx.child(
            // `.gpx` en plus du type : la plupart des navigateurs n'en connaissent pas le MIME.
            // ponytail: plafonné à ce que le module peut relire (2 Mo), pas aux 5 Mo de la spec —
            // au-delà, la trace serait déposée mais illisible ; à monter avec `files::read`.
            FileUpload::new()
                .name(name("gpx_file"))
                .value(trail.gpx_file.as_str())
                .accept("application/gpx+xml,.gpx")
                .maxBytes(portaki_sdk::limits::HOST_FILE_READ_MAX_BYTES as u32),
        )
        .child(FieldHint::new().text("i18n:host.trails.gpx.hint"))
        .into(),
    );

    children.push(
        field("title", "i18n:host.trails.rowTitle")
            .required(true)
            .child(
                TextInput::new()
                    .name(name("title"))
                    .value(values.title.as_str())
                    .placeholder("i18n:host.trails.rowTitle.placeholder"),
            )
            .into(),
    );

    children.push(
        field("level", "i18n:host.level.label")
            .required(true)
            .child(segmented(
                name("level"),
                "i18n:host.level.label",
                LEVELS,
                trail.level_key().unwrap_or(""),
            ))
            .into(),
    );

    // Durée en heures et minutes, distance, dénivelé : côte à côte, l'unité après la case.
    let (hours, minutes) = values
        .duration
        .map(|total| ((total / 60.0).floor(), (total % 60.0).round()))
        .unzip();
    let mut elevation = field("elevation_m", "i18n:host.trails.elevation");
    if track.is_some_and(|t| !t.has_elevation) && values.elevation.is_none() {
        elevation = elevation.error(
            crate::i18n::text("host.trails.gpx.noElevation")
                .get(&ctx.locale)
                .to_string(),
        );
    }
    children.push(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(10.0)
            .children(vec![
                field("duration_min", "i18n:host.trails.duration")
                    .child(
                        Stack::new()
                            .direction(StackDirection::Horizontal)
                            .gap(6.0)
                            .children(vec![
                                number(name("duration_h"), 0.0, 24.0, hours),
                                unit("i18n:host.unit.hours"),
                                number(name("duration_m"), 0.0, 59.0, minutes),
                                unit("i18n:host.unit.minutes"),
                            ]),
                    )
                    .child(FieldHint::new().text("i18n:host.trails.duration.hint"))
                    .into(),
                field("distance_km", "i18n:host.trails.distance")
                    .child(with_unit(
                        number(
                            name("distance_km"),
                            0.0,
                            crate::config::DISTANCE_KM.1,
                            values.distance,
                        ),
                        "i18n:host.unit.km",
                    ))
                    .into(),
                elevation
                    .child(with_unit(
                        number(
                            name("elevation_m"),
                            0.0,
                            crate::config::ELEVATION_M.1,
                            values.elevation,
                        ),
                        "i18n:host.unit.metres",
                    ))
                    .into(),
            ])
            .into(),
    );
    if forced && track.is_some() {
        children.push(
            FieldHint::new()
                .text("i18n:host.trails.prefill.done")
                .into(),
        );
    }
    // ponytail: offert dès qu'il y a une trace, pas seulement quand une valeur a été corrigée —
    // le savoir demanderait de relire chaque trace à chaque rendu.
    if trail.gpx_ref().is_some() {
        let mut input = added_input(&view.added);
        input.insert(RECOMPUTE.into(), Value::String(trail.id.clone()));
        children.push(
            Button::new()
                .label("i18n:host.trails.prefill")
                .variant(ButtonVariant::Outline)
                .action(emit_input(input))
                .into(),
        );
    }

    children.push(
        field("shape", "i18n:host.shape.label")
            .child(segmented(
                name("shape"),
                "i18n:host.shape.label",
                SHAPES,
                values.shape,
            ))
            .into(),
    );

    children.push(start_field(view, trail, &values, &name, error("lat"), ctx));

    children.push(
        field("description", "i18n:host.trails.description")
            .child(
                TextArea::new()
                    .name(name("description"))
                    .value(trail.description.host_value(ctx))
                    .rows(4)
                    .placeholder("i18n:host.trails.description.placeholder"),
            )
            .into(),
    );
    children.push(
        field("photo", "i18n:host.trails.photo")
            .child(
                ImageUpload::new()
                    .name(name("photo"))
                    .value(trail.photo.as_str()),
            )
            .into(),
    );
    children.push(
        field("link_url", "i18n:host.trails.link")
            .child(
                TextInput::new()
                    .name(name("link_url"))
                    .value(trail.link_url.as_str())
                    .placeholder("i18n:host.trails.link.placeholder"),
            )
            .child(FieldHint::new().text("i18n:host.trails.link.hint"))
            .into(),
    );
    children.push(
        field("tips", "i18n:host.trails.tips")
            .child(
                TextArea::new()
                    .name(name("tips"))
                    .value(trail.tips.host_value(ctx))
                    .rows(3)
                    .placeholder("i18n:host.trails.tips.placeholder"),
            )
            .child(FieldHint::new().text("i18n:host.trails.tips.hint"))
            .into(),
    );

    // La saison, du JJ/MM au JJ/MM ; une saison d'avant (`MM-JJ`) s'affiche dans ce format.
    let season = |key: &str, placeholder: &str, value: &str| -> Component {
        field(
            key,
            &format!("i18n:host.trails.season.{}", &key["season_".len()..]),
        )
        .child(
            TextInput::new()
                .name(name(key))
                .value(day_month(value))
                .placeholder(placeholder),
        )
        .into()
    };
    children.push(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(10.0)
            .children(vec![
                season("season_from", "01/04", &trail.season_from),
                season("season_to", "31/10", &trail.season_to),
            ])
            .into(),
    );
    children.push(FieldHint::new().text("i18n:host.trails.season.hint").into());

    Stack::new().gap(14.0).children(children).into()
}

/// Le départ : son libellé et le plan, sa distance au logement, et « le départ est au logement ».
fn start_field(
    view: &View,
    trail: &TrailRow,
    values: &Values,
    name: &dyn Fn(&str) -> String,
    error: Option<String>,
    ctx: &HostContext,
) -> Component {
    let mut picker = AddressMapPicker::new()
        .addressName(name("address"))
        .latName(name("lat"))
        .lngName(name("lng"))
        .address(trail.address.as_deref().unwrap_or(""))
        .hint("i18n:host.trails.start.hint");
    if let Some((lat, lng)) = values.start {
        picker = picker.lat(lat).lng(lng);
    }
    let mut field = Field::new()
        .name(name("lat"))
        .label("i18n:host.trails.start")
        .required(true);
    // Trop loin bloque ; absent avertit, avec le même message.
    let error = error.or_else(|| {
        (values.start.is_none() && !trail.is_blank()).then(|| {
            crate::i18n::text("host.trails.start.required")
                .get(&ctx.locale)
                .to_string()
        })
    });
    if let Some(error) = error {
        field = field.error(error);
    }
    field = field.child(picker);
    if let Some(metres) = view
        .home
        .zip(values.start)
        .map(|(home, start)| crate::format::haversine_metres(home, start))
    {
        let key = if metres >= FAR_START_METRES {
            "host.trails.start.distance.far"
        } else {
            "host.trails.start.distance"
        };
        if let Ok(text) = t!(key, distance = crate::format::walking(metres)) {
            field = field.child(FieldHint::new().text(text));
        }
    }
    // Pour la page publique, une fois le départ posé : la position réelle n'est connue qu'ici,
    // et le formulaire propose la réponse tant que l'hôte n'en a pas enregistré une.
    if trail.coordinates().is_some() {
        field = field.child(
            ToggleRow::new()
                .name(name("starts_at_property"))
                .label("i18n:host.trails.startsAtProperty")
                .checked(at_property(view, trail)),
        );
    }
    field.into()
}

/// « 04-01 » (une saison d'avant) → « 01/04 » ; le reste tel quel.
fn day_month(value: &str) -> String {
    let value = value.trim();
    match value.split_once('-') {
        Some((month, day)) if crate::config::parse_month_day(value).is_some() => {
            format!("{day:0>2}/{month:0>2}")
        }
        _ => value.to_string(),
    }
}

/// Un choix figé en contrôle segmenté : le niveau et la forme ne sont pas du texte libre (§2.23).
fn segmented(name: String, label: &str, values: &[&str], chosen: &str) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .layout(ChoiceListLayout::Segmented)
        .choices(
            values
                .iter()
                .map(|value| ChoiceOption::new(*value, format!("{label}.{value}")))
                .collect(),
        )
        .value(chosen)
}

/// Une mesure : vide vaut « pas renseigné », et la tuile disparaît chez le voyageur.
fn number(name: String, min: f64, max: f64, value: Option<f64>) -> Component {
    let mut input = NumberInput::new().name(name).min(min).max(max);
    if let Some(value) = value {
        input = input.value(value);
    }
    input.into()
}

fn unit(label: &str) -> Component {
    Text::new().text(label).variant(TextVariant::Caption).into()
}

fn with_unit(input: Component, label: &str) -> Stack {
    Stack::new()
        .direction(StackDirection::Horizontal)
        .gap(6.0)
        .children(vec![input, unit(label)])
}

/// §2.2 Pour aller plus loin : le lien de la commune, sous la liste du voyageur.
fn commune_section(view: &View, ctx: &HostContext) -> Component {
    let mut field = Field::new()
        .name("commune_url")
        .label("i18n:host.commune.label");
    if let Some(error) = view.error("commune_url", ctx) {
        field = field.error(error);
    }
    Section::new()
        .title("i18n:host.commune.title")
        .child(
            field
                .child(
                    TextInput::new()
                        .name("commune_url")
                        .value(view.config.commune_url.as_str())
                        .placeholder("i18n:host.commune.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.commune.hint")),
        )
        .into()
}

/// « Sur votre page publique » : l'interrupteur du bloc, puis 2 à 4 itinéraires en pastilles.
///
/// Le choix reste rendu sans itinéraire à choisir : le formulaire envoie toujours les clés
/// déclarées.
fn public_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let enabled = ctx.input_bool("public_enabled", config.public_enabled);
    let routed = config.routed_trails();
    let choices: Vec<ChoiceOption> = routed
        .iter()
        .map(|(id, trail)| ChoiceOption::new(id.clone(), trail.title.host_value(ctx).to_string()))
        .collect();
    let hint = if routed.is_empty() {
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
        .children(vec![
            ToggleRow::new()
                .name("public_enabled")
                .label("i18n:host.public.enabled")
                .icon(IconName::Home)
                .checked(enabled)
                .into(),
            Field::new()
                .name("public_trails")
                .label("i18n:host.public.trails")
                .child(
                    ChoiceList::new()
                        .name("public_trails")
                        .multi(true)
                        .layout(ChoiceListLayout::Compact)
                        .limit(PUBLIC_MAX as u32)
                        .choices(choices)
                        .value(serde_json::to_string(&config.public_trails).unwrap_or_default()),
                )
                .child(FieldHint::new().text(hint))
                .into(),
        ])
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(shape: &'static str) -> Track {
        Track {
            points: Vec::new(),
            distance_km: 3.0,
            elevation_m: 100.0,
            shape,
            has_elevation: true,
            name: Some("Cap".into()),
        }
    }

    fn shape_of(row: &TrailRow, track: Option<&Track>) -> &'static str {
        values(row, "", track, true).shape
    }

    /// Une trace propose ce qui manque ; « Recalculer » remplace les mesures, jamais le titre.
    #[test]
    fn the_trace_fills_the_gaps_and_recompute_replaces_the_measures() {
        let row = TrailRow {
            distance_km: Some(9.0),
            ..TrailRow::default()
        };
        let proposed = values(&row, "", Some(&track("loop")), false);
        assert_eq!(proposed.title, "Cap");
        assert_eq!(proposed.distance, Some(9.0));
        assert_eq!(proposed.elevation, Some(100.0));
        // 3 km à 4 km/h, plus 100 m de montée : 45 + 10 min.
        assert_eq!(proposed.duration, Some(55.0));
        let recomputed = values(&row, "Garoupe", Some(&track("loop")), true);
        assert_eq!(recomputed.title, "Garoupe");
        assert_eq!(recomputed.distance, Some(3.0));
        // Sans altitude, la trace ne propose pas de dénivelé.
        let flat = Track {
            has_elevation: false,
            ..track("loop")
        };
        assert_eq!(
            values(&TrailRow::default(), "", Some(&flat), true).elevation,
            None
        );
    }

    #[test]
    fn the_trace_does_not_override_the_host_s_shape() {
        let row = TrailRow {
            shape: "one_way".into(),
            ..TrailRow::default()
        };
        assert_eq!(shape_of(&row, Some(&track("round_trip"))), "one_way");
    }

    #[test]
    fn the_trace_fills_an_empty_shape_with_a_known_one() {
        let shape = shape_of(&TrailRow::default(), Some(&track("round_trip")));
        assert_eq!(shape, "round_trip");
        assert!(SHAPES.contains(&shape));
        assert_eq!(shape_of(&TrailRow::default(), None), "");
    }

    #[test]
    fn a_legacy_season_reads_as_day_month() {
        assert_eq!(day_month("04-01"), "01/04");
        assert_eq!(day_month("01/04"), "01/04");
        assert_eq!(day_month("avril"), "avril");
    }
}
