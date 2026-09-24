//! Lost/found report persistence via `host::repo`.

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, gte, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::LostFoundReport;
use crate::status;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<LostFoundReport>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

fn sort_newest_first(rows: &mut [LostFoundReport]) {
    rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
}

/// Lists reports for a stay, newest first.
pub fn list_by_stay(stay_id: Uuid) -> Result<Vec<LostFoundReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_ROWS.with(|store| {
            store
                .borrow()
                .iter()
                .filter(|row| row.stay_id == stay_id)
                .cloned()
                .collect()
        })
    } else {
        let page = find::<LostFoundReport, LostFoundReport>(
            Query::<LostFoundReport>::new()
                .r#where(eq("stay_id", stay_id))
                .limit(200),
        )?;
        page.items
    };
    sort_newest_first(&mut rows);
    Ok(rows)
}

/// Lists the most recent reports for the property (host scope), newest first, max 20.
pub fn list_recent() -> Result<Vec<LostFoundReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow().clone())
    } else {
        let page =
            find::<LostFoundReport, LostFoundReport>(Query::<LostFoundReport>::new().limit(200))?;
        page.items
    };
    sort_newest_first(&mut rows);
    rows.truncate(20);
    Ok(rows)
}

/// Lists the property's reports created at or after `since` (host stats window), newest first.
pub fn list_since(since: DateTime<Utc>) -> Result<Vec<LostFoundReport>> {
    let mut rows: Vec<LostFoundReport> = if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow().clone())
    } else {
        // ponytail: one 1000-row page caps the window; paginate if a property outgrows it.
        find::<LostFoundReport, LostFoundReport>(
            Query::<LostFoundReport>::new()
                .r#where(gte("created_at", since))
                .limit(1000),
        )?
        .items
    };
    rows.retain(|row| row.created_at >= since);
    sort_newest_first(&mut rows);
    Ok(rows)
}

/// Inserts a new report for the stay.
pub fn create(
    stay_id: Uuid,
    kind: String,
    item_description: String,
    contact_hint: Option<String>,
    details: Option<String>,
    status: String,
) -> Result<LostFoundReport> {
    let now = time::now()?;
    let row = LostFoundReport {
        id: Uuid::new_v4(),
        stay_id,
        kind,
        item_description,
        contact_hint,
        details,
        status: if status.trim().is_empty() {
            status::DEFAULT.to_string()
        } else {
            status
        },
        created_at: now,
    };
    persist_row(row.clone())?;
    Ok(row)
}

/// Loads a report by id.
pub fn find_by_id(id: Uuid) -> Result<Option<LostFoundReport>> {
    if in_memory_enabled() {
        return Ok(TEST_ROWS.with(|store| store.borrow().iter().find(|row| row.id == id).cloned()));
    }
    repo::find_by_id::<LostFoundReport, LostFoundReport>(id)
}

/// Updates the workflow status of an existing report (host).
pub fn update_status(id: Uuid, status: String) -> Result<LostFoundReport> {
    let mut row = find_by_id(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    row.status = if status.trim().is_empty() {
        status::DEFAULT.to_string()
    } else {
        status
    };
    persist_row(row.clone())?;
    Ok(row)
}

fn persist_row(row: LostFoundReport) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|store| {
            let mut rows = store.borrow_mut();
            if let Some(index) = rows.iter().position(|existing| existing.id == row.id) {
                rows[index] = row;
            } else {
                rows.push(row);
            }
        });
        return Ok(());
    }
    // Gateway `repo_create` upserts on primary key (`id`).
    let _ = repo::create::<LostFoundReport, LostFoundReport, LostFoundReport>(row)?;
    Ok(())
}
