//! « Café demandé · Marie » dans À venir (spec Consommables §1) : `timelineTasks`, `taskToggle`,
//! `taskComplete`.
//!
//! Une tâche par demande encore à traiter, calculée, jamais stockée ; son id est celui de la
//! demande. Une demande « Prévu » n'en est plus une : l'hôte l'a déjà vue. L'unique case,
//! « Marquer livré », clôt la demande et la tâche disparaît au calcul suivant.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::ConsumableReport;
use crate::{status, storage};

pub(super) const ITEM: &str = "restocked";

/// Les demandes à traiter et où les poser : à l'envoi, ramené au début de la fenêtre quand c'est
/// plus tôt — une demande oubliée reste à faire, elle ne sort pas de la frise.
pub(super) fn to_handle(
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

pub(super) fn set_status(task_id: &str, next: &str) -> Result<()> {
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
            guest_name: String::new(),
            stay_check_in: None,
            stay_check_out: None,
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
