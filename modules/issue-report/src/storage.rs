//! Issue report persistence via `host::repo`.

use portaki_sdk::host::repo::{self, eq, find, Query};
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
    };
    persist_row(row.clone())?;
    Ok(row)
}

fn persist_row(row: IssueReport) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow_mut().push(row));
        return Ok(());
    }
    let _ = repo::create::<IssueReport, IssueReport, IssueReport>(row)?;
    Ok(())
}
