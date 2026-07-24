//! Host dashboard surface — design `checklist-editor-v1` (Wasm SDUI).
//!
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Form, Grid, IndexedInput, Page};
use portaki_sdk::sdui::surface::Surface;

use crate::labels::{self, lang_code};
use crate::storage;

const ITEM_SLOTS: usize = 6;

/// Host checklist editor — indexed tiles → `updateConfig` / `replaceItems`.
///
/// Emits all [`ITEM_SLOTS`] inputs. The host `IndexedInput` binding keeps one
/// trailing empty slot from draft form values (empty → 1 blank; typing → +1).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let items = storage::list_items().unwrap_or_default();

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

    // No in-form Save — workspace header owns Enregistrer → updateConfig.
    Surface::new(
        Page::new().child(
            Form::new().child(
                Card::new()
                    .title("i18n:surface.host.main.title")
                    .subtitle("i18n:surface.host.main.subtitle")
                    .icon("check-circle")
                    .child(
                        Grid::new()
                            .columns(4)
                            .gap(10.0)
                            .minColumnWidth(280.0)
                            .children(tiles),
                    ),
            ),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}
