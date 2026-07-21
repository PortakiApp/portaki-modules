//! Guest home booklet card — mixed-destination departure board glance.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Card, Text, TimedEntry};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{home_board, station_caption, MODULE_ICON};

pub fn build_home_card(ctx: &GuestContext) -> Surface {
    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(json!(station_caption(&ctx.locale)))
            .variant(json!("caption"))
            .emphasis(Emphasis::Subtle),
    )];
    children.extend(home_board().into_iter().map(board_entry_component));

    Surface::new(
        Card::new()
            .icon(json!(MODULE_ICON))
            .title(json!("i18n:home.card.title"))
            .action(json!({
                "type": "openOverlay",
                "presentation": "fullscreen",
                "surfaceRender": "explore.detail",
                "args": {
                    "icon": MODULE_ICON,
                    "title": "i18n:home.card.title"
                }
            }))
            .children(children),
    )
    .with_id("home.card")
}

fn board_entry_component(entry: crate::content::BoardEntry) -> Component {
    Component::TimedEntry(
        TimedEntry::new()
            .time(json!(entry.time))
            .title(json!(entry.destination))
            .subtitle(json!(entry.platform)),
    )
}
