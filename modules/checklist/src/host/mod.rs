//! Host dashboard surface — design `checklist-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Card, Field, Form, Page, Stack, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::labels::{self, lang_code};
use crate::storage;

const ITEM_SLOTS: usize = 6;

/// Host checklist editor — item cards → `replaceItems` for active locale.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let items = storage::list_items().unwrap_or_default();

    let submit_items: Vec<crate::commands::ChecklistItemInput> = items
        .iter()
        .map(|item| crate::commands::ChecklistItemInput {
            label: labels::get_label(item, &lang),
            label_fr: String::new(),
            label_en: String::new(),
            sort_order: item.sort_order,
        })
        .collect();

    let save_action = crate::ids::module_id().command(
        crate::ids::REPLACE_ITEMS,
        crate::commands::ReplaceItemsArgs {
            items: submit_items,
            items_json: None,
        },
    );

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..ITEM_SLOTS {
        let item = items.get(index);
        let slot = index + 1;
        let label = item
            .map(|i| labels::get_label(i, &lang))
            .unwrap_or_default();

        cards.push(
            Card::new()
                .title(format!("i18n:host.item.slot{slot}"))
                .icon("check-circle")
                .children(vec![Field::new()
                    .name(format!("items.{index}.label"))
                    .label("i18n:host.item.label")
                    .child(
                        TextInput::new()
                            .name(format!("items.{index}.label"))
                            .value(label),
                    )
                    .into()])
                .into(),
        );
    }
    cards.push(
        Button::new()
            .label("i18n:host.save")
            .tone(Tone::Primary)
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(crate::ids::HOST_MAIN)
}
