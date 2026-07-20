//! RulesContent persistence via `host::repo` (+ in-memory for tests / `portaki dev`).

use portaki_sdk::host::repo::{self, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::RulesContent;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<RulesContent>> = const { RefCell::new(Vec::new()) };
}

/// Clears the in-memory store used by unit tests.
pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

/// Loads the single content row for the current property (0–1 rows).
pub fn load_content() -> Result<Option<RulesContent>> {
    if in_memory_enabled() {
        let rows = TEST_ROWS.with(|rows| rows.borrow().clone());
        if let Some(row) = rows.into_iter().next() {
            return Ok(Some(row));
        }
    }

    let page = find::<RulesContent, RulesContent>(Query::<RulesContent>::new().limit(1))?;
    Ok(page.items.into_iter().next())
}

/// Upserts content for the property.
pub fn save_content_row(content_fr: String, content_en: String) -> Result<RulesContent> {
    let now = time::now()?;
    if let Some(mut row) = load_content()? {
        row.content_fr = content_fr;
        row.content_en = content_en;
        row.updated_at = now;
        persist(row.clone())?;
        return Ok(row);
    }

    let row = RulesContent {
        id: Uuid::new_v4(),
        content_fr,
        content_en,
        created_at: now,
        updated_at: now,
    };
    persist(row.clone())?;
    Ok(row)
}

fn persist(row: RulesContent) -> Result<()> {
    if in_memory_enabled() {
        TEST_ROWS.with(|rows| {
            let mut guard = rows.borrow_mut();
            if let Some(index) = guard.iter().position(|item| item.id == row.id) {
                guard[index] = row;
            } else {
                guard.clear();
                guard.push(row);
            }
        });
        return Ok(());
    }
    // Runtime create is upsert on PK — same pattern as weather.
    let _ = repo::create::<RulesContent, RulesContent, RulesContent>(row)?;
    Ok(())
}
