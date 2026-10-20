//! `timelineTasks` — les demandes à traiter, dans À venir.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{self, TimelineTaskItem, TimelineTasks, TimelineTasksArgs};
use portaki_sdk::prelude::*;

use super::tasks::{to_handle, ITEM};
use crate::storage;

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    // Demandes fermées : l'onglet et les tâches sont inactifs (§3).
    if !crate::Products::load(&ctx)?.meta().enabled() {
        return Ok(TimelineTasks { tasks: Vec::new() });
    }
    let reports = storage::list_all()?;
    Ok(TimelineTasks {
        tasks: to_handle(&reports, args.from, args.to)
            .into_iter()
            .map(|(report, at)| {
                let guest = args
                    .stays
                    .iter()
                    .find(|stay| stay.id == report.stay_id)
                    .map(|stay| stay.guest_name.trim())
                    .unwrap_or_default();
                timeline::task(
                    report.id.to_string(),
                    at,
                    args.property_id,
                    portaki_sdk::bundle_text!("task.title", &[("item", &report.item_label)]),
                    I18nText::new(guest, guest),
                )
                .stay(report.stay_id)
                .items(vec![TimelineTaskItem::new(
                    ITEM,
                    portaki_sdk::bundle_text!("host.main.markRestocked", &[]),
                )])
            })
            .collect(),
    })
}
