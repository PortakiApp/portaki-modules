//! Stay-scoped host surface — shortage reports for one stay.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, List, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use portaki_sdk::host::time;

use crate::storage;

use super::report_ui::build_report_block;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
fn host_now() -> chrono::DateTime<chrono::Utc> {
    time::now().unwrap_or_else(|_| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).expect("epoch is valid")
    })
}

/// Stay detail embed — guest shortage reports for `input.stayId`.
///
/// Empty stay → Card + EmptyState (tab must not look blank). Non-empty → Card + list.
#[portaki_sdk::surface(host, id = "stay")]
pub fn render_host_stay(ctx: HostContext) -> Surface {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let locale = ctx.locale.as_str();

    let children: Vec<Component> = match stay_id {
        None => vec![Text::new()
            .text("i18n:host.stay.missingStay")
            .variant(TextVariant::Caption)
            .into()],
        Some(stay_id) => {
            let reports = storage::list_by_stay(stay_id).unwrap_or_default();
            if reports.is_empty() {
                vec![empty_stay_card()]
            } else {
                let now = host_now();
                let items: Vec<Component> = reports
                    .iter()
                    .map(|report| build_report_block(report, now, locale))
                    .collect();
                vec![Card::new()
                    .title("i18n:host.stay.listTitle")
                    .icon("package")
                    .children(vec![Component::List(List::new().children(items))])
                    .into()]
            }
        }
    };

    Surface::new(Page::new().children(children)).with_id(crate::ids::HOST_STAY)
}

fn empty_stay_card() -> Component {
    Card::new()
        .title("i18n:host.stay.listTitle")
        .icon("package")
        .children(vec![EmptyState::new()
            .title("i18n:host.stay.empty")
            .description("i18n:host.stay.empty.help")
            .icon("package")
            .into()])
        .into()
}
