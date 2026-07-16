//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::{load_config, FacilityRow};

const FACILITY_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let config = load_config().unwrap_or_default();
    let facilities = config.parse_facilities();

    let submit_args = json!({
        "facilities": facilities_to_submit(&facilities),
        "general_note": config.general_note,
    });
    let save_action = serde_json::to_value(Action::command(
        "facility-hours",
        "updateConfig",
        submit_args,
    ))
    .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();
    for index in 0..FACILITY_SLOTS {
        push_facility_slot(&mut form_children, index, facilities.get(index));
    }
    form_children.push(
        Field::new()
            .name(json!("general_note"))
            .label(json!("i18n:host.note.label"))
            .child(
                TextArea::new()
                    .name(json!("general_note"))
                    .value(json!(config.general_note))
                    .placeholder(json!("i18n:host.note.placeholder")),
            )
            .into(),
    );
    form_children.push(
        Text::new()
            .text(json!("i18n:host.main.help"))
            .variant(json!("caption"))
            .into(),
    );
    form_children.push(
        Button::new()
            .label(json!("i18n:host.save"))
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new()
            .title(json!("i18n:surface.host.main.title"))
            .child(
                Text::new()
                    .text(json!("i18n:surface.host.main.subtitle"))
                    .variant(json!("body")),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id("main")
}

fn facilities_to_submit(facilities: &[FacilityRow]) -> Vec<serde_json::Value> {
    facilities
        .iter()
        .map(|f| {
            json!({
                "name_fr": f.title.fr,
                "name_en": f.title.en,
                "hours": f.hours.clone().unwrap_or_default(),
            })
        })
        .collect()
}

fn push_facility_slot(children: &mut Vec<Component>, index: usize, facility: Option<&FacilityRow>) {
    let slot = index + 1;
    let name_fr = facility.map(|f| f.title.fr.as_str()).unwrap_or("");
    let name_en = facility.map(|f| f.title.en.as_str()).unwrap_or("");
    let hours = facility.and_then(|f| f.hours.as_deref()).unwrap_or("");

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.facility.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("facilities.{index}.name_fr")))
            .label(json!("i18n:host.facility.nameFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("facilities.{index}.name_fr")))
                    .value(json!(name_fr)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("facilities.{index}.name_en")))
            .label(json!("i18n:host.facility.nameEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("facilities.{index}.name_en")))
                    .value(json!(name_en)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("facilities.{index}.hours")))
            .label(json!("i18n:host.facility.hours"))
            .child(
                TextInput::new()
                    .name(json!(format!("facilities.{index}.hours")))
                    .value(json!(hours))
                    .placeholder(json!("i18n:host.facility.hours.placeholder")),
            )
            .into(),
    );
}
