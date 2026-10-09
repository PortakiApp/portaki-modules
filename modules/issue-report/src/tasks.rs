//! « Signalement non traité depuis 24 h » (spec Signaler §1) : `timelineTasks`, `taskToggle`,
//! `taskComplete`.
//!
//! Une tâche par signalement ouvert depuis plus de 24 h, calculée, jamais stockée ; son id est
//! celui du signalement. Son unique case, « Marquer comme résolu », clôt le signalement : la
//! tâche disparaît au calcul suivant.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TimelineTaskItem, TimelineTasks, TimelineTasksArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::IssueReport;
use crate::{i18n, storage};

const ITEM: &str = "resolve";

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(_ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let reports = storage::list_since(DateTime::<Utc>::UNIX_EPOCH)?;
    Ok(TimelineTasks {
        tasks: overdue(&reports, time::now()?, args.from, args.to)
            .into_iter()
            .map(|(report, at)| {
                timeline::task(
                    report.id.to_string(),
                    at,
                    args.property_id,
                    i18n::text("task.title", &[]),
                    // Le résumé du voyageur, tel qu'il l'a écrit.
                    I18nText::new(&report.summary, &report.summary),
                )
                .stay(report.stay_id)
                .items(vec![TimelineTaskItem::new(
                    ITEM,
                    i18n::text("host.detail.resolve", &[]),
                )])
            })
            .collect(),
    })
}

/// Les signalements ouverts depuis 24 h et où les poser : 24 h après l'envoi, ramené au début de
/// la fenêtre quand c'est plus tôt — un signalement oublié reste à faire, il ne sort pas de la
/// frise.
pub fn overdue(
    reports: &[IssueReport],
    now: DateTime<Utc>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<(&IssueReport, DateTime<Utc>)> {
    let mut out: Vec<_> = reports
        .iter()
        .filter(|report| report.resolved_at.is_none())
        .map(|report| (report, report.created_at + Duration::hours(24)))
        .filter(|(_, late_at)| *late_at <= now && *late_at <= to)
        .map(|(report, late_at)| (report, late_at.max(from)))
        .collect();
    out.sort_by_key(|(_, at)| *at);
    out
}

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Clore le signalement",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","itemId":"resolve","done":true}"#
    )
)]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    // Un signalement clos ne se rouvre pas.
    if !args.done {
        return Err(PortakiError::Host("cannot_reopen".to_string()));
    }
    complete(&ctx, &args.task_id)
}

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Clore le signalement",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a"}"#
    )
)]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    complete(&ctx, &args.task_id)
}

fn complete(ctx: &Context, task_id: &str) -> Result<()> {
    let report_id =
        Uuid::parse_str(task_id).map_err(|_| PortakiError::Host("task_not_found".to_string()))?;
    // Pas de `checklist.task-updated` : le runtime réserve `checklist.*` au module checklist, et
    // l'envoi refusé ferait échouer une résolution déjà enregistrée. Le journal suffit.
    crate::commands::resolve_report(ctx, report_id).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn report(created: &str, resolved: Option<&str>) -> IssueReport {
        IssueReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            category: "appliance".into(),
            summary: "Plus d'eau chaude".into(),
            details: None,
            created_at: at(created),
            resolved_at: resolved.map(at),
            photo: None,
        }
    }

    #[test]
    fn only_reports_open_for_a_day_become_tasks() {
        let reports = [
            report("2026-08-10T08:00:00Z", None), // 26 h : tâche
            report("2026-08-10T20:00:00Z", None), // 14 h : pas encore
            report("2026-08-09T08:00:00Z", Some("2026-08-09T09:00:00Z")), // résolu
            report("2026-08-01T08:00:00Z", None), // ancien : au début
        ];
        let tasks = overdue(
            &reports,
            at("2026-08-11T10:00:00Z"),
            at("2026-08-04T00:00:00Z"),
            at("2026-08-18T00:00:00Z"),
        );
        let ats: Vec<_> = tasks.iter().map(|(_, at)| *at).collect();
        assert_eq!(
            ats,
            [at("2026-08-04T00:00:00Z"), at("2026-08-11T08:00:00Z")]
        );
        assert_eq!(tasks[1].0.id, reports[0].id);
    }
}
