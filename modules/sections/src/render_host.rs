//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::model::lang_code;
use crate::store;

/// Host editor form — create/update one section for the active `ctx.locale`.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let property_locale = ctx.property.locale.clone();
    let sections = store::list_all(&ctx.locale, &property_locale).unwrap_or_default();
    let (title, body) = sections
        .first()
        .map(|section| {
            let row = section.locales.iter().find(|l| lang_code(&l.lang) == lang);
            (
                row.map(|l| l.title.clone())
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or_else(|| section.title.clone()),
                row.map(|l| l.body_markdown.clone())
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or_else(|| section.body_markdown.clone()),
            )
        })
        .unwrap_or_else(|| {
            if lang == "en" {
                (
                    "Welcome".into(),
                    "Welcome to L'Islette! The whole team wishes you a great stay.".into(),
                )
            } else {
                (
                    "Bienvenue".into(),
                    "Bienvenue à L'Islette ! Toute l'équipe vous souhaite un excellent séjour."
                        .into(),
                )
            }
        });

    let section_id = sections.first().map(|s| s.id);
    let submit_args = json!({
        "id": section_id,
        "title": title,
        "body_markdown": body,
        "lang": lang,
        "locales": []
    });
    let save_action = serde_json::to_value(Action::command("sections", "saveSection", submit_args))
        .unwrap_or(json!({}));

    Surface::new(
        Page::new()
            .title(json!("i18n:surface.host.main.title"))
            .child(
                Text::new()
                    .text(json!("i18n:surface.host.main.subtitle"))
                    .variant(json!("body")),
            )
            .child(
                Form::new()
                    .child(
                        Field::new()
                            .name(json!("title"))
                            .label(json!("i18n:host.title.label"))
                            .child(TextInput::new().name(json!("title")).value(json!(title))),
                    )
                    .child(
                        Field::new()
                            .name(json!("body_markdown"))
                            .label(json!("i18n:host.body.label"))
                            .child(
                                TextArea::new()
                                    .name(json!("body_markdown"))
                                    .value(json!(body)),
                            ),
                    )
                    .child(
                        Text::new()
                            .text(json!("i18n:host.main.help"))
                            .variant(json!("caption")),
                    )
                    .child(
                        Button::new()
                            .label(json!("i18n:host.save"))
                            .action(save_action),
                    ),
            ),
    )
    .with_id("main")
}
