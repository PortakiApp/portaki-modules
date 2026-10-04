//! Host dashboard surface — l'onglet « Randonnées » de l'espace du logement.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, FieldHint, Form, ImageUpload, NumberInput, Page, Select, Stack,
    StepList, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::config::{ModuleConfig, TrailRow, LEVELS, MAX_TRAILS, SHAPES};

/// Les bornes d'une mesure de randonnée : au-delà, c'est une faute de frappe.
const MAX_DURATION_MIN: f64 = 1_440.0;
const MAX_DISTANCE_KM: f64 = 200.0;
const MAX_ELEVATION_M: f64 = 5_000.0;

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
                    commune_card(&config),
                ]))),
    )
    .with_id(MAIN))
}

/// Les itinéraires, en lignes dynamiques bornées — le motif de `rules` et `ical-sync`.
fn trails_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.parse_trails().len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| trail_row(index, config.trails.get(index), ctx))
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

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn trail_row(index: usize, trail: Option<&TrailRow>, ctx: &HostContext) -> Component {
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
        index,
        "title",
        "i18n:host.trails.rowTitle",
        "i18n:host.trails.rowTitle.placeholder",
        title,
    ));
    children.push(choice_field(
        index,
        "level",
        "i18n:host.level.label",
        LEVELS,
        trail.and_then(TrailRow::level_key).unwrap_or(""),
    ));
    children.push(FieldHint::new().text("i18n:host.level.hint").into());
    children.push(number_field(
        index,
        "duration_min",
        "i18n:host.trails.duration",
        MAX_DURATION_MIN,
        trail.and_then(|t| t.duration_min),
    ));
    children.push(number_field(
        index,
        "distance_km",
        "i18n:host.trails.distance",
        MAX_DISTANCE_KM,
        trail.and_then(|t| t.distance_km),
    ));
    children.push(number_field(
        index,
        "elevation_m",
        "i18n:host.trails.elevation",
        MAX_ELEVATION_M,
        trail.and_then(|t| t.elevation_m),
    ));
    children.push(choice_field(
        index,
        "shape",
        "i18n:host.shape.label",
        SHAPES,
        trail.and_then(TrailRow::shape_key).unwrap_or(""),
    ));

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
    children.push(picker.into());

    children.push(
        Field::new()
            .name(format!("trails.{index}.description"))
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
        index,
        "link_url",
        "i18n:host.trails.link",
        "i18n:host.trails.link.placeholder",
        trail.map(|t| t.link_url.clone()).unwrap_or_default(),
    ));
    children.push(FieldHint::new().text("i18n:host.trails.link.hint").into());
    children.push(
        Field::new()
            .name(format!("trails.{index}.gpx_file"))
            .label("i18n:host.trails.gpx")
            .child(
                ImageUpload::new()
                    .name(format!("trails.{index}.gpx_file"))
                    .value(trail.map(|t| t.gpx_file.clone()).unwrap_or_default()),
            )
            .into(),
    );
    children.push(FieldHint::new().text("i18n:host.trails.gpx.hint").into());

    Stack::new()
        .id(format!("trail-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

fn text_field(
    index: usize,
    name: &str,
    label: &str,
    placeholder: &str,
    value: String,
) -> Component {
    let field = format!("trails.{index}.{name}");
    Field::new()
        .name(field.clone())
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
fn number_field(index: usize, name: &str, label: &str, max: f64, value: Option<f64>) -> Component {
    let field = format!("trails.{index}.{name}");
    let mut input = NumberInput::new().name(field.clone()).min(0.0).max(max);
    if let Some(value) = value {
        input = input.value(value);
    }
    Field::new().name(field).label(label).child(input).into()
}

/// Une liste figée : le niveau et la forme ne sont pas du texte libre (§2.23).
fn choice_field(index: usize, name: &str, label: &str, values: &[&str], chosen: &str) -> Component {
    let field = format!("trails.{index}.{name}");
    let options = values
        .iter()
        .map(|value| ChoiceOption::new(*value, format!("{label}.{value}")))
        .collect();
    Field::new()
        .name(field.clone())
        .label(label)
        .child(
            Select::new()
                .name(field)
                .options(options)
                .value(chosen.to_string()),
        )
        .into()
}

/// Le lien de la commune, sous la liste du voyageur.
fn commune_card(config: &ModuleConfig) -> Component {
    Card::new()
        .title("i18n:host.commune.title")
        .icon(IconName::Link)
        .child(
            Field::new()
                .name("commune_url")
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
