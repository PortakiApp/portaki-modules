//! Lost/found report persistence via `host::repo`.

use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::LostFoundReport;

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

/// Inserts a new report for the stay.
pub fn create(
    stay_id: Uuid,
    kind: String,
    item_description: String,
    contact_hint: Option<String>,
    details: Option<String>,
) -> Result<LostFoundReport> {
    let now = time::now()?;
    let row = LostFoundReport {
        id: Uuid::new_v4(),
        stay_id,
        kind,
        item_description,
        contact_hint,
        details,
        created_at: now,
    };
    persist_row(row.clone())?;
    Ok(row)
}

fn persist_row(row: LostFoundReport) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow_mut().push(row));
        return Ok(());
    }
    let _ = repo::create::<LostFoundReport, LostFoundReport, LostFoundReport>(row)?;
    Ok(())
}
