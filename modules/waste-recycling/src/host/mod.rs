//! Host dashboard surface — design `waste-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Eyebrow, Field, FieldHint, Form, Page, Select, Stack, StepList, Text,
    TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::config::{bin_color_name, BinRow, DropoffRow, ModuleConfig, MAX_DROPOFF_POINTS};

/// Combien de bacs le formulaire accepte.
///
/// Une capacité, pas un nombre de lignes dessinées : six emplacements figés gelaient la liste à
/// six — l'hôte ne pouvait pas en saisir un septième parce que le formulaire ne le dessinait
/// jamais, et voyait quatre cartes vides quand il en avait saisi deux.
pub const MAX_BINS: usize = 12;

/// Host configuration page — bin cards + collection schedule.
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

    let mut cards: Vec<Component> = vec![bins_card(&config, &ctx)];
    cards.push(schedule_card(&config, &ctx));
    cards.push(dropoff_card(&config, &ctx));
    cards.push(compost_card(&config, &ctx));

    // No Save button — the modules drawer owns the footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(MAIN))
}

/// Les jours de collecte, la consigne de sortie et la phrase libre.
///
/// Les sept cases viennent **à côté** de la phrase, pas à sa place : un hôte qui les coche gagne le
/// jour calculé et les cas de départ, un hôte qui n'y touche pas garde le bandeau qu'il avait.
fn schedule_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let days = [
        ("collects_mon", "mon", config.collects_mon),
        ("collects_tue", "tue", config.collects_tue),
        ("collects_wed", "wed", config.collects_wed),
        ("collects_thu", "thu", config.collects_thu),
        ("collects_fri", "fri", config.collects_fri),
        ("collects_sat", "sat", config.collects_sat),
        ("collects_sun", "sun", config.collects_sun),
    ];

    // Un champ par jour, nommé comme son `#[field]` : la plateforme refuse tout le formulaire dès
    // qu'un nom n'est pas une clé déclarée, et un groupe de cases n'a pas de nom à lui.
    let mut children: Vec<Component> = vec![
        Eyebrow::new().text("i18n:host.collection.days").into(),
        Text::new()
            .text("i18n:host.collection.days.desc")
            .variant(TextVariant::Caption)
            .into(),
    ];
    children.extend(days.into_iter().map(|(name, key, ticked)| {
        Field::new()
            .name(name)
            .label(format!("i18n:host.collection.day.{key}"))
            .child(Toggle::new().name(name).checked(ticked))
            .into()
    }));

    children.push(
        Field::new()
            .name("collection_schedule")
            .label("i18n:host.schedule.label")
            .child(
                TextArea::new()
                    .name("collection_schedule")
                    .value(config.collection_schedule.host_value(ctx))
                    .placeholder("i18n:host.schedule.placeholder"),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name("takeout_note")
            .label("i18n:host.takeout.label")
            .children(vec![
                FieldHint::new().text("i18n:host.takeout.desc").into(),
                TextArea::new()
                    .name("takeout_note")
                    .value(config.takeout_note.host_value(ctx))
                    .placeholder("i18n:host.takeout.placeholder")
                    .into(),
            ])
            .into(),
    );

    // Le chemin jusqu'au local, une étape par ligne : il appartient à la collecte plus qu'aux
    // bacs — c'est le trajet du jour de sortie.
    children.push(
        Field::new()
            .name("bin_room_steps")
            .label("i18n:host.binRoom.label")
            .children(vec![
                FieldHint::new().text("i18n:host.binRoom.hint").into(),
                TextArea::new()
                    .name("bin_room_steps")
                    .value(config.bin_room_steps.host_value(ctx))
                    .rows(3)
                    .placeholder("i18n:host.binRoom.placeholder")
                    .into(),
            ])
            .into(),
    );

    Card::new()
        .title("i18n:host.section.schedule")
        .icon(IconName::Calendar)
        .children(children)
        .into()
}

/// Les points d'apport : des lignes dessinées à la demande, pas dix emplacements vides.
///
/// Le motif est celui de `rules` et d'`ical-sync` — un `StepList` dont « Ajouter » émet le nombre
/// de lignes voulu, relu ici dans la borne du module. Il passera par l'aide du SDK quand elle sera
/// publiée ; en attendant les quatre lignes vivent ici plutôt que de figer la liste à six.
fn dropoff_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let stored = config.parse_dropoff_points().len();
    let rows_count = draft_rows(ctx, "dropoff_points", stored, MAX_DROPOFF_POINTS);
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| dropoff_row(index, config.dropoff_points.get(index), ctx))
        .collect();

    Card::new()
        .title("i18n:host.dropoff.title")
        .subtitle("i18n:host.dropoff.subtitle")
        .icon(IconName::MapPin)
        .child(
            StepList::new()
                .addLabel("i18n:host.dropoff.add")
                .removeLabel("i18n:host.dropoff.remove")
                .emptyTitle("i18n:host.dropoff.emptyTitle")
                .emptyDescription("i18n:host.dropoff.emptyDescription")
                .itemKeyPrefix("dropoff_points")
                .addAction(emit_input(RowCount {
                    dropoff_points_count: (rows_count + 1).min(MAX_DROPOFF_POINTS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
///
/// La borne est un paramètre depuis que deux listes l'appellent : elle était celle des points
/// d'apport pour tout le monde, ce qui aurait plafonné les bacs à dix sans que rien ne le dise.
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

fn dropoff_row(index: usize, point: Option<&DropoffRow>, ctx: &HostContext) -> Component {
    let title = point.map(|p| p.title.host_value(ctx)).unwrap_or_default();
    let note = point.map(|p| p.note.host_value(ctx)).unwrap_or_default();
    let address = point.and_then(|p| p.address.as_deref()).unwrap_or("");
    let id = point
        .filter(|p| !p.is_blank())
        .map(|p| sdui::row_id("dropoff_points", index, Some(&p.id)));

    let mut picker = AddressMapPicker::new()
        .addressName(format!("dropoff_points.{index}.address"))
        .latName(format!("dropoff_points.{index}.lat"))
        .lngName(format!("dropoff_points.{index}.lng"))
        .address(address)
        .label("i18n:host.dropoff.position");
    if let Some((lat, lng)) = point.and_then(DropoffRow::coordinates) {
        picker = picker.lat(lat).lng(lng);
    }

    let mut children: Vec<Component> = id.into_iter().collect();
    children.push(
        Field::new()
            .name(format!("dropoff_points.{index}.title"))
            .label("i18n:host.dropoff.pointTitle")
            .child(
                TextInput::new()
                    .name(format!("dropoff_points.{index}.title"))
                    .value(title)
                    .placeholder("i18n:host.dropoff.pointTitle.placeholder"),
            )
            .into(),
    );
    children.push(picker.into());
    children.push(Eyebrow::new().text("i18n:host.dropoff.accepts").into());
    for (suffix, ticked) in [
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
    ] {
        let name = format!("dropoff_points.{index}.{suffix}");
        children.push(
            Field::new()
                .name(name.clone())
                .label(format!("i18n:host.dropoff.{suffix}"))
                .child(Toggle::new().name(name).checked(ticked))
                .into(),
        );
    }
    children.push(
        Field::new()
            .name(format!("dropoff_points.{index}.note"))
            .label("i18n:host.dropoff.note")
            .child(
                TextInput::new()
                    .name(format!("dropoff_points.{index}.note"))
                    .value(note)
                    .placeholder("i18n:host.dropoff.note.placeholder"),
            )
            .into(),
    );

    Stack::new()
        .id(format!("dropoff-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

/// Le composteur : une case, un emplacement, et deux listes d'une ligne par élément.
fn compost_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![
        Field::new()
            .name("compost_enabled")
            .label("i18n:host.compost.enabled")
            .child(
                Toggle::new()
                    .name("compost_enabled")
                    .checked(config.compost_enabled),
            )
            .into(),
        Field::new()
            .name("compost_location")
            .label("i18n:host.compost.location")
            .child(
                TextInput::new()
                    .name("compost_location")
                    .value(config.compost_location.host_value(ctx))
                    .placeholder("i18n:host.compost.location.placeholder"),
            )
            .into(),
        FieldHint::new()
            .text("i18n:host.compost.location.hint")
            .into(),
    ];
    for (name, label) in [
        ("compost_accepted", "i18n:host.compost.accepted"),
        ("compost_refused", "i18n:host.compost.refused"),
    ] {
        let value = if name == "compost_accepted" {
            config.compost_accepted.host_value(ctx)
        } else {
            config.compost_refused.host_value(ctx)
        };
        children.push(
            Field::new()
                .name(name)
                .label(label)
                .child(
                    TextArea::new()
                        .name(name)
                        .value(value)
                        .placeholder("i18n:host.compost.lines.placeholder"),
                )
                .into(),
        );
    }

    Card::new()
        .title("i18n:host.compost.title")
        .subtitle("i18n:host.compost.subtitle")
        .icon(IconName::Recycle)
        .children(children)
        .into()
}

/// Les bacs du logement, en lignes dynamiques bornées.
fn bins_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, "bins", config.bins.len(), MAX_BINS);
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| bin_row(index, config.bins.get(index), ctx))
        .collect();

    Card::new()
        .title("i18n:host.bins.title")
        .subtitle("i18n:host.bins.subtitle")
        .icon(IconName::Recycle)
        .child(
            StepList::new()
                .addLabel("i18n:host.bins.add")
                .removeLabel("i18n:host.bins.remove")
                .emptyTitle("i18n:host.bins.emptyTitle")
                .emptyDescription("i18n:host.bins.emptyDescription")
                .itemKeyPrefix("bins")
                .addAction(emit_input(BinRowCount {
                    bins_count: (rows_count + 1).min(MAX_BINS),
                }))
                .children(rows),
        )
        .into()
}

fn bin_row(index: usize, bin: Option<&BinRow>, ctx: &HostContext) -> Component {
    let title = bin.map(|b| b.title.host_value(ctx)).unwrap_or_default();
    let items = bin.map(|b| b.items.host_value(ctx)).unwrap_or_default();
    let color = bin_color_name(bin.and_then(|b| b.color.as_deref())).unwrap_or("");
    let location = bin.map(|b| b.location.host_value(ctx)).unwrap_or_default();
    // A filled row sends its id, so a save merges into it (and keeps its other languages). A
    // blank slot has nothing to keep — and an id would make it count as filled.
    let id = bin
        .filter(|b| !b.is_blank())
        .map(|b| sdui::row_id("bins", index, Some(&b.id)));

    Stack::new()
        .id(format!("bin-{index}"))
        .gap(10.0)
        .children(
            id.into_iter()
                .chain([
                    Field::new()
                        .name(format!("bins.{index}.title"))
                        .label("i18n:host.bin.title")
                        .child(
                            TextInput::new()
                                .name(format!("bins.{index}.title"))
                                .value(title),
                        )
                        .into(),
                    // One item per line.
                    Field::new()
                        .name(format!("bins.{index}.items"))
                        .label("i18n:host.bin.items")
                        .child(
                            TextArea::new()
                                .name(format!("bins.{index}.items"))
                                .value(items)
                                .placeholder("i18n:host.bin.items.placeholder"),
                        )
                        .into(),
                    // Où le bac se trouve : le voyageur cherche la poubelle avant de chercher
                    // comment trier.
                    Field::new()
                        .name(format!("bins.{index}.location"))
                        .label("i18n:host.bin.location")
                        .child(
                            TextInput::new()
                                .name(format!("bins.{index}.location"))
                                .value(location)
                                .placeholder("i18n:host.bin.location.placeholder"),
                        )
                        .into(),
                    Field::new()
                        .name(format!("bins.{index}.color"))
                        .label("i18n:host.bin.color")
                        .child(
                            Select::new()
                                .name(format!("bins.{index}.color"))
                                .options(vec![
                                    ChoiceOption::new("", "i18n:host.bin.color.none"),
                                    ChoiceOption::new("yellow", "i18n:host.bin.color.yellow"),
                                    ChoiceOption::new("green", "i18n:host.bin.color.green"),
                                    ChoiceOption::new("brown", "i18n:host.bin.color.brown"),
                                    ChoiceOption::new("grey", "i18n:host.bin.color.grey"),
                                ])
                                .value(color),
                        )
                        .into(),
                ])
                .collect(),
        )
        .into()
}
