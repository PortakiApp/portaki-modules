//! Les coches du voyageur et l'état des tâches d'équipe : deux dépôts déclarés (`#[record]`), un
//! enregistrement par séjour et par étape, cochée ou non, avec la date de la coche.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::record::{self, mark};
use portaki_sdk::resource::{Actor, Entry, RecordDecl};
use uuid::Uuid;

use crate::{Completions, TaskStates, Tick, Ticked};

/// A guest step ticked for a stay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistCompletion {
    pub stay_id: Uuid,
    pub item_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

/// One item of a host task, ticked or not. Tasks are computed from the stays; only this is stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItemState {
    /// `<checklistId>:<stayId>`.
    pub task_id: String,
    pub item_id: Uuid,
    pub done: bool,
    /// `portaki-file:<uuid>` attached when ticked.
    pub photo: Option<String>,
    pub done_at: Option<DateTime<Utc>>,
}

fn marks(decl: RecordDecl, stay: Option<Uuid>) -> Result<Vec<Entry<Ticked, Tick>>> {
    match stay {
        Some(stay) => record::for_stay(decl, stay),
        None => record::list(decl),
    }
}

fn step(entry: &Entry<Ticked, Tick>) -> Option<Uuid> {
    Uuid::parse_str(&entry.item.step).ok()
}

fn ticked_at(entry: &Entry<Ticked, Tick>) -> Option<DateTime<Utc>> {
    (entry.status == Tick::Done)
        .then(|| entry.reached_at(Tick::Done))
        .flatten()
}

/// Completions of the property (`None`) or of one stay.
pub fn list_completions(stay_id: Option<Uuid>) -> Result<Vec<ChecklistCompletion>> {
    Ok(marks(Completions::DECL, stay_id)?
        .iter()
        .filter_map(|entry| {
            Some(ChecklistCompletion {
                stay_id: entry.stay_id?,
                item_id: step(entry)?,
                completed_at: ticked_at(entry)?,
            })
        })
        .collect())
}

/// Marks an item complete for the stay (idempotent).
pub fn complete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    if crate::domain::find_step(item_id)?.is_none() {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    let tick = Ticked {
        step: item_id.to_string(),
        ..Ticked::default()
    };
    mark::<Completions>(Completions::DECL, stay_id, tick, Tick::Done, Actor::Guest).map(|_| ())
}

/// Removes a completion for the stay (idempotent).
pub fn uncomplete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    if !list_completions(Some(stay_id))?
        .iter()
        .any(|row| row.item_id == item_id)
    {
        return Ok(());
    }
    let tick = Ticked {
        step: item_id.to_string(),
        ..Ticked::default()
    };
    mark::<Completions>(Completions::DECL, stay_id, tick, Tick::Open, Actor::Guest).map(|_| ())
}

/// Ticked states of the property (`None`) or of one task.
pub fn task_states(task_id: Option<&str>) -> Result<Vec<TaskItemState>> {
    let stay = task_id.and_then(stay_of);
    Ok(marks(TaskStates::DECL, stay)?
        .iter()
        .filter(|entry| task_id.is_none_or(|id| entry.item.task == id))
        .filter_map(|entry| {
            Some(TaskItemState {
                task_id: entry.item.task.clone(),
                item_id: step(entry)?,
                done: entry.status == Tick::Done,
                photo: Some(entry.item.photo.clone()).filter(|p| !p.is_empty()),
                done_at: ticked_at(entry),
            })
        })
        .collect())
}

/// Ticks or unticks one item of a task; unticking drops the photo.
pub fn set_task_item(
    task_id: &str,
    item_id: Uuid,
    done: bool,
    photo: Option<String>,
) -> Result<()> {
    let stay = stay_of(task_id).ok_or_else(|| PortakiError::Host("task_not_found".into()))?;
    let kept = task_states(Some(task_id))?
        .into_iter()
        .find(|row| row.item_id == item_id)
        .and_then(|row| row.photo);
    let tick = Ticked {
        step: item_id.to_string(),
        photo: if done { photo.or(kept) } else { None }.unwrap_or_default(),
        task: task_id.to_string(),
        ..Ticked::default()
    };
    let to = if done { Tick::Done } else { Tick::Open };
    mark::<TaskStates>(TaskStates::DECL, stay, tick, to, Actor::Host).map(|_| ())
}

/// The stay of `<checklistId>:<stayId>`.
fn stay_of(task_id: &str) -> Option<Uuid> {
    Uuid::parse_str(task_id.split_once(':')?.1).ok()
}
