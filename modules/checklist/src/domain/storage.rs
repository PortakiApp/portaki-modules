//! Les coches du voyageur et l'état des tâches d'équipe, via `host::repo` (en mémoire sous test).

use std::cell::RefCell;
use std::thread::LocalKey;

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::{ChecklistCompletion, TaskItemState};

thread_local! {
    static TEST_COMPLETIONS: RefCell<Vec<ChecklistCompletion>> = const { RefCell::new(Vec::new()) };
    static TEST_TASK_STATES: RefCell<Vec<TaskItemState>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_COMPLETIONS.with(|rows| rows.borrow_mut().clear());
    TEST_TASK_STATES.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

trait Row: Clone + Serialize + DeserializeOwned {
    fn id(&self) -> Uuid;
}

macro_rules! row {
    ($($ty:ty),*) => {$(impl Row for $ty { fn id(&self) -> Uuid { self.id } })*};
}
row!(ChecklistCompletion, TaskItemState);

type Table<T> = LocalKey<RefCell<Vec<T>>>;

/// Rows of the property scope, or those whose `field` equals `value`.
fn select<T: Row>(table: &'static Table<T>, filter: Option<(&str, String)>) -> Result<Vec<T>> {
    if in_memory_enabled() {
        return Ok(table.with(|rows| rows.borrow().clone()));
    }
    let mut query = Query::<T>::new().limit(1000);
    if let Some((field, value)) = filter {
        query = query.r#where(eq(field, value));
    }
    // ponytail: one 1000-row page; paginate if a property outgrows it.
    Ok(find::<T, T>(query)?.items)
}

fn upsert<T: Row>(table: &'static Table<T>, row: T) -> Result<()> {
    if in_memory_enabled() {
        table.with(|rows| {
            let mut rows = rows.borrow_mut();
            match rows.iter().position(|existing| existing.id() == row.id()) {
                Some(index) => rows[index] = row,
                None => rows.push(row),
            }
        });
        return Ok(());
    }
    // Gateway `repo_create` upserts on primary key (`id`).
    let _ = repo::create::<T, T, T>(row)?;
    Ok(())
}

fn remove<T: Row>(table: &'static Table<T>, id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        table.with(|rows| rows.borrow_mut().retain(|row| row.id() != id));
        return Ok(());
    }
    repo::delete::<T>(id)?;
    Ok(())
}

// --- Guest completions ----------------------------------------------------------------------

/// Completions of the property (`None`) or of one stay.
pub fn list_completions(stay_id: Option<Uuid>) -> Result<Vec<ChecklistCompletion>> {
    let rows = select(
        &TEST_COMPLETIONS,
        stay_id.map(|id| ("stay_id", id.to_string())),
    )?;
    Ok(rows
        .into_iter()
        .filter(|row| stay_id.is_none_or(|id| row.stay_id == id))
        .collect())
}

/// Marks an item complete for the stay (idempotent).
pub fn complete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    if super::find_step(item_id)?.is_none() {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    if list_completions(Some(stay_id))?
        .iter()
        .any(|row| row.item_id == item_id)
    {
        return Ok(());
    }
    upsert(
        &TEST_COMPLETIONS,
        ChecklistCompletion {
            id: Uuid::new_v4(),
            stay_id,
            item_id,
            completed_at: time::now()?,
        },
    )
}

/// Removes a completion for the stay (idempotent).
pub fn uncomplete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    for row in list_completions(Some(stay_id))? {
        if row.item_id == item_id {
            remove(&TEST_COMPLETIONS, row.id)?;
        }
    }
    Ok(())
}

// --- Host task state ------------------------------------------------------------------------

/// Ticked states of the property (`None`) or of one task.
pub fn task_states(task_id: Option<&str>) -> Result<Vec<TaskItemState>> {
    let rows = select(
        &TEST_TASK_STATES,
        task_id.map(|id| ("task_id", id.to_string())),
    )?;
    Ok(rows
        .into_iter()
        .filter(|row| task_id.is_none_or(|id| row.task_id == id))
        .collect())
}

/// Ticks or unticks one item of a task; unticking drops the photo.
pub fn set_task_item(
    task_id: &str,
    item_id: Uuid,
    done: bool,
    photo: Option<String>,
    now: DateTime<Utc>,
) -> Result<()> {
    let existing = task_states(Some(task_id))?
        .into_iter()
        .find(|row| row.item_id == item_id);
    upsert(
        &TEST_TASK_STATES,
        TaskItemState {
            id: existing.as_ref().map_or_else(Uuid::new_v4, |row| row.id),
            task_id: task_id.to_string(),
            item_id,
            done,
            photo: if done {
                photo.or_else(|| existing.and_then(|row| row.photo))
            } else {
                None
            },
            done_at: done.then_some(now),
        },
    )
}
