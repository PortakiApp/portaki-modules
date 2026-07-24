//! Property stats strip — `property-stats-card` host surface (`stock`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::storage;

#[portaki_sdk::surface(host, id = "stock")]
pub fn render_host_stats(_ctx: HostContext) -> Surface {
    let catalog_count = storage::list_items().map(|items| items.len()).unwrap_or(0);
    let open_count = storage::count_open().unwrap_or(0);

    let mut children: Vec<Component> = vec![
        Field::new()
            .name("catalog_count")
            .label("i18n:stats.catalog")
            .child(
                Text::new()
                    .text(format!("{catalog_count}"))
                    .variant(TextVariant::Body),
            )
            .into(),
        Field::new()
            .name("open_count")
            .label("i18n:stats.open")
            .child(
                Text::new()
                    .text(format!("{open_count}"))
                    .variant(TextVariant::Body),
            )
            .into(),
    ];

    if catalog_count == 0 {
        children.push(
            Text::new()
                .text("i18n:stats.emptyHint")
                .variant(TextVariant::Caption)
                .into(),
        );
    }

    Surface::new(
        Page::new().child(
            Card::new()
                .title("i18n:stats.title")
                .subtitle("i18n:stats.subtitle")
                .icon("package")
                .children(children),
        ),
    )
    .with_id(crate::ids::HOST_STATS)
}
