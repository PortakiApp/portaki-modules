//! Host dashboard surface — design `sections-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, Page, Stack, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

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
    let submit_args = crate::commands::SaveSectionArgs {
        id: section_id,
        sort_order: None,
        locales: Vec::new(),
        title: title.clone(),
        body_markdown: body.clone(),
        lang: lang.clone(),
        title_fr: String::new(),
        title_en: String::new(),
        body_markdown_fr: String::new(),
        body_markdown_en: String::new(),
    };
    let save_action = crate::ids::module_id().command(crate::ids::SAVE_SECTION, submit_args);

    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Card::new()
                        .title("i18n:host.section.content")
                        .subtitle("i18n:host.section.content.help")
                        .icon("file-text")
                        .children(vec![
                            Field::new()
                                .name("title")
                                .label("i18n:host.title.label")
                                .child(TextInput::new().name("title").value(title))
                                .into(),
                            Field::new()
                                .name("body_markdown")
                                .label("i18n:host.body.label")
                                .child(TextArea::new().name("body_markdown").value(body))
                                .into(),
                        ])
                        .into(),
                    Button::new()
                        .label("i18n:host.save")
                        .tone(Tone::Primary)
                        .action(save_action)
                        .into(),
                ]))),
    )
    .with_id(crate::ids::HOST_MAIN)
}
