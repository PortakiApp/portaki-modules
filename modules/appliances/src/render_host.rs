//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{ApplianceDevice, AppliancesPayload};
use crate::store;

const DEVICE_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let row = store::load_content().ok().flatten();
    let fr = row
        .as_ref()
        .map(|r| AppliancesPayload::parse(&r.content_fr))
        .unwrap_or_else(default_fr);
    let en = row
        .as_ref()
        .map(|r| AppliancesPayload::parse(&r.content_en))
        .unwrap_or_else(default_en);

    let submit_args = json!({
        "safety_notice_fr": fr.safety_notice,
        "safety_notice_en": en.safety_notice,
        "devices": devices_to_submit(&fr, &en),
    });
    let save_action =
        serde_json::to_value(Action::command("appliances", "saveContent", submit_args))
            .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = vec![
        Field::new()
            .name(json!("safety_notice_fr"))
            .label(json!("i18n:host.safetyFr"))
            .child(
                TextArea::new()
                    .name(json!("safety_notice_fr"))
                    .value(json!(fr.safety_notice)),
            )
            .into(),
        Field::new()
            .name(json!("safety_notice_en"))
            .label(json!("i18n:host.safetyEn"))
            .child(
                TextArea::new()
                    .name(json!("safety_notice_en"))
                    .value(json!(en.safety_notice)),
            )
            .into(),
    ];

    for index in 0..DEVICE_SLOTS {
        push_device_slot(
            &mut form_children,
            index,
            fr.devices.get(index),
            en.devices.get(index),
        );
    }

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

fn default_fr() -> AppliancesPayload {
    AppliancesPayload {
        safety_notice: "Coupez l'eau et le gaz en cas de fuite.".into(),
        devices: vec![ApplianceDevice {
            id: "tv".into(),
            icon: "📺".into(),
            title: "Télévision".into(),
            subtitle: "Salon · Samsung 55\"".into(),
            steps: vec![
                "Allumez avec la télécommande noire.".into(),
                "Source HDMI 1 pour l'Apple TV.".into(),
            ],
            tip: "Télécommande sur le meuble TV.".into(),
            manual_url: String::new(),
        }],
    }
}

fn default_en() -> AppliancesPayload {
    AppliancesPayload {
        safety_notice: "Shut off water and gas if you notice a leak.".into(),
        devices: vec![ApplianceDevice {
            id: "tv".into(),
            icon: "📺".into(),
            title: "Television".into(),
            subtitle: "Living room · Samsung 55\"".into(),
            steps: vec![
                "Use the black remote to power on.".into(),
                "Select HDMI 1 for Apple TV.".into(),
            ],
            tip: "Remote is on the TV stand.".into(),
            manual_url: String::new(),
        }],
    }
}

fn devices_to_submit(fr: &AppliancesPayload, en: &AppliancesPayload) -> Vec<serde_json::Value> {
    let len = fr.devices.len().max(en.devices.len());
    (0..len)
        .map(|i| {
            let fr_d = fr.devices.get(i);
            let en_d = en.devices.get(i);
            json!({
                "name_fr": fr_d.map(|d| d.title.as_str()).unwrap_or(""),
                "name_en": en_d.map(|d| d.title.as_str()).unwrap_or(""),
                "subtitle_fr": fr_d.map(|d| d.subtitle.as_str()).unwrap_or(""),
                "subtitle_en": en_d.map(|d| d.subtitle.as_str()).unwrap_or(""),
                "steps_fr": fr_d.map(|d| d.steps.join("\n")).unwrap_or_default(),
                "steps_en": en_d.map(|d| d.steps.join("\n")).unwrap_or_default(),
                "tip_fr": fr_d.map(|d| d.tip.as_str()).unwrap_or(""),
                "tip_en": en_d.map(|d| d.tip.as_str()).unwrap_or(""),
            })
        })
        .collect()
}

fn push_device_slot(
    children: &mut Vec<Component>,
    index: usize,
    fr: Option<&ApplianceDevice>,
    en: Option<&ApplianceDevice>,
) {
    let slot = index + 1;
    children.push(
        Text::new()
            .text(json!(format!("i18n:host.device.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.name_fr")))
            .label(json!("i18n:host.device.nameFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.name_fr")))
                    .value(json!(fr.map(|d| d.title.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.name_en")))
            .label(json!("i18n:host.device.nameEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.name_en")))
                    .value(json!(en.map(|d| d.title.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.subtitle_fr")))
            .label(json!("i18n:host.device.subtitleFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.subtitle_fr")))
                    .value(json!(fr.map(|d| d.subtitle.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.subtitle_en")))
            .label(json!("i18n:host.device.subtitleEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.subtitle_en")))
                    .value(json!(en.map(|d| d.subtitle.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.steps_fr")))
            .label(json!("i18n:host.device.stepsFr"))
            .child(
                TextArea::new()
                    .name(json!(format!("devices.{index}.steps_fr")))
                    .value(json!(fr.map(|d| d.steps.join("\n")).unwrap_or_default())),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.steps_en")))
            .label(json!("i18n:host.device.stepsEn"))
            .child(
                TextArea::new()
                    .name(json!(format!("devices.{index}.steps_en")))
                    .value(json!(en.map(|d| d.steps.join("\n")).unwrap_or_default())),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.tip_fr")))
            .label(json!("i18n:host.device.tipFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.tip_fr")))
                    .value(json!(fr.map(|d| d.tip.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("devices.{index}.tip_en")))
            .label(json!("i18n:host.device.tipEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("devices.{index}.tip_en")))
                    .value(json!(en.map(|d| d.tip.as_str()).unwrap_or(""))),
            )
            .into(),
    );
}
