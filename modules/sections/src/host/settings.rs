//! L'éditeur des sections, dérivé de la collection.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::CollectionEditor;
use portaki_sdk::sdui::primitives::{Form, Page};

use crate::Sections;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::SectionsEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Home
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let sections = Sections::load(&ctx)?;
    let editor = CollectionEditor::new(&ctx, &sections)
        .hint("host.body.hint")
        .column_with("title", "body");
    Ok(Surface::new(
        Page::new().child(Form::new().children(editor.build())),
    ))
}
