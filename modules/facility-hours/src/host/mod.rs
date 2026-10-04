//! Host dashboard surface — design `facility-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, Page, Select, Stack, StepList, Text, TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{FacilityRow, ModuleConfig, ICONS};

/// Combien d'équipements le formulaire accepte.
///
/// Une capacité, pas un nombre de lignes dessinées : six emplacements figés gelaient la liste à
/// six — l'hôte ne pouvait pas en saisir un septième parce que le formulaire ne le dessinait
/// jamais, et voyait quatre cartes vides quand il en avait saisi deux.
pub const MAX_FACILITIES: usize = 12;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::FacilityEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::ClockCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    let mut cards: Vec<Component> = vec![facilities_card(&config, &ctx)];
    cards.push(
        Card::new()
            .title("i18n:host.section.note")
            .icon(IconName::InfoCircle)
            .children(vec![Field::new()
                .name("general_note")
                .label("i18n:host.note.label")
                .child(
                    TextArea::new()
                        .name("general_note")
                        .value(config.general_note.host_value(&ctx))
                        .placeholder("i18n:host.note.placeholder"),
                )
                .into()])
            .into(),
    );

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

/// Les équipements, en lignes dynamiques bornées.
fn facilities_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.facilities.len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| facility_row(index, config.facilities.get(index), ctx))
        .collect();

    Card::new()
        .title("i18n:host.facilities.title")
        .subtitle("i18n:host.facilities.subtitle")
        .icon(IconName::ClockCircle)
        .child(
            StepList::new()
                .addLabel("i18n:host.facilities.add")
                .removeLabel("i18n:host.facilities.remove")
                .emptyTitle("i18n:host.facilities.emptyTitle")
                .emptyDescription("i18n:host.facilities.emptyDescription")
                .itemKeyPrefix("facilities")
                .addAction(emit_input(RowCount {
                    facilities_count: (rows_count + 1).min(MAX_FACILITIES),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("facilities_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_FACILITIES),
        None => stored.clamp(1, MAX_FACILITIES),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    facilities_count: usize,
}

fn emit_input(payload: impl serde::Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn facility_row(index: usize, facility: Option<&FacilityRow>, ctx: &HostContext) -> Component {
    let title = facility
        .map(|f| f.title.host_value(ctx))
        .unwrap_or_default();
    let hours = facility.and_then(|f| f.hours.as_deref()).unwrap_or("");
    let group = facility.and_then(FacilityRow::group_label).unwrap_or("");
    let season_from = facility.map(|f| f.season_from.as_str()).unwrap_or("");
    let season_to = facility.map(|f| f.season_to.as_str()).unwrap_or("");
    let icon = facility.and_then(FacilityRow::icon_name).unwrap_or("");
    let opens_at = facility.map(|f| f.opens_at.as_str()).unwrap_or("");
    let closes_at = facility.map(|f| f.closes_at.as_str()).unwrap_or("");
    let all_day = facility.is_some_and(|f| f.all_day);
    let lines = facility
        .map(|f| f.lines.host_value(ctx))
        .unwrap_or_default();
    let note = facility.map(|f| f.note.host_value(ctx)).unwrap_or_default();
    // A filled row sends its id, so a save merges into it (and keeps its other languages). A blank slot has nothing to keep — and an id would make it count as filled.
    let id = facility
        .filter(|f| !f.is_blank())
        .map(|f| sdui::row_id("facilities", index, Some(&f.id)));

    Stack::new()
        .id(format!("facility-{index}"))
        .gap(10.0)
        .children(
            id.into_iter()
                .chain([
                    Field::new()
                        .name(format!("facilities.{index}.title"))
                        .label("i18n:host.facility.name")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.title"))
                                .value(title),
                        )
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.group"))
                        .label("i18n:host.facility.group")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.group"))
                                .value(group)
                                .placeholder("i18n:host.facility.group.placeholder"),
                        )
                        .into(),
                    FieldHint::new()
                        .text("i18n:host.facility.group.hint")
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.icon"))
                        .label("i18n:host.facility.icon")
                        .child(
                            Select::new()
                                .name(format!("facilities.{index}.icon"))
                                .options(
                                    std::iter::once(ChoiceOption::new(
                                        "",
                                        "i18n:host.facility.icon.none",
                                    ))
                                    .chain(ICONS.iter().map(|icon| {
                                        ChoiceOption::new(*icon, format!("i18n:host.icon.{icon}"))
                                    }))
                                    .collect(),
                                )
                                .value(icon.to_string()),
                        )
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.season_from"))
                        .label("i18n:host.facility.season.from")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.season_from"))
                                .value(season_from)
                                .placeholder("04-01"),
                        )
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.season_to"))
                        .label("i18n:host.facility.season.to")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.season_to"))
                                .value(season_to)
                                .placeholder("10-31"),
                        )
                        .into(),
                    FieldHint::new()
                        .text("i18n:host.facility.season.hint")
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.hours"))
                        .label("i18n:host.facility.hours")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.hours"))
                                .value(hours)
                                .placeholder("i18n:host.facility.hours.placeholder"),
                        )
                        .into(),
                    // Les heures structurées, à côté de la phrase et non à sa place : un hôte qui
                    // les remplit gagne l'état en direct, un hôte qui ne les remplit pas garde
                    // exactement ce qu'il avait écrit.
                    Field::new()
                        .name(format!("facilities.{index}.opens_at"))
                        .label("i18n:host.facility.opensAt")
                        .children(vec![
                            FieldHint::new()
                                .text("i18n:host.facility.opensAt.desc")
                                .into(),
                            TextInput::new()
                                .name(format!("facilities.{index}.opens_at"))
                                .value(opens_at)
                                .placeholder("08:00")
                                .into(),
                        ])
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.closes_at"))
                        .label("i18n:host.facility.closesAt")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.closes_at"))
                                .value(closes_at)
                                .placeholder("20:00"),
                        )
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.all_day"))
                        .label("i18n:host.facility.allDay")
                        .child(
                            Toggle::new()
                                .name(format!("facilities.{index}.all_day"))
                                .checked(all_day),
                        )
                        .into(),
                    // One schedule per line.
                    Field::new()
                        .name(format!("facilities.{index}.lines"))
                        .label("i18n:host.facility.lines")
                        .child(
                            TextArea::new()
                                .name(format!("facilities.{index}.lines"))
                                .value(lines)
                                .placeholder("i18n:host.facility.lines.placeholder"),
                        )
                        .into(),
                    Field::new()
                        .name(format!("facilities.{index}.note"))
                        .label("i18n:host.facility.note")
                        .child(
                            TextInput::new()
                                .name(format!("facilities.{index}.note"))
                                .value(note),
                        )
                        .into(),
                ])
                .collect(),
        )
        .into()
}
