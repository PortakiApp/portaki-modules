//! Les coches et les états de tâche d'avant `#[record]`, repris une fois par le SDK : la première
//! lecture du dépôt encore vide les y range, dates gardées.

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Actor, Entry, HistoryEvent, HistoryLine};
use uuid::Uuid;

use crate::{Tick, Ticked};

/// L'ancienne table des coches du voyageur, lue seulement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct ChecklistCompletion {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

/// L'ancienne table des états de tâche, lue seulement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct TaskItemState {
    pub id: Uuid,
    pub task_id: String,
    pub item_id: Uuid,
    pub done: bool,
    pub photo: Option<String>,
    pub done_at: Option<DateTime<Utc>>,
}

fn entry(
    id: Uuid,
    stay: Uuid,
    item: Ticked,
    done_at: Option<DateTime<Utc>>,
    actor: Actor,
) -> Entry<Ticked, Tick> {
    let at = done_at.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let mut history = vec![HistoryLine {
        at,
        actor,
        event: HistoryEvent::Created,
    }];
    if done_at.is_some() {
        history.push(HistoryLine {
            at,
            actor,
            event: HistoryEvent::Status { to: "done".into() },
        });
    }
    Entry {
        id,
        item,
        status: if done_at.is_some() {
            Tick::Done
        } else {
            Tick::Open
        },
        stay_id: Some(stay),
        created_at: at,
        actor,
        guest_name: String::new(),
        check_in: None,
        check_out: None,
        history,
    }
}

// ponytail: one 500-row page each, the platform's cap; older rows than that stay behind.
pub fn earlier_completions() -> Result<Vec<Entry<Ticked, Tick>>> {
    let rows = find::<ChecklistCompletion, ChecklistCompletion>(Query::new().limit(500))?.items;
    Ok(rows
        .into_iter()
        .map(|row| {
            let tick = Ticked {
                step: row.item_id.to_string(),
                ..Ticked::default()
            };
            entry(
                row.id,
                row.stay_id,
                tick,
                Some(row.completed_at),
                Actor::Guest,
            )
        })
        .collect())
}

pub fn earlier_task_states() -> Result<Vec<Entry<Ticked, Tick>>> {
    let rows = find::<TaskItemState, TaskItemState>(Query::new().limit(500))?.items;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let stay = Uuid::parse_str(row.task_id.split_once(':')?.1).ok()?;
            let tick = Ticked {
                step: row.item_id.to_string(),
                photo: row.photo.unwrap_or_default(),
                task: row.task_id,
                ..Ticked::default()
            };
            let done_at = row
                .done
                .then(|| row.done_at.unwrap_or(DateTime::<Utc>::UNIX_EPOCH));
            Some(entry(row.id, stay, tick, done_at, Actor::Host))
        })
        .collect())
}
