//! Host dashboard surface — design `waste-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, Page, Select, Stack, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{bin_color_name, BinRow, ModuleConfig};

const BIN_SLOTS: usize = 6;

/// Host configuration page — bin cards + collection schedule.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::WasteEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Recycle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // The stored rows where they are, blank ones included, then empty slots.
    let mut cards: Vec<Component> = Vec::new();
    for index in 0..BIN_SLOTS.max(config.bins.len()) {
        cards.push(bin_card(index, config.bins.get(index), &ctx));
    }
    cards.push(
        Card::new()
            .title("i18n:host.section.schedule")
            .icon(IconName::Calendar)
            .children(vec![Field::new()
                .name("collection_schedule")
                .label("i18n:host.schedule.label")
                .child(
                    TextArea::new()
                        .name("collection_schedule")
                        .value(config.collection_schedule.host_value(&ctx))
                        .placeholder("i18n:host.schedule.placeholder"),
                )
                .into()])
            .into(),
    );

    // No Save button — the modules drawer owns the footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(MAIN))
}

fn bin_card(index: usize, bin: Option<&BinRow>, ctx: &HostContext) -> Component {
    let slot = index + 1;
    let title = bin.map(|b| b.title.host_value(ctx)).unwrap_or_default();
    let items = bin.map(|b| b.items.host_value(ctx)).unwrap_or_default();
    let color = bin_color_name(bin.and_then(|b| b.color.as_deref())).unwrap_or("");
    // A filled row sends its id, so a save merges into it (and keeps its other languages). A
    // blank slot has nothing to keep — and an id would make it count as filled.
    let id = bin
        .filter(|b| !b.is_blank())
        .map(|b| sdui::row_id("bins", index, Some(&b.id)));

    Card::new()
        .title(format!("i18n:host.bin.slot{slot}"))
        .icon(IconName::Refresh)
        .children(
            id.into_iter()
                .chain([
                    Field::new()
                        .name(format!("bins.{index}.title"))
                        .label("i18n:host.bin.title")
                        .child(
                            TextInput::new()
                                .name(format!("bins.{index}.title"))
                                .value(title),
                        )
                        .into(),
                    // One item per line.
                    Field::new()
                        .name(format!("bins.{index}.items"))
                        .label("i18n:host.bin.items")
                        .child(
                            TextArea::new()
                                .name(format!("bins.{index}.items"))
                                .value(items)
                                .placeholder("i18n:host.bin.items.placeholder"),
                        )
                        .into(),
                    Field::new()
                        .name(format!("bins.{index}.color"))
                        .label("i18n:host.bin.color")
                        .child(
                            Select::new()
                                .name(format!("bins.{index}.color"))
                                .options(vec![
                                    ChoiceOption::new("", "i18n:host.bin.color.none"),
                                    ChoiceOption::new("yellow", "i18n:host.bin.color.yellow"),
                                    ChoiceOption::new("green", "i18n:host.bin.color.green"),
                                    ChoiceOption::new("brown", "i18n:host.bin.color.brown"),
                                    ChoiceOption::new("grey", "i18n:host.bin.color.grey"),
                                ])
                                .value(color),
                        )
                        .into(),
                ])
                .collect(),
        )
        .into()
}
