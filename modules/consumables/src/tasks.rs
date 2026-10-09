//! « Café demandé · Marie » dans À venir (spec Consommables §1) : `timelineTasks`, `taskToggle`,
//! `taskComplete`.
//!
//! Une tâche par demande encore à traiter, calculée, jamais stockée ; son id est celui de la
//! demande. Une demande « Prévu » n'en est plus une : l'hôte l'a déjà vue. L'unique case,
//! « Marquer livré », clôt la demande et la tâche disparaît au calcul suivant.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TimelineTaskItem, TimelineTasks, TimelineTasksArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::ConsumableReport;
use crate::{i18n, status, storage};

const ITEM: &str = "restocked";

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(_ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    // Demandes fermées : l'onglet et les tâches sont inactifs (§3).
    if !storage::settings::read().requests_enabled() {
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
                    i18n::text("task.title", &[("item", &report.item_label)]),
                    I18nText::new(guest, guest),
                )
                .stay(report.stay_id)
                .items(vec![TimelineTaskItem::new(
                    ITEM,
                    i18n::text("host.main.markRestocked", &[]),
                )])
            })
            .collect(),
    })
}

/// Les demandes à traiter et où les poser : à l'envoi, ramené au début de la fenêtre quand c'est
/// plus tôt — une demande oubliée reste à faire, elle ne sort pas de la frise.
pub fn to_handle(
    reports: &[ConsumableReport],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<(&ConsumableReport, DateTime<Utc>)> {
    let mut out: Vec<_> = reports
        .iter()
        .filter(|report| report.status == status::DEFAULT && report.created_at <= to)
        .map(|report| (report, report.created_at.max(from)))
        .collect();
    out.sort_by_key(|(_, at)| *at);
    out
}

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Marquer livré",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","itemId":"restocked","done":true}"#
    )
)]
pub fn task_toggle(_ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    // Décocher rouvre la demande : l'hôte s'est trompé de ligne.
    let next = if args.done {
        status::RESTOCKED
    } else {
        status::DEFAULT
    };
    set_status(&args.task_id, next)
}

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Marquer livré",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a"}"#
    )
)]
pub fn task_complete(_ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    set_status(&args.task_id, status::RESTOCKED)
}

fn set_status(task_id: &str, next: &str) -> Result<()> {
    let report_id =
        Uuid::parse_str(task_id).map_err(|_| PortakiError::Host("task_not_found".to_string()))?;
    storage::update_status(report_id, next.to_string(), None).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn report(created: &str, status: &str) -> ConsumableReport {
        ConsumableReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            item_id: Uuid::new_v4(),
            item_label: "Café".into(),
            level: "missing".into(),
            note: None,
            status: status.into(),
            created_at: at(created),
            restocked_at: None,
            host_reply: None,
        }
    }

    #[test]
    fn only_requests_still_to_handle_become_tasks() {
        let reports = [
            report("2026-08-10T08:00:00Z", "open"),      // à traiter
            report("2026-08-10T09:00:00Z", "planned"),   // déjà vue
            report("2026-08-10T10:00:00Z", "restocked"), // livrée
            report("2026-08-01T08:00:00Z", "open"),      // ancienne : au début
            report("2026-08-20T08:00:00Z", "open"),      // après la fenêtre
        ];
        let tasks = to_handle(
            &reports,
            at("2026-08-04T00:00:00Z"),
            at("2026-08-18T00:00:00Z"),
        );
        let ats: Vec<_> = tasks.iter().map(|(_, at)| *at).collect();
        assert_eq!(
            ats,
            [at("2026-08-04T00:00:00Z"), at("2026-08-10T08:00:00Z")]
        );
        assert_eq!(tasks[1].0.id, reports[0].id);
    }
}
