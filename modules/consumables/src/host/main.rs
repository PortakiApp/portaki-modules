//! Host property editor — catalog + open reports (`consumables-editor-v1`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Form, Grid, IndexedInput, InfoBanner, List, Page, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::labels::{self, lang_code};
use crate::storage;

use super::report_ui::build_report_block;

const ITEM_SLOTS: usize = 8;

/// Host main — catalog IndexedInputs + seed defaults + open shortage reports.
///
/// Save chrome is owned by the modules sheet / workspace (`updateConfig`).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let items = storage::list_items().unwrap_or_default();
    let open_reports = storage::list_open().unwrap_or_default();
    let locale = ctx.locale.as_str();

    let mut tiles: Vec<Component> = Vec::with_capacity(ITEM_SLOTS);
    for index in 0..ITEM_SLOTS {
        let label = items
            .get(index)
            .map(|item| labels::get_label(item, &lang))
            .unwrap_or_default();

        tiles.push(
            IndexedInput::new()
                .index((index + 1) as u32)
                .name(format!("items.{index}.label"))
                .value(label)
                .placeholder("i18n:host.item.empty")
                .showCheck(true)
                .into(),
        );
    }

    let catalog_form = Form::new().child(
        Card::new()
            .title("i18n:host.main.catalogTitle")
            .subtitle("i18n:host.main.catalogHelp")
            .icon("package")
            .child(
                Grid::new()
                    .columns(4)
                    .gap(10.0)
                    .minColumnWidth(280.0)
                    .children(tiles),
            ),
    );

    let seed_card = Card::new()
        .title("i18n:host.main.seedDefaults")
        .subtitle("i18n:host.main.seedHelp")
        .icon("sparkles")
        .child(
            Form::new().child(
                Button::new()
                    .label("i18n:host.main.seedDefaults")
                    .action(crate::ids::module_id().command_empty(crate::ids::SEED_DEFAULTS)),
            ),
        );

    let recent_body: Vec<Component> = if open_reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon("package")
            .into()]
    } else {
        let report_items: Vec<Component> = open_reports
            .iter()
            .map(|report| build_report_block(report, locale))
            .collect();
        vec![
            Text::new()
                .text("i18n:host.main.recentIntro")
                .variant(TextVariant::Caption)
                .into(),
            Component::List(List::new().children(report_items)),
        ]
    };

    let recent_card = Card::new()
        .title("i18n:host.main.recentTitle")
        .subtitle("i18n:host.main.recentHelp")
        .icon("package")
        .children(recent_body);

    let mut children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        catalog_form.into(),
    ];
    if items.is_empty() {
        children.push(seed_card.into());
    }
    children.push(recent_card.into());

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
        .with_id(crate::ids::HOST_MAIN)
}
