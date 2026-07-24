//! Host dashboard surface — design `facility-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, Page, Stack, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, FacilityRow, Localized};

const FACILITY_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let facilities = config.parse_facilities();
    let general_note = config.general_note.get(&lang).to_string();

    let submit_args = crate::commands::UpdateConfigArgs {
        facilities: facilities_to_submit(&facilities, &lang),
        facilities_json: String::new(),
        general_note: general_note.clone(),
    };
    let save_action = crate::ids::module_id().command(crate::ids::UPDATE_CONFIG, submit_args);

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..FACILITY_SLOTS {
        cards.push(facility_card(index, facilities.get(index), &lang));
    }
    cards.push(
        Card::new()
            .title("i18n:host.section.note")
            .icon("info-circle")
            .children(vec![Field::new()
                .name("general_note")
                .label("i18n:host.note.label")
                .child(
                    TextArea::new()
                        .name("general_note")
                        .value(general_note)
                        .placeholder("i18n:host.note.placeholder"),
                )
                .into()])
            .into(),
    );
    cards.push(
        Button::new()
            .label("i18n:host.save")
            .tone(Tone::Primary)
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]),
            ),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn facilities_to_submit(
    facilities: &[FacilityRow],
    lang: &str,
) -> Vec<crate::commands::FacilityInput> {
    facilities
        .iter()
        .map(|f| crate::commands::FacilityInput {
            name: f.title.get(lang).to_string(),
            name_fr: String::new(),
            name_en: String::new(),
            hours: f.hours.clone().unwrap_or_default(),
        })
        .collect()
}

fn facility_card(index: usize, facility: Option<&FacilityRow>, lang: &str) -> Component {
    let slot = index + 1;
    let name = facility.map(|f| f.title.get(lang)).unwrap_or("");
    let hours = facility.and_then(|f| f.hours.as_deref()).unwrap_or("");

    Card::new()
        .title(format!("i18n:host.facility.slot{slot}"))
        .icon("clock-circle")
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
