//! Host property editor — design `editorLostFound` / `lostfound-editor-v1`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, EmptyState, Field, Form, InfoBanner, List, Page, RichTextEditor, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;
use crate::storage;

use super::status_ui::build_report_block;

/// Host main — info banner, optional TipTap guest note, recent reports + status.
///
/// Save chrome is owned by the modules sheet / workspace (`updateConfig`).
/// Note form is separate from per-report status forms (no nested forms).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let host_note = config.host_note.clone().unwrap_or_default();
    let reports = storage::list_recent().unwrap_or_default();
    let locale = ctx.locale.as_str();

    let note_form = Form::new().child(
        Card::new()
            .title("i18n:host.hostNote.label")
            .subtitle("i18n:host.hostNote.help")
            .icon("search")
            .children(vec![Field::new()
                .name("host_note")
                .label("i18n:host.hostNote.fieldLabel")
                .child(RichTextEditor::new().name("host_note").value(host_note))
                .into()]),
    );

    let recent_body: Vec<Component> = if reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon("search")
            .into()]
    } else {
        let items: Vec<Component> = reports
            .iter()
            .map(|report| build_report_block(report, locale))
            .collect();
        vec![
            Text::new()
                .text("i18n:host.main.recentIntro")
                .variant(TextVariant::Caption)
                .into(),
            Component::List(List::new().children(items)),
        ]
    };

    let recent_card = Card::new()
        .title("i18n:host.main.recentTitle")
        .subtitle("i18n:host.main.recentHelp")
        .icon("search")
        .children(recent_body);

    let children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        note_form.into(),
        recent_card.into(),
    ];

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
        .with_id(crate::ids::HOST_MAIN)
}
