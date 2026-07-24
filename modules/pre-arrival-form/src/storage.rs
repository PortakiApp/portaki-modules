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
pub fn upsert(
    stay_id: Uuid,
    arrival_time: Option<String>,
    occasion: Option<String>,
    allergies: Option<String>,
    guest_count: Option<String>,
    special_needs: Option<String>,
    id_document: Option<String>,
    guest_message: Option<String>,
) -> Result<PreArrivalResponse> {
    let now = time::now()?;
    if let Some(existing) = find_by_stay(stay_id)? {
        delete_row(existing.id)?;
        let row = PreArrivalResponse {
            id: existing.id,
            stay_id,
            arrival_time,
            occasion,
            allergies,
            guest_count,
            special_needs,
            id_document,
            guest_message,
            completed_at: now,
        };
        persist_row(row.clone())?;
        return Ok(row);
    }
    let row = PreArrivalResponse {
        id: Uuid::new_v4(),
        stay_id,
        arrival_time,
        occasion,
        allergies,
        guest_count,
        special_needs,
        id_document,
        guest_message,
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
