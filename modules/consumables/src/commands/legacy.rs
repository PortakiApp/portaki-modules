//! Les demandes d'avant `#[record]`, reprises une fois par le SDK : la première lecture du dépôt
//! encore vide les y range, id, statut, réponse et dates gardés.

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Actor, Choice, HistoryEvent, HistoryLine};
use uuid::Uuid;

use crate::{Entry, Level, Request, Status};

/// L'ancienne table, lue seulement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 5)]
pub struct ConsumableReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    pub item_label: String,
    pub level: String,
    pub note: Option<String>,
    #[serde(default)]
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub restocked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub host_reply: Option<String>,
    #[serde(default)]
    pub guest_name: String,
    #[serde(default)]
    pub stay_check_in: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stay_check_out: Option<DateTime<Utc>>,
}

// ponytail: one 500-row page, the platform's cap; older requests than that stay behind.
pub fn earlier_requests() -> Result<Vec<Entry>> {
    let rows = find::<ConsumableReport, ConsumableReport>(Query::new().limit(500))?.items;
    Ok(rows.into_iter().map(entry).collect())
}

fn entry(row: ConsumableReport) -> Entry {
    let status = Status::from_wire(&row.status).unwrap_or(Status::Open);
    let line = |at, event| HistoryLine {
        at,
        actor: Actor::Host,
        event,
    };
    let mut history = vec![HistoryLine {
        at: row.created_at,
        actor: Actor::Guest,
        event: HistoryEvent::Created,
    }];
    if let Some(text) = row.host_reply.filter(|r| !r.trim().is_empty()) {
        history.push(line(row.created_at, HistoryEvent::Note { text }));
    }
    if status != Status::Open {
        let at = row.restocked_at.unwrap_or(row.created_at);
        let to = status.wire().to_string();
        history.push(line(at, HistoryEvent::Status { to }));
    }
    Entry {
        id: row.id,
        item: Request {
            item_id: row.item_id.to_string(),
            item_label: row.item_label,
            level: Level::from_wire(&row.level).unwrap_or_default(),
            note: row.note.unwrap_or_default(),
            ..Request::default()
        },
        status,
        stay_id: Some(row.stay_id),
        created_at: row.created_at,
        actor: Actor::Guest,
        guest_name: row.guest_name,
        check_in: row.stay_check_in,
        check_out: row.stay_check_out,
        history,
    }
}
