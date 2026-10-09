//! Encart du séjour (spec Signaler §1) : « 1 signalement en cours », puis chaque signalement du
//! séjour — résumé, il y a combien de temps, statut. Une ligne ouvre le détail, comme dans les
//! statistiques.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Page, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::storage;

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::DangerTriangle
)]
pub fn render_host_stay(ctx: HostContext) -> Surface {
    let card = Card::new()
        .title("i18n:host.stay.title")
        .icon(IconName::DangerTriangle);
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let card = match stay_id {
        None => card.child(
            Text::new()
                .text("i18n:host.stay.missingStay")
                .variant(TextVariant::Caption),
        ),
        Some(stay_id) => {
            let reports = storage::list_by_stay(stay_id).unwrap_or_default();
            if reports.is_empty() {
                card.child(
                    EmptyState::new()
                        .title("i18n:host.stay.empty")
                        .description("i18n:host.stay.empty.help")
                        .icon(IconName::DangerTriangle),
                )
            } else {
                let open = reports.iter().filter(|r| r.resolved_at.is_none()).count();
                let headline = match open {
                    0 => "i18n:host.stay.openNone".to_string(),
                    1 => "i18n:host.stay.openOne".to_string(),
                    count => {
                        t!("host.stay.openMany", count = count.to_string()).unwrap_or_default()
                    }
                };
                let now = super::host_now();
                card.subtitle(headline).child(
                    Stack::new().gap(0.0).children(
                        reports
                            .iter()
                            .map(|report| super::report_row(report, now, &ctx.locale))
                            .collect(),
                    ),
                )
            }
        }
    };
    Surface::new(Page::new().child(card)).with_id(STAY)
}
