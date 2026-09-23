//! Issue report persistence via `host::repo`.

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, gte, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::IssueReport;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<IssueReport>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

fn sort_newest_first(rows: &mut [IssueReport]) {
    rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
}

/// Lists reports for a stay, newest first.
pub fn list_by_stay(stay_id: Uuid) -> Result<Vec<IssueReport>> {
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
        let page = find::<IssueReport, IssueReport>(
            Query::<IssueReport>::new()
                .r#where(eq("stay_id", stay_id))
                .limit(200),
        )?;
        page.items
    };
    sort_newest_first(&mut rows);
    Ok(rows)
}

/// Lists the most recent reports for the property (host scope), newest first, max 20.
pub fn list_recent() -> Result<Vec<IssueReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow().clone())
    } else {
        let page = find::<IssueReport, IssueReport>(Query::<IssueReport>::new().limit(200))?;
        page.items
    };
    sort_newest_first(&mut rows);
    rows.truncate(20);
    Ok(rows)
}

/// Lists the property's reports created at or after `since` (host stats window).
pub fn list_since(since: DateTime<Utc>) -> Result<Vec<IssueReport>> {
    if in_memory_enabled() {
        return Ok(TEST_ROWS.with(|store| {
            store
                .borrow()
                .iter()
                .filter(|row| row.created_at >= since)
                .cloned()
                .collect()
        }));
    }
    // ponytail: one 1000-row page caps the window; move to repo::count per category if a property outgrows it.
    let page = find::<IssueReport, IssueReport>(
        Query::<IssueReport>::new()
            .r#where(gte("created_at", since))
            .limit(1000),
    )?;
    Ok(page.items)
}

/// Inserts a new report for the stay.
pub fn create(
    stay_id: Uuid,
    category: String,
    summary: String,
    details: Option<String>,
) -> Result<IssueReport> {
    let now = time::now()?;
    let row = IssueReport {
        id: Uuid::new_v4(),
        stay_id,
        category,
        summary,
        details,
        created_at: now,
        resolved_at: None,
    };
    persist_row(row.clone())?;
    Ok(row)
}

/// Marks a report resolved; a second call keeps the first resolution time.
pub fn resolve(id: Uuid) -> Result<IssueReport> {
    let mut row = find_by_id(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    if row.resolved_at.is_none() {
        row.resolved_at = Some(time::now()?);
        persist_row(row.clone())?;
    }
    Ok(row)
}

fn find_by_id(id: Uuid) -> Result<Option<IssueReport>> {
    if in_memory_enabled() {
        return Ok(TEST_ROWS.with(|store| store.borrow().iter().find(|row| row.id == id).cloned()));
    }
    repo::find_by_id::<IssueReport, IssueReport>(id)
}

fn persist_row(row: IssueReport) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|store| {
            let mut rows = store.borrow_mut();
            match rows.iter().position(|existing| existing.id == row.id) {
                Some(index) => rows[index] = row,
                None => rows.push(row),
            }
        });
        return Ok(());
    }
    // Gateway `repo_create` upserts on primary key (`id`).
    let _ = repo::create::<IssueReport, IssueReport, IssueReport>(row)?;
    Ok(())
}
