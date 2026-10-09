//! L'onglet du logement (spec Objet oublié §1, `property-workspace-tab`) : tous les objets,
//! déclarés par les voyageurs ou trouvés par l'hôte, avec leur statut et ce qu'il faut pour les
//! rendre — le souhait du voyageur, l'adresse de renvoi.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, List, Page};
use portaki_sdk::sdui::surface::Surface;

use crate::storage;

use super::status_ui::build_report_block;

// « Objet à renvoyer » dans À venir : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Package
)]
#[portaki_sdk::surface(
    host,
    id = "items",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.items",
    icon = IconName::Search
)]
pub fn render_host_items(ctx: HostContext) -> Result<Surface> {
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let reports = storage::list_since(DateTime::<Utc>::UNIX_EPOCH)?;
    let body: Component = if reports.is_empty() {
        EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon(IconName::Search)
            .into()
    } else {
        List::new()
            .children(
                reports
                    .iter()
                    .map(|report| build_report_block(report, now, &ctx.locale))
                    .collect(),
            )
            .into()
    };
    Ok(Surface::new(
        Page::new().child(
            Card::new()
                .title("i18n:host.items.title")
                .icon(IconName::Search)
                .child(body),
        ),
    )
    .with_id(ITEMS))
}
