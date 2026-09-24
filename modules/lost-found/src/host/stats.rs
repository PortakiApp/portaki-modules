//! Property stats tab — `property-stats-card` host surface: the recent items and their status.
//!
//! The list lives here, not in the config tab: it is follow-up, not configuration.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, List, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use portaki_sdk::host::time;

use crate::storage;

use super::status_ui::build_report_block;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
fn host_now() -> chrono::DateTime<chrono::Utc> {
    time::now().unwrap_or_else(|_| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).expect("epoch is valid")
    })
}

/// « Objets déclarés récemment » — latest property reports with a status update form each.
#[portaki_sdk::surface(host, id = "lost-stats")]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let reports = storage::list_recent().unwrap_or_default();
    let locale = ctx.locale.as_str();

    let recent_body: Vec<Component> = if reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon("search")
            .into()]
    } else {
        let now = host_now();
        let items: Vec<Component> = reports
            .iter()
            .map(|report| build_report_block(report, now, locale))
            .collect();
        vec![
            Text::new()
                .text("i18n:host.main.recentIntro")
                .variant(TextVariant::Caption)
                .into(),
            Component::List(List::new().children(items)),
        ]
    };

    Surface::new(
        Page::new().child(
            Card::new()
                .title("i18n:host.main.recentTitle")
                .subtitle("i18n:host.main.recentHelp")
                .icon("search")
                .children(recent_body),
        ),
    )
    .with_id(crate::ids::HOST_STATS)
}
