//! Host dashboard surface — design `facility-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, Page, Stack, Text, TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{FacilityRow, ModuleConfig};

const FACILITY_SLOTS: usize = 6;

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

    // The stored rows where they are, blank ones included, then empty slots.
    let mut cards: Vec<Component> = Vec::new();
    for index in 0..FACILITY_SLOTS.max(config.facilities.len()) {
        cards.push(facility_card(index, config.facilities.get(index), &ctx));
    }
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

fn facility_card(index: usize, facility: Option<&FacilityRow>, ctx: &HostContext) -> Component {
    let slot = index + 1;
    let title = facility
        .map(|f| f.title.host_value(ctx))
        .unwrap_or_default();
    let hours = facility.and_then(|f| f.hours.as_deref()).unwrap_or("");
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

    Card::new()
        .title(format!("i18n:host.facility.slot{slot}"))
        .icon(IconName::ClockCircle)
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
