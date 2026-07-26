//! Host dashboard surfaces — config cards embedded in the module sheet.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Select, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, EventRow, Localized};
use crate::nearby::has_open_agenda;

const EVENT_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let events = config.parse_events();
    let disclaimer = config.disclaimer.get(&lang).to_string();
    let open_agenda = has_open_agenda(&ctx);

    let mut form_children: Vec<Component> = Vec::new();
    form_children.push(nearby_card(&config, open_agenda));
    for index in 0..EVENT_SLOTS {
        form_children.push(event_slot_card(index, events.get(index), &lang));
    }
    form_children.push(
        Card::new()
            .title("i18n:host.section.disclaimer")
            .subtitle("i18n:host.section.disclaimer.help")
            .icon("info-circle")
            .children(vec![Field::new()
                .name("disclaimer")
                .label("i18n:host.disclaimer.label")
                .child(
                    TextArea::new()
                        .name("disclaimer")
                        .value(disclaimer)
                        .placeholder("i18n:host.disclaimer.placeholder"),
                )
                .into()])
            .into(),
    );
    form_children.push(
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Surface::new(Page::new().child(Form::new().children(form_children)))
        .with_id(crate::ids::HOST_MAIN)
}

fn nearby_card(config: &crate::config::ModuleConfig, open_agenda: bool) -> Component {
    let status_key = if !config.nearby_enabled {
        "i18n:host.nearby.status.off"
    } else if open_agenda {
        "i18n:host.nearby.status.ready"
    } else {
        "i18n:host.nearby.status.missingKey"
    };
    let nearby_value = if config.nearby_enabled {
        "true"
    } else {
        "false"
    };
    let radius_value = config.normalized_radius_km().to_string();

    Card::new()
        .title("i18n:host.section.nearby")
        .subtitle("i18n:host.section.nearby.help")
        .icon("map-pin")
        .children(vec![
            Field::new()
                .name("nearby_enabled")
                .label("i18n:host.nearby.enabled")
                .child(
                    Select::new()
                        .name("nearby_enabled")
                        .options(vec![
                            ChoiceOption::new("true", "i18n:host.nearby.enabled.on"),
                            ChoiceOption::new("false", "i18n:host.nearby.enabled.off"),
                        ])
                        .value(nearby_value),
                )
                .into(),
            Field::new()
                .name("radius_km")
                .label("i18n:host.nearby.radius")
                .child(
                    Select::new()
                        .name("radius_km")
                        .options(vec![
                            ChoiceOption::new("10", "i18n:host.nearby.radius.10"),
                            ChoiceOption::new("20", "i18n:host.nearby.radius.20"),
                            ChoiceOption::new("40", "i18n:host.nearby.radius.40"),
                            ChoiceOption::new("60", "i18n:host.nearby.radius.60"),
                            ChoiceOption::new("100", "i18n:host.nearby.radius.100"),
                        ])
                        .value(radius_value),
                )
                .into(),
            Text::new()
                .text(status_key)
                .variant(TextVariant::Caption)
                .into(),
        ])
        .into()
}

fn event_slot_card(index: usize, event: Option<&EventRow>, lang: &str) -> Component {
    let slot = index + 1;
    let title = event.map(|e| e.title.get(lang)).unwrap_or("");
    let place = event.map(|e| e.place.get(lang)).unwrap_or("");
    let starts_at = event.map(|e| e.starts_at.as_str()).unwrap_or("");
    let url = event.and_then(|e| e.url.as_deref()).unwrap_or("");
    let lat = event
        .and_then(|e| e.lat)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let lng = event
        .and_then(|e| e.lng)
        .map(|v| v.to_string())
        .unwrap_or_default();

    Card::new()
        .title(format!("i18n:host.event.slot{slot}"))
        .icon("calendar")
        .children(vec![
            Field::new()
                .name(format!("events.{index}.title"))
                .label("i18n:host.event.title")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.title"))
                        .value(title),
                )
                .into(),
            Field::new()
                .name(format!("events.{index}.place"))
                .label("i18n:host.event.place")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.place"))
                        .value(place),
                )
                .into(),
            Field::new()
                .name(format!("events.{index}.starts_at"))
                .label("i18n:host.event.startsAt")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.starts_at"))
                        .value(starts_at)
                        .placeholder("i18n:host.event.startsAt.placeholder"),
                )
                .into(),
            Field::new()
                .name(format!("events.{index}.url"))
                .label("i18n:host.event.url")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.url"))
                        .value(url),
                )
                .into(),
            Field::new()
                .name(format!("events.{index}.lat"))
                .label("i18n:host.event.lat")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.lat"))
                        .value(lat),
                )
                .into(),
            Field::new()
                .name(format!("events.{index}.lng"))
                .label("i18n:host.event.lng")
                .child(
                    TextInput::new()
                        .name(format!("events.{index}.lng"))
                        .value(lng),
                )
                .into(),
        ])
        .into()
}
