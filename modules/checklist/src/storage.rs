//! Checklist persistence via `host::repo` (in-memory under tests / debug).

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{ChecklistCompletion, ChecklistItem};

use std::cell::RefCell;

thread_local! {
    static TEST_ITEMS: RefCell<Vec<ChecklistItem>> = const { RefCell::new(Vec::new()) };
    static TEST_COMPLETIONS: RefCell<Vec<ChecklistCompletion>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ITEMS.with(|rows| rows.borrow_mut().clear());
    TEST_COMPLETIONS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

/// Lists checklist items ordered by `sort_order`, then `created_at`.
pub fn list_items() -> Result<Vec<ChecklistItem>> {
    let mut items = if in_memory_enabled() {
        TEST_ITEMS.with(|rows| rows.borrow().clone())
    } else {
        let page = find::<ChecklistItem, ChecklistItem>(Query::<ChecklistItem>::new().limit(200))?;
        page.items
    };
    items.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(items)
}

/// Lists completion rows for a stay.
pub fn list_completions(stay_id: Uuid) -> Result<Vec<ChecklistCompletion>> {
    if in_memory_enabled() {
        return Ok(TEST_COMPLETIONS.with(|rows| {
            rows.borrow()
                .iter()
                .filter(|row| row.stay_id == stay_id)
                .cloned()
                .collect()
        }));
    }
    let page = find::<ChecklistCompletion, ChecklistCompletion>(
        Query::<ChecklistCompletion>::new()
            .r#where(eq("stay_id", stay_id))
            .limit(200),
    )?;
    Ok(page.items)
}

/// Returns completed item ids for a stay.
pub fn list_completed_item_ids(stay_id: Uuid) -> Result<Vec<Uuid>> {
    Ok(list_completions(stay_id)?
        .into_iter()
        .map(|row| row.item_id)
        .collect())
}

/// Replaces all property checklist items.
pub fn replace_items(items: Vec<(String, String, i32)>) -> Result<()> {
    replace_items_preserving_ids(
        items
            .into_iter()
            .map(|(label_fr, label_en, sort_order)| (None, label_fr, label_en, sort_order))
            .collect(),
    )
}

/// Replace items while keeping IDs when provided (preserves stay completions + other langs).
pub fn replace_items_preserving_ids(items: Vec<(Option<Uuid>, String, String, i32)>) -> Result<()> {
    let existing = list_items()?;
    let keep_ids: std::collections::HashSet<Uuid> =
        items.iter().filter_map(|(id, _, _, _)| *id).collect();
    for row in &existing {
        if !keep_ids.contains(&row.id) {
            delete_item(row.id)?;
        }
    }
    let now = time::now()?;
    for (index, (id, label_fr, label_en, sort_order)) in items.into_iter().enumerate() {
        let order = if sort_order == 0 && index > 0 {
            index as i32
        } else {
            sort_order
        };
        let item_id = id.unwrap_or_else(Uuid::new_v4);
        let created_at = existing
            .iter()
            .find(|row| row.id == item_id)
            .map(|row| row.created_at)
            .unwrap_or(now);
        persist_item(ChecklistItem {
            id: item_id,
            label_fr,
            label_en,
            sort_order: order,
            created_at,
        })?;
    }
    Ok(())
}

/// Marks an item complete for the stay (idempotent).
pub fn complete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    assert_item_exists(item_id)?;
    if list_completions(stay_id)?
        .iter()
        .any(|row| row.item_id == item_id)
    {
        return Ok(());
    }
    let now = time::now()?;
    persist_completion(ChecklistCompletion {
        id: Uuid::new_v4(),
        stay_id,
        item_id,
        completed_at: now,
    })
}

/// Removes a completion for the stay (idempotent).
pub fn uncomplete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    let rows = list_completions(stay_id)?;
    for row in rows {
        if row.item_id == item_id {
            delete_completion(row.id)?;
        }
    }
    Ok(())
}

/// Progress percentage 0–100 for the stay.
pub fn progress_percent(stay_id: Uuid) -> Result<u8> {
    let items = list_items()?;
    if items.is_empty() {
        return Ok(0);
    }
    let done = list_completed_item_ids(stay_id)?.len();
    Ok(((done * 100) / items.len()) as u8)
}

fn assert_item_exists(item_id: Uuid) -> Result<()> {
    let items = list_items()?;
    if items.iter().any(|item| item.id == item_id) {
        return Ok(());
    }
    Err(PortakiError::Host("item_not_found".to_string()))
}

fn persist_item(row: ChecklistItem) -> Result<()> {
    if in_memory_enabled() {
        TEST_ITEMS.with(|rows| {
            let mut guard = rows.borrow_mut();
            if let Some(index) = guard.iter().position(|item| item.id == row.id) {
                guard[index] = row;
            } else {
                guard.push(row);
            }
        });
        return Ok(());
    }
    let _ = repo::create::<ChecklistItem, ChecklistItem, ChecklistItem>(row)?;
    Ok(())
}

fn delete_item(id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        TEST_ITEMS.with(|rows| rows.borrow_mut().retain(|item| item.id != id));
        return Ok(());
    }
    repo::delete::<ChecklistItem>(id)?;
    Ok(())
}

fn persist_completion(row: ChecklistCompletion) -> Result<()> {
    if in_memory_enabled() {
        TEST_COMPLETIONS.with(|rows| rows.borrow_mut().push(row));
        return Ok(());
    }
    let _ = repo::create::<ChecklistCompletion, ChecklistCompletion, ChecklistCompletion>(row)?;
    Ok(())
}

fn delete_completion(id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        TEST_COMPLETIONS.with(|rows| rows.borrow_mut().retain(|row| row.id != id));
        return Ok(());
    }
    repo::delete::<ChecklistCompletion>(id)?;
    Ok(())
}

/// Seeds fixture items for integration tests.
#[allow(dead_code)]
pub fn seed_test_items(now: DateTime<Utc>, labels: &[(&str, &str)]) -> Vec<Uuid> {
    reset_test_store();
    let mut ids = Vec::new();
    for (index, (fr, en)) in labels.iter().enumerate() {
        let id = Uuid::new_v4();
        ids.push(id);
        let _ = persist_item(ChecklistItem {
            id,
            label_fr: (*fr).to_string(),
            label_en: (*en).to_string(),
            sort_order: index as i32,
            created_at: now,
        });
    }
    ids
}
