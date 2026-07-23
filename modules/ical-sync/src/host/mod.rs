//! Host dashboard surface — config cards in the module sheet.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, InfoBanner, Page, Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();

    let save_action = crate::ids::module_id().command(
        crate::ids::UPDATE_CONFIG,
        crate::commands::UpdateConfigArgs {
            ical_url_primary: config.ical_url_primary.clone(),
            ical_url_secondary: config.ical_url_secondary.clone(),
        },
    );

    let last_sync = config
        .last_sync_at
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:host.status.never".to_string());
    let summary = config
        .sync_summary
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:host.status.emptySummary".to_string());

    let form_children: Vec<Component> = vec![
        InfoBanner::new()
            .title("i18n:host.alert.title")
            .message("i18n:host.alert.message")
            .into(),
        Card::new()
            .title("i18n:host.section.feeds")
            .subtitle("i18n:host.section.feeds.help")
            .icon("calendar")
            .children(vec![
                Field::new()
                    .name("ical_url_primary")
                    .label("i18n:host.primary.label")
                    .child(
                        TextInput::new()
                            .name("ical_url_primary")
                            .value(config.ical_url_primary.clone())
                            .placeholder("i18n:host.primary.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("ical_url_secondary")
                    .label("i18n:host.secondary.label")
                    .child(
                        TextInput::new()
                            .name("ical_url_secondary")
                            .value(config.ical_url_secondary.clone())
                            .placeholder("i18n:host.secondary.placeholder"),
                    )
                    .into(),
            ])
            .into(),
        Card::new()
            .title("i18n:host.section.status")
            .subtitle("i18n:host.section.status.help")
            .icon("refresh")
            .children(vec![
                Field::new()
                    .name("last_sync_at")
                    .label("i18n:host.status.lastSync")
                    .child(Text::new().text(last_sync).variant(TextVariant::Body))
                    .into(),
                Field::new()
                    .name("sync_summary")
                    .label("i18n:host.status.summary")
                    .child(Text::new().text(summary).variant(TextVariant::Caption))
                    .into(),
            ])
            .into(),
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
        Button::new()
            .label("i18n:host.save")
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    ];

    Surface::new(
        Page::new()
            .title("i18n:surface.host.main.title")
            .child(
                Text::new()
                    .text("i18n:surface.host.main.subtitle")
                    .variant(TextVariant::Body),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id(crate::ids::HOST_MAIN)
}
