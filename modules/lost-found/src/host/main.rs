//! Host property editor — design `editorLostFound` / `lostfound-editor-v1`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, InfoBanner, Page, RichTextEditor, Stack};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;

/// Host main — info banner + optional TipTap guest note. The declared items live in the
/// property stats tab (`lost-stats`).
///
/// Save chrome is owned by the modules sheet / workspace (`updateConfig`).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let host_note = config.host_note.clone().unwrap_or_default();

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

    let children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        note_form.into(),
    ];

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
        .with_id(crate::ids::HOST_MAIN)
}
