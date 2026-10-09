//! « Objet à renvoyer » (spec Objet oublié §1, `workspace-timeline-task`) : `timelineTasks`,
//! `taskToggle`, `taskComplete`.
//!
//! Une tâche par objet dont le voyageur a demandé le renvoi et qui n'est ni rendu ni introuvable ;
//! calculée, jamais stockée, son id est celui de l'objet. Son unique case, « Marquer comme
//! renvoyé », passe l'objet à « Renvoyé » : la tâche disparaît au calcul suivant.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TimelineStay, TimelineTaskItem, TimelineTasks,
    TimelineTasksArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::LostFoundReport;
use crate::{description, i18n, storage};

const ITEM: &str = "shipped";

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(_ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let reports = storage::list_since(DateTime::<Utc>::UNIX_EPOCH)?;
    let now = time::now()?;
    Ok(TimelineTasks {
        tasks: to_ship(&reports, args.from, args.to)
            .into_iter()
            .map(|(report, at)| {
                let days = (now - report.created_at).num_days().max(0).to_string();
                let item = description::to_plain_text(&report.item_description);
                timeline::task(
                    report.id.to_string(),
                    at,
                    args.property_id,
                    title(guest_of(&args.stays, report.stay_id)),
                    i18n::text("task.context", &[("item", &item), ("days", &days)]),
                )
                .stay(report.stay_id)
                .items(vec![TimelineTaskItem::new(
                    ITEM,
                    i18n::text("task.item", &[]),
                )])
            })
            .collect(),
    })
}

/// « Objet à renvoyer à Marie », ou sans prénom connu « Objet à renvoyer ».
fn title(guest: Option<&str>) -> I18nText {
    match guest {
        Some(guest) => i18n::text("task.title.named", &[("guest", guest)]),
        None => i18n::text("task.title", &[]),
    }
}

fn guest_of(stays: &[TimelineStay], stay_id: Uuid) -> Option<&str> {
    stays
        .iter()
        .find(|stay| stay.id == stay_id)
        .map(|stay| stay.guest_name.trim())
        .filter(|name| !name.is_empty())
}

/// Les objets à renvoyer et où les poser : à leur déclaration, ramenée au début de la fenêtre
/// quand elle est plus tôt — un renvoi en retard reste à faire, il ne sort pas de la frise.
pub fn to_ship(
    reports: &[LostFoundReport],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<(&LostFoundReport, DateTime<Utc>)> {
    let mut out: Vec<_> = reports
        .iter()
        .filter(|report| report.return_choice.as_deref() == Some("ship"))
        .filter(|report| matches!(report.status.as_str(), "declared" | "found"))
        .filter(|report| report.created_at <= to)
        .map(|report| (report, report.created_at.max(from)))
        .collect();
    out.sort_by_key(|(_, at)| *at);
    out
}

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Objet renvoyé",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","itemId":"shipped","done":true}"#
    )
)]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    // Un objet renvoyé ne revient pas : « Renvoyé » est un statut final.
    if !args.done {
        return Err(PortakiError::Host("cannot_reopen".to_string()));
    }
    complete(&ctx, &args.task_id)
}

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Objet renvoyé",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a"}"#
    )
)]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    complete(&ctx, &args.task_id)
}

fn complete(ctx: &Context, task_id: &str) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let report_id =
        Uuid::parse_str(task_id).map_err(|_| PortakiError::Host("task_not_found".to_string()))?;
    crate::commands::set_status(ctx, report_id, "shipped")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn report(created: &str, choice: Option<&str>, status: &str) -> LostFoundReport {
        LostFoundReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            kind: "lost".into(),
            item_description: "Chargeur".into(),
            contact_hint: None,
            details: None,
            return_address: None,
            category: None,
            room: None,
            photo: None,
            return_choice: choice.map(str::to_string),
            status: status.into(),
            created_at: at(created),
        }
    }

    #[test]
    fn only_items_to_ship_and_not_yet_back_become_tasks() {
        let reports = [
            report("2026-08-10T08:00:00Z", Some("ship"), "declared"), // tâche
            report("2026-08-10T09:00:00Z", Some("ship"), "found"),    // tâche
            report("2026-08-10T10:00:00Z", Some("pickup"), "found"),  // retrait
            report("2026-08-10T11:00:00Z", Some("ship"), "shipped"),  // déjà parti
            report("2026-08-10T12:00:00Z", Some("ship"), "not_found"), // introuvable
            report("2026-08-01T08:00:00Z", Some("ship"), "found"),    // ancien : au début
            report("2026-08-20T08:00:00Z", Some("ship"), "found"),    // après la fenêtre
        ];
        let tasks = to_ship(
            &reports,
            at("2026-08-04T00:00:00Z"),
            at("2026-08-18T00:00:00Z"),
        );
        let ids: Vec<_> = tasks.iter().map(|(report, _)| report.id).collect();
        assert_eq!(ids, [reports[5].id, reports[0].id, reports[1].id]);
        assert_eq!(tasks[0].1, at("2026-08-04T00:00:00Z"));
        assert_eq!(tasks[1].1, at("2026-08-10T08:00:00Z"));
    }
}
