//! L'éditeur des checklists, dérivé de la collection : une ligne par liste, son panneau avec les
//! réglages de son public et ses étapes. Sous le tableau, les modèles de départ.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::CollectionEditor;
use portaki_sdk::sdui::primitives::{FieldHint, Form, Grid, Page, Section, SelectableCard, Stack};

use crate::commands::{CreateChecklistArgs, CREATE_CHECKLIST};
use crate::lists::TEMPLATES;
use crate::{List, Lists};

/// La nav dit au dashboard d'appeler `timelineTasks` : les tâches d'équipe dans À venir.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Sparkles
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::ChecklistEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::CheckCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let lists = Lists::load(&ctx)?;
    let live: Vec<String> = crate::domain::booklet_lists()?
        .into_iter()
        .map(|list| list.id)
        .collect();
    let editor = CollectionEditor::new(&ctx, &lists)
        .column("name")
        .column("audience")
        .computed("meta", "host.steps.title")
        .cell("meta", move |list: &List| meta(list, &live))
        .when_empty(FieldHint::new().text("i18n:host.new.hint"));
    let page = Stack::new()
        .gap(16.0)
        .child(Form::new().children(editor.build()))
        .child(templates());
    Ok(Surface::new(Page::new().child(page)))
}

/// « Dans le livret · 5 étapes », ou « 5 étapes ».
fn meta(list: &List, live: &[String]) -> String {
    let key = if live.contains(&list.id) {
        "host.lists.meta.live"
    } else {
        "host.lists.meta.steps"
    };
    t!(key, count = list.written_steps().len()).unwrap_or_default()
}

/// « Nouvelle checklist » : un modèle ne fait que pré-remplir, tout reste modifiable.
fn templates() -> Component {
    let cards = TEMPLATES
        .iter()
        .map(|template| {
            let args = CreateChecklistArgs {
                template: template.id.to_string(),
                id: None,
            };
            SelectableCard::new()
                .value(template.id)
                .label(format!("i18n:template.{}.name", template.id))
                .description(format!("i18n:template.{}.desc", template.id))
                .icon(template.icon)
                .action(crate::ids::module_id().command(CREATE_CHECKLIST, args))
                .into()
        })
        .collect();
    let grid = Grid::new().minColumnWidth(180.0).gap(10.0).children(cards);
    Section::new()
        .title("i18n:host.new.title")
        .child(grid)
        .child(FieldHint::new().text("i18n:host.new.hint"))
        .into()
}
