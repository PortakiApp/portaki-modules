//! Stay-scoped host surface — design `foundObjectModal` / mobile `sheetFound`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{List, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::storage;

use super::create::build_create_found_form;
use super::status_ui::build_report_block;

/// Stay detail embed — declare found (modal layout) + stay reports / status.
#[portaki_sdk::surface(host, id = "stay")]
pub fn render_host_stay(ctx: HostContext) -> Surface {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let locale = ctx.locale.as_str();

    let mut children: Vec<Component> = vec![build_create_found_form(&ctx)];

    match stay_id {
        None => {}
        Some(stay_id) => {
            let reports = storage::list_by_stay(stay_id).unwrap_or_default();
            if reports.is_empty() {
                children.push(
                    Text::new()
                        .text("i18n:host.stay.empty")
                        .variant(TextVariant::Caption)
                        .into(),
                );
            } else {
                children.push(
                    Text::new()
                        .text("i18n:host.stay.listTitle")
                        .variant(TextVariant::Title)
                        .into(),
                );
                let items: Vec<Component> = reports
                    .iter()
                    .map(|report| build_report_block(report, locale))
                    .collect();
                children.push(Component::List(List::new().children(items)));
            }
        }
    }

    // No Page title — HeaderTitle inside the create block owns the modal chrome.
    Surface::new(Page::new().children(children)).with_id(crate::ids::HOST_STAY)
}
