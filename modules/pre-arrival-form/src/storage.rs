//! Pre-arrival response persistence via `host::repo`.

use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::PreArrivalResponse;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<PreArrivalResponse>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

/// Finds the response for a stay, if any.
pub fn find_by_stay(stay_id: Uuid) -> Result<Option<PreArrivalResponse>> {
    if in_memory_enabled() {
        return Ok(TEST_ROWS.with(|rows| {
            rows.borrow()
                .iter()
                .find(|row| row.stay_id == stay_id)
                .cloned()
        }));
    }
    let page = find::<PreArrivalResponse, PreArrivalResponse>(
        Query::<PreArrivalResponse>::new()
            .r#where(eq("stay_id", stay_id))
            .limit(1),
    )?;
    Ok(page.items.into_iter().next())
}

/// Upserts the response for a stay.
#[allow(clippy::too_many_arguments)]
/// Ce que le voyageur a répondu, avant d'avoir un identifiant et une date.
///
/// Une structure et non neuf paramètres : la liste s'allonge à chaque question du formulaire, et
/// neuf `Option<String>` positionnels finissent par s'échanger sans que rien ne le dise.
#[derive(Debug, Clone, Default)]
pub struct ResponseDraft {
    pub arrival_time: Option<String>,
    pub occasion: Option<String>,
    pub allergies: Option<String>,
    pub guest_count: Option<String>,
    pub special_needs: Option<String>,
    pub id_document: Option<String>,
    pub transport: Option<String>,
    pub guest_message: Option<String>,
}

pub fn upsert(stay_id: Uuid, draft: ResponseDraft) -> Result<PreArrivalResponse> {
    let now = time::now()?;
    let id = match find_by_stay(stay_id)? {
        Some(existing) => {
            delete_row(existing.id)?;
            existing.id
        }
        None => Uuid::new_v4(),
    };
    let row = PreArrivalResponse {
        id,
        stay_id,
        arrival_time: draft.arrival_time,
        occasion: draft.occasion,
        allergies: draft.allergies,
        guest_count: draft.guest_count,
        special_needs: draft.special_needs,
        id_document: draft.id_document,
        transport: draft.transport,
        guest_message: draft.guest_message,
        completed_at: now,
    };
    persist_row(row.clone())?;
    Ok(row)
}

fn persist_row(row: PreArrivalResponse) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|rows| {
            let mut guard = rows.borrow_mut();
            if let Some(index) = guard.iter().position(|item| item.id == row.id) {
                guard[index] = row;
            } else if let Some(index) = guard.iter().position(|item| item.stay_id == row.stay_id) {
                guard[index] = row;
            } else {
                guard.push(row);
            }
        });
        return Ok(());
    }
    let _ = repo::create::<PreArrivalResponse, PreArrivalResponse, PreArrivalResponse>(row)?;
    Ok(())
}

fn delete_row(id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|rows| rows.borrow_mut().retain(|row| row.id != id));
        return Ok(());
    }
    repo::delete::<PreArrivalResponse>(id)?;
    Ok(())
}
