//! Stay-scoped host surface — list / status for one stay (create is stay-action).
//!
//! Empty stay → Card + EmptyState (tab must not look blank). Non-empty → Card + list.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, List, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::storage;

use super::status_ui::build_report_block;

/// Stay detail embed — reports / status for the stay (no create form).
///
/// When there are no reports, shows an empty-state card so the stay-detail tab
/// is not blank. The stay-action « Déclarer un objet trouvé » button stays
/// available regardless.
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
                let items: Vec<Component> = reports
                    .iter()
                    .map(|report| build_report_block(report, locale))
                    .collect();
                vec![Card::new()
                    .title("i18n:host.stay.listTitle")
                    .icon("search")
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
        .icon("search")
        .children(vec![EmptyState::new()
            .title("i18n:host.stay.empty")
            .description("i18n:host.stay.empty.help")
            .icon("search")
            .into()])
        .into()
}
