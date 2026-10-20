//! La reprise des signalements d'avant le `#[record]` : chaque ligne `issue_report` devient un
//! signalement (même id, même séjour, même voyageur), son historique vient des lignes
//! `issue_report_event`, une seule fois — `#[record(legacy = …)]`.
//!
//! Une ligne d'avant la colonne `status` (v1) la lit de `resolved_at`, comme l'ancien module ;
//! sans ligne d'historique, elle reçoit « Signalé » et, résolue, « Résolu » à `resolved_at`.

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::resource::{Actor, Choice, Entry, HistoryEvent, HistoryLine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Category, Report, Status};

/// L'ancien signalement, gardé déclaré pour être lu.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct IssueReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub category: String,
    pub summary: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub photo: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub guest_name: String,
    #[serde(default)]
    pub stay_check_in: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stay_check_out: Option<DateTime<Utc>>,
}

#[portaki_sdk::entity_indexes(IssueReport)]
pub const ISSUE_REPORT_INDEXES: &[&str] = &["stay_id"];

/// Une ancienne ligne d'historique : `created` · `status` · `reply`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct IssueReportEvent {
    pub id: Uuid,
    pub report_id: Uuid,
    pub stay_id: Uuid,
    pub kind: String,
    pub actor: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(IssueReportEvent)]
pub const ISSUE_REPORT_EVENT_INDEXES: &[&str] = &["report_id", "stay_id"];

/// Les signalements d'avant, avec leur historique.
pub fn earlier_reports() -> portaki_sdk::Result<Vec<Entry<Report, Status>>> {
    let reports = find::<IssueReport, IssueReport>(Query::new())?.items;
    if reports.is_empty() {
        return Ok(Vec::new());
    }
    let events = find::<IssueReportEvent, IssueReportEvent>(Query::new())?.items;
    Ok(reports
        .iter()
        .map(|report| convert(report, &events))
        .collect())
}

/// Le signalement d'une ancienne ligne ; `events` peut porter ceux des autres signalements.
pub fn convert(report: &IssueReport, events: &[IssueReportEvent]) -> Entry<Report, Status> {
    let status = status_of(report);
    let mut own: Vec<&IssueReportEvent> =
        events.iter().filter(|e| e.report_id == report.id).collect();
    own.sort_by_key(|e| e.created_at);
    let actor = own
        .iter()
        .find(|e| e.kind == "created")
        .map_or(Actor::Guest, |e| actor_of(&e.actor));
    let mut history: Vec<HistoryLine> = own.iter().filter_map(|e| line(e)).collect();
    if !own.iter().any(|e| e.kind == "created") {
        history.insert(0, at(report.created_at, actor, HistoryEvent::Created));
    }
    if own.is_empty() && status == Status::Resolved {
        if let Some(resolved) = report.resolved_at {
            let to = status.wire().to_string();
            history.push(at(resolved, Actor::Host, HistoryEvent::Status { to }));
        }
    }
    Entry {
        id: report.id,
        item: Report {
            id: report.id.to_string(),
            // Une catégorie que le formulaire n'offre plus se lisait « Autre ».
            category: Category::from_wire(&report.category).unwrap_or(Category::Other),
            summary: report.summary.clone(),
            details: report.details.clone().unwrap_or_default(),
            photo: report.photo.clone().unwrap_or_default(),
        },
        status,
        stay_id: Some(report.stay_id),
        created_at: report.created_at,
        actor,
        guest_name: report.guest_name.clone(),
        check_in: report.stay_check_in,
        check_out: report.stay_check_out,
        history,
    }
}

/// Le statut, tel que l'ancien module le lisait : vide = d'avant la colonne, lu de `resolved_at`.
fn status_of(report: &IssueReport) -> Status {
    Status::from_wire(report.status.trim()).unwrap_or(if report.resolved_at.is_some() {
        Status::Resolved
    } else {
        Status::New
    })
}

fn actor_of(raw: &str) -> Actor {
    if raw == "host" {
        Actor::Host
    } else {
        Actor::Guest
    }
}

fn line(event: &IssueReportEvent) -> Option<HistoryLine> {
    let what = match event.kind.as_str() {
        "created" => HistoryEvent::Created,
        "status" => HistoryEvent::Status {
            to: Status::from_wire(event.status.trim())?.wire().to_string(),
        },
        "reply" => HistoryEvent::Note {
            text: event.text.clone().filter(|t| !t.trim().is_empty())?,
        },
        _ => return None,
    };
    Some(at(event.created_at, actor_of(&event.actor), what))
}

fn at(when: DateTime<Utc>, actor: Actor, event: HistoryEvent) -> HistoryLine {
    HistoryLine {
        at: when,
        actor,
        event,
    }
}
