//! Host dashboard surface — l'onglet « Randonnées » de l'espace du logement.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Button, Card, ChoiceList, Field, FieldHint, Form, ImageUpload, NumberInput,
    Page, Select, Stack, StepList, Text, TextArea, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::config::{ModuleConfig, TrailRow, LEVELS, MAX_TRAILS, PUBLIC_MAX, PUBLIC_MIN, SHAPES};

/// Les bornes d'une mesure de randonnée : au-delà, c'est une faute de frappe.
const MAX_DURATION_MIN: f64 = crate::config::DURATION_MIN.1;
const MAX_DISTANCE_KM: f64 = crate::config::DISTANCE_KM.1;
const MAX_ELEVATION_M: f64 = crate::config::ELEVATION_M.1;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::Mountain
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // No Save button — the modules drawer owns the footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:host.intro")
                        .variant(TextVariant::Caption)
                        .into(),
                    trails_card(&config, &ctx),
                    commune_card(&config, &ctx),
                    public_card(&config, &ctx),
                ]))),
    )
    .with_id(MAIN))
}

/// Les itinéraires, en lignes dynamiques bornées — le motif de `rules` et `ical-sync`.
fn trails_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.parse_trails().len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| trail_row(index, config, ctx))
        .collect();

    Card::new()
        .title("i18n:host.trails.title")
        .subtitle("i18n:host.trails.subtitle")
        .icon(IconName::Mountain)
        .child(
            StepList::new()
                .addLabel("i18n:host.trails.add")
                .removeLabel("i18n:host.trails.remove")
                .emptyTitle("i18n:host.trails.emptyTitle")
                .emptyDescription("i18n:host.trails.emptyDescription")
                .itemKeyPrefix("trails")
                .addAction(emit_input(RowCount {
                    trails_count: (rows_count + 1).min(MAX_TRAILS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("trails_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_TRAILS),
        None => stored.clamp(1, MAX_TRAILS),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    trails_count: usize,
}

/// Quelle ligne demande à être pré-remplie depuis sa trace.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct PrefillRow {
    prefill_trail: usize,
}

/// Les mesures lues dans la trace de cette ligne, quand l'hôte vient de les demander.
///
/// Lues à ce rendu-là seulement : relire le fichier à chaque ouverture du formulaire coûterait un
/// appel par ligne, et écraserait sans le dire une mesure que l'hôte aurait corrigée à la main.
/// Une trace refusée rend la clé de son message, sous le champ : le formulaire garde ce qui est
/// enregistré, et l'hôte sait pourquoi rien n'a changé. Une trace que la plateforme ne sert pas
/// rend `None`, comme avant.
fn prefill(
    ctx: &HostContext,
    index: usize,
    trail: Option<&TrailRow>,
) -> Option<std::result::Result<crate::gpx::Track, &'static str>> {
    if ctx.input_u64("prefill_trail") != Some(index as u64) {
        return None;
    }
    let reference = trail?.gpx_ref()?;
    let bytes = portaki_sdk::host::files::read(reference).ok()?;
    Some(match crate::gpx::refusal(&bytes) {
        Some(refusal) => Err(refusal),
        None => crate::gpx::read(&String::from_utf8_lossy(&bytes)).ok_or("host.trails.gpx.invalid"),
    })
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn trail_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let trail: Option<&TrailRow> = config.trails.get(index);
    // Les mesures de la trace, si l'hôte vient de cliquer « pré-remplir ». Elles remplacent ce
    // qu'il avait saisi dans ces trois champs, et il peut encore les corriger avant d'enregistrer.
    let prefilled = prefill(ctx, index, trail);
    let measured = prefilled.as_ref().and_then(|read| read.as_ref().ok());
    let gpx_refusal = prefilled.as_ref().and_then(|read| read.as_ref().err());
    let title = trail
        .map(|t| t.title.host_value(ctx))
        .unwrap_or_default()
        .to_string();
    let description = trail
        .map(|t| t.description.host_value(ctx))
        .unwrap_or_default()
        .to_string();
    let address = trail.and_then(|t| t.address.as_deref()).unwrap_or("");
    let id = trail
        .filter(|t| !t.is_blank())
        .map(|t| sdui::row_id("trails", index, Some(&t.id)));

    let mut children: Vec<Component> = id.into_iter().collect();
    children.push(text_field(
        config,
        ctx,
        index,
        "title",
        "i18n:host.trails.rowTitle",
        "i18n:host.trails.rowTitle.placeholder",
        title,
    ));
    children.push(choice_field(
        config,
        ctx,
        index,
        "level",
        "i18n:host.level.label",
        LEVELS,
        trail.and_then(TrailRow::level_key).unwrap_or(""),
    ));
    children.push(FieldHint::new().text("i18n:host.level.hint").into());
    children.push(number_field(
        config,
        ctx,
        index,
        "duration_min",
        "i18n:host.trails.duration",
        MAX_DURATION_MIN,
        trail.and_then(|t| t.duration_min),
    ));
    children.push(number_field(
        config,
        ctx,
        index,
        "distance_km",
        "i18n:host.trails.distance",
        MAX_DISTANCE_KM,
        measured
            .map(|track| round_tenth(track.distance_km))
            .or_else(|| trail.and_then(|t| t.distance_km)),
    ));
    children.push(number_field(
        config,
        ctx,
        index,
        "elevation_m",
        "i18n:host.trails.elevation",
        MAX_ELEVATION_M,
        measured
            .map(|track| track.elevation_m.round())
            .or_else(|| trail.and_then(|t| t.elevation_m)),
    ));
    children.push(choice_field(
        config,
        ctx,
        index,
        "shape",
        "i18n:host.shape.label",
        SHAPES,
        shape_value(trail, measured),
    ));
    if measured.is_some() {
        children.push(
            FieldHint::new()
                .text("i18n:host.trails.prefill.done")
                .into(),
        );
    }

    let mut picker = AddressMapPicker::new()
        .addressName(format!("trails.{index}.address"))
        .latName(format!("trails.{index}.lat"))
        .lngName(format!("trails.{index}.lng"))
        .address(address)
        .label("i18n:host.trails.start")
        .hint("i18n:host.trails.start.hint");
    if let Some((lat, lng)) = trail.and_then(TrailRow::coordinates) {
        picker = picker.lat(lat).lng(lng);
    }
    children.push(
        named(config, ctx, &format!("trails.{index}.lat"))
            .child(picker)
            .into(),
    );
    // Pour la page publique, sur une ligne déjà enregistrée : pas sur un créneau vide, qu'il ferait
    // compter comme rempli.
    if let Some(trail) = trail.filter(|t| !t.is_blank()) {
        // Le départ au logement : la position réelle n'est connue qu'ici, côté hôte, et le
        // formulaire propose la réponse tant que l'hôte n'en a pas enregistré une. Seulement une
        // fois le départ posé : un « non » enregistré avant resterait après.
        if trail.coordinates().is_some() {
            let property = ctx.property.coordinates.map(|point| (point.lat, point.lng));
            let at_property = trail
                .starts_at_property
                .unwrap_or_else(|| trail.near_property(property));
            children.push(
                ToggleRow::new()
                    .name(format!("trails.{index}.starts_at_property"))
                    .label("i18n:host.trails.startsAtProperty")
                    .checked(at_property)
                    .into(),
            );
        }
    }

    children.push(
        named(config, ctx, &format!("trails.{index}.description"))
            .label("i18n:host.trails.description")
            .child(
                TextArea::new()
                    .name(format!("trails.{index}.description"))
                    .value(description)
                    .rows(3)
                    .placeholder("i18n:host.trails.description.placeholder"),
            )
            .into(),
    );
    children.push(text_field(
        config,
        ctx,
        index,
        "link_url",
        "i18n:host.trails.link",
        "i18n:host.trails.link.placeholder",
        trail.map(|t| t.link_url.clone()).unwrap_or_default(),
    ));
    children.push(FieldHint::new().text("i18n:host.trails.link.hint").into());
    let mut gpx_field = Field::new()
        .name(format!("trails.{index}.gpx_file"))
        .label("i18n:host.trails.gpx");
    if let Some(refusal) = gpx_refusal {
        gpx_field = gpx_field.error(crate::i18n::text(refusal).get(&ctx.locale).to_string());
    }
    children.push(
        gpx_field
            .child(
                ImageUpload::new()
                    .name(format!("trails.{index}.gpx_file"))
                    .value(trail.map(|t| t.gpx_file.clone()).unwrap_or_default()),
            )
            .into(),
    );
    children.push(FieldHint::new().text("i18n:host.trails.gpx.hint").into());
    // « Lire les mesures de la trace » : la distance, le dénivelé et le type s'en déduisent, et
    // les recopier d'une application de randonnée à la main invite la faute de frappe (§2.23).
    if trail.and_then(TrailRow::gpx_ref).is_some() {
        children.push(
            Button::new()
                .label("i18n:host.trails.prefill")
                .variant(ButtonVariant::Outline)
                .action(emit_input(PrefillRow {
                    prefill_trail: index,
                }))
                .into(),
        );
    }
    children.push(
        Field::new()
            .name(format!("trails.{index}.photo"))
            .label("i18n:host.trails.photo")
            .child(
                ImageUpload::new()
                    .name(format!("trails.{index}.photo"))
                    .value(trail.map(|t| t.photo.clone()).unwrap_or_default()),
            )
            .into(),
    );

    for (name, label, placeholder) in [
        ("season_from", "i18n:host.trails.season.from", "04-01"),
        ("season_to", "i18n:host.trails.season.to", "10-31"),
    ] {
        let value = trail
            .map(|t| {
                if name == "season_from" {
                    &t.season_from
                } else {
                    &t.season_to
                }
            })
            .cloned()
            .unwrap_or_default();
        children.push(text_field(
            config,
            ctx,
            index,
            name,
            label,
            placeholder,
            value,
        ));
    }
    children.push(FieldHint::new().text("i18n:host.trails.season.hint").into());

    Stack::new()
        .id(format!("trail-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

fn text_field(
    config: &ModuleConfig,
    ctx: &HostContext,
    index: usize,
    name: &str,
    label: &str,
    placeholder: &str,
    value: String,
) -> Component {
    let field = format!("trails.{index}.{name}");
    named(config, ctx, &field)
        .label(label)
        .child(
            TextInput::new()
                .name(field)
                .value(value)
                .placeholder(placeholder),
        )
        .into()
}

/// Une mesure : vide vaut « pas renseigné », et la tuile disparaît chez le voyageur.
fn number_field(
    config: &ModuleConfig,
    ctx: &HostContext,
    index: usize,
    name: &str,
    label: &str,
    max: f64,
    value: Option<f64>,
) -> Component {
    let field = format!("trails.{index}.{name}");
    let mut input = NumberInput::new().name(field.clone()).min(0.0).max(max);
    if let Some(value) = value {
        input = input.value(value);
    }
    named(config, ctx, &field).label(label).child(input).into()
}

/// Une liste figée : le niveau et la forme ne sont pas du texte libre (§2.23).
fn choice_field(
    config: &ModuleConfig,
    ctx: &HostContext,
    index: usize,
    name: &str,
    label: &str,
    values: &[&str],
    chosen: &str,
) -> Component {
    let field = format!("trails.{index}.{name}");
    let options = values
        .iter()
        .map(|value| ChoiceOption::new(*value, format!("{label}.{value}")))
        .collect();
    named(config, ctx, &field)
        .label(label)
        .child(
            Select::new()
                .name(field)
                .options(options)
                .value(chosen.to_string()),
        )
        .into()
}

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    let field = Field::new().name(name);
    let property = ctx.property.coordinates.map(|point| (point.lat, point.lng));
    match config.error_of(name, property) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// Le type de la ligne : celui que l'hôte a choisi, sinon celui que la trace suggère (§9 n° 5).
///
/// Une trace ne distingue pas un aller-retour d'un aller simple : elle ne remplit qu'un champ vide.
fn shape_value(trail: Option<&TrailRow>, measured: Option<&crate::gpx::Track>) -> &'static str {
    trail
        .and_then(TrailRow::shape_key)
        .or_else(|| measured.map(|track| track.shape))
        .unwrap_or("")
}

/// Au dixième de kilomètre : une distance de randonnée au millième dit une précision que ni le
/// GPS ni le pas n'ont.
fn round_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// Le lien de la commune, sous la liste du voyageur.
fn commune_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.commune.title")
        .icon(IconName::Link)
        .child(
            named(config, ctx, "commune_url")
                .label("i18n:host.commune.label")
                .child(
                    TextInput::new()
                        .name("commune_url")
                        .value(config.commune_url.clone())
                        .placeholder("i18n:host.commune.placeholder"),
                ),
        )
        .child(FieldHint::new().text("i18n:host.commune.hint"))
        .into()
}

/// « Page publique » : l'interrupteur du bloc, puis le choix de 2 à 4 itinéraires affichables.
///
/// Le choix reste rendu sans itinéraire à choisir : le formulaire envoie toujours les clés
/// déclarées.
fn public_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let enabled = ctx.input_bool("public_enabled", config.public_enabled);
    let routed = config.routed_trails();
    let choices: Vec<ChoiceOption> = routed
        .iter()
        .map(|(id, trail)| {
            let option = ChoiceOption::new(id.clone(), trail.title.host_value(ctx).to_string());
            match trail.level_key() {
                Some(level) => option.description(crate::format::level(level)),
                None => option,
            }
        })
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
    Card::new()
        .title("i18n:host.public.title")
        .subtitle("i18n:host.public.subtitle")
        .icon(IconName::Home)
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
                .child(Stack::new().children(vec![
                    FieldHint::new().text(hint).into(),
                    ChoiceList::new()
                        .name("public_trails")
                        .multi(true)
                        .limit(PUBLIC_MAX as u32)
                        .choices(choices)
                        .value(serde_json::to_string(&config.public_trails).unwrap_or_default())
                        .into(),
                ]))
                .into(),
        ])
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpx::Track;

    fn track(shape: &'static str) -> Track {
        Track {
            points: Vec::new(),
            distance_km: 3.0,
            elevation_m: 100.0,
            shape,
        }
    }

    #[test]
    fn the_trace_does_not_override_the_host_s_shape() {
        let row = TrailRow {
            shape: "one_way".into(),
            ..TrailRow::default()
        };
        assert_eq!(
            shape_value(Some(&row), Some(&track("round_trip"))),
            "one_way"
        );
    }

    #[test]
    fn the_trace_fills_an_empty_shape_with_a_known_one() {
        let row = TrailRow::default();
        let shape = shape_value(Some(&row), Some(&track("round_trip")));
        assert_eq!(shape, "round_trip");
        assert!(SHAPES.contains(&shape));
        assert_eq!(shape_value(None, None), "");
    }
}
