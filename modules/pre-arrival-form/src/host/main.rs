//! Host property surface — design `editorPrearrival` informational layout.
//!
//! Runtime has no host config for when/question toggles yet — cards mirror the
//! fixed guest fields (arrival, occasion, allergies, message) as read-only rows.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, InfoBanner, ListItem, Page, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

/// Host main — explains the guest pre-arrival form (no editable config keys).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;

    let question_rows: Vec<Component> = vec![
        ListItem::new()
            .title("i18n:form.arrival.label")
            .leading("clock-circle")
            .chevron(false)
            .into(),
        ListItem::new()
            .title("i18n:form.occasion.label")
            .leading("gift")
            .chevron(false)
            .into(),
        ListItem::new()
            .title("i18n:form.allergies.label")
            .leading("info-circle")
            .chevron(false)
            .into(),
        ListItem::new()
            .title("i18n:form.message.label")
            .leading("message")
            .chevron(false)
            .into(),
    ];

    let children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        Card::new()
            .title("i18n:host.section.when")
            .subtitle("i18n:host.section.when.help")
            .icon("clock-circle")
            .child(
                Text::new()
                    .text("i18n:host.section.when.body")
                    .variant(TextVariant::Body),
            )
            .into(),
        Card::new()
            .title("i18n:host.section.questions")
            .subtitle("i18n:host.section.questions.help")
            .icon("clipboard")
            .children(question_rows)
            .into(),
    ];

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
        .with_id(crate::ids::HOST_MAIN)
}
