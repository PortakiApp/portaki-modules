//! Stay-scoped host surface — shortage reports for one stay.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, List, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::storage;

use super::report_ui::build_report_block;

/// Stay detail embed — reports when any exist; empty tree otherwise.
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
                Vec::new()
            } else {
                let items: Vec<Component> = reports
                    .iter()
                    .map(|report| build_report_block(report, locale))
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
