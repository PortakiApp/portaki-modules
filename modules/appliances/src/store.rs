//! AppliancesContent persistence via `host::repo` (+ in-memory for tests / `portaki dev`).

use portaki_sdk::host::repo::{self, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::AppliancesContent;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<AppliancesContent>> = const { RefCell::new(Vec::new()) };
}

pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

pub fn load_content() -> Result<Option<AppliancesContent>> {
    if in_memory_enabled() {
        let rows = TEST_ROWS.with(|rows| rows.borrow().clone());
        if let Some(row) = rows.into_iter().next() {
            return Ok(Some(row));
        }
    }

    let page =
        find::<AppliancesContent, AppliancesContent>(Query::<AppliancesContent>::new().limit(1))?;
    Ok(page.items.into_iter().next())
}

pub fn load_payload() -> Result<crate::content::AppliancesPayload> {
    let row = load_content()?;
    Ok(match row {
        Some(row) => crate::content::load_from_locale_slots(&row.content_fr, &row.content_en),
        None => crate::content::AppliancesPayload::default(),
    })
}

/// Persist canonical single-language JSON in `content_fr` (`content_en` cleared).
pub fn save_payload(payload: &crate::content::AppliancesPayload) -> Result<AppliancesContent> {
    let json = payload
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("appliances payload: {e}")))?;
    save_content_row(json, String::new())
}

pub fn save_content_row(content_fr: String, content_en: String) -> Result<AppliancesContent> {
    let now = time::now()?;
    if let Some(mut row) = load_content()? {
        row.content_fr = content_fr;
        row.content_en = content_en;
        row.updated_at = now;
        persist(row.clone())?;
        return Ok(row);
    }

    let row = AppliancesContent {
        id: Uuid::new_v4(),
        content_fr,
        content_en,
        created_at: now,
        updated_at: now,
    };
    persist(row.clone())?;
    Ok(row)
}

fn persist(row: AppliancesContent) -> Result<()> {
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
    let _ = repo::create::<AppliancesContent, AppliancesContent, AppliancesContent>(row)?;
    Ok(())
}
