//! Host dashboard surface — design `facility-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Stack, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{FacilityRow, Localized, ModuleConfig};

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
    let lang = Localized::lang_code(&ctx.locale);
    let config = ModuleConfig::read(&ctx)?;
    let facilities = config.parse_facilities();

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..FACILITY_SLOTS {
        cards.push(facility_card(index, facilities.get(index), &lang));
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
                        .value(config.general_note.pick(&lang))
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
    .with_id(crate::ids::HOST_MAIN))
}

fn facility_card(index: usize, facility: Option<&FacilityRow>, lang: &str) -> Component {
    let slot = index + 1;
    let name = facility.map(|f| f.title.pick(lang)).unwrap_or_default();
    let hours = facility.and_then(|f| f.hours.as_deref()).unwrap_or("");

    Card::new()
        .title(format!("i18n:host.facility.slot{slot}"))
        .icon(IconName::ClockCircle)
        .children(vec![
            Field::new()
                .name(format!("facilities.{index}.name"))
                .label("i18n:host.facility.name")
                .child(
                    TextInput::new()
                        .name(format!("facilities.{index}.name"))
                        .value(name),
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
        ])
        .into()
}
