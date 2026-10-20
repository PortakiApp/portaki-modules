//! La boîte de réception, dérivée du dépôt : filtres, lignes, détail avec le séjour.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Inbox};
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::primitives::{Button, Page, Stack, Text};

use crate::Reports;

#[portaki_sdk::surface(host, id = "reports", placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.reports", icon = IconName::DangerTriangle)]
pub fn render_host_reports(ctx: HostContext) -> Result<Surface> {
    let inbox = Inbox::<Reports>::new(&ctx, Reports::DECL, "host")
        .icon(IconName::DangerTriangle)
        .tag(|r| format!("i18n:host.category.{}", r.category.wire()))
        .detail_extra(|e| {
            let dates = match (e.check_in, e.check_out) {
                (Some(from), Some(to)) => {
                    format!("{} → {}", from.format("%d/%m"), to.format("%d/%m"))
                }
                _ => String::new(),
            };
            let line = [e.guest_name.as_str(), dates.as_str()]
                .into_iter()
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join(" · ");
            let stay = e
                .stay_id
                .map(|id| NavigateTarget::path(format!("/stays/{id}")));
            vec![Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(12.0)
                .child(Text::new().text(line).variant(TextVariant::Caption))
                .child(
                    Button::new()
                        .label("i18n:host.detail.openStay")
                        .variant(ButtonVariant::Outline)
                        .action(Action::navigate(
                            stay.unwrap_or_else(|| NavigateTarget::path("/stays")),
                            None,
                        )),
                )
                .into()]
        });
    Ok(Surface::new(Page::new().child(inbox.build()?)))
}
