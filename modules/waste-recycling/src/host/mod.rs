//! Host dashboard surface — design `waste-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, Page, Select, Stack, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{bin_color_name, BinRow, Localized, ModuleConfig};

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
    let lang = Localized::lang_code(&ctx.locale);
    let config = ModuleConfig::read(&ctx)?;
    let bins = config.parse_bins();

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..BIN_SLOTS {
        cards.push(bin_card(index, bins.get(index), &lang));
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
                        .value(config.collection_schedule.pick(&lang))
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
    .with_id(crate::ids::HOST_MAIN))
}

fn bin_card(index: usize, bin: Option<&BinRow>, lang: &str) -> Component {
    let slot = index + 1;
    let title = bin.map(|b| b.title.pick(lang)).unwrap_or_default();
    // One line: an older bin's several items are joined rather than dropped at the next save.
    let items = bin
        .map(|b| {
            b.items
                .iter()
                .map(|item| item.pick(lang))
                .filter(|item| !item.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let color = bin_color_name(bin.and_then(|b| b.color.as_deref())).unwrap_or("");

    Card::new()
        .title(format!("i18n:host.bin.slot{slot}"))
        .icon(IconName::Refresh)
        .children(vec![
            Field::new()
                .name(format!("bins.{index}.title"))
                .label("i18n:host.bin.title")
                .child(
                    TextInput::new()
                        .name(format!("bins.{index}.title"))
                        .value(title),
                )
                .into(),
            Field::new()
                .name(format!("bins.{index}.items"))
                .label("i18n:host.bin.items")
                .child(
                    TextInput::new()
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
        .into()
}
