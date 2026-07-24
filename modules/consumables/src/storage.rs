//! Consumables persistence via `host::repo` (in-memory under tests / debug).

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{ConsumableItem, ConsumableReport};
use crate::status;

use std::cell::RefCell;

thread_local! {
    static TEST_ITEMS: RefCell<Vec<ConsumableItem>> = const { RefCell::new(Vec::new()) };
    static TEST_REPORTS: RefCell<Vec<ConsumableReport>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ITEMS.with(|rows| rows.borrow_mut().clear());
    TEST_REPORTS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

/// Lists catalog items ordered by `sort_order`, then `created_at`.
pub fn list_items() -> Result<Vec<ConsumableItem>> {
    let mut items = if in_memory_enabled() {
        TEST_ITEMS.with(|rows| rows.borrow().clone())
    } else {
        let page =
            find::<ConsumableItem, ConsumableItem>(Query::<ConsumableItem>::new().limit(200))?;
        page.items
    };
    items.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(items)
}

/// Loads a catalog item by id.
pub fn find_item(id: Uuid) -> Result<Option<ConsumableItem>> {
    if in_memory_enabled() {
        return Ok(TEST_ITEMS.with(|store| {
            store.borrow().iter().find(|row| row.id == id).cloned()
        }));
    }
    repo::find_by_id::<ConsumableItem, ConsumableItem>(id)
}

fn sort_newest_first(rows: &mut [ConsumableReport]) {
    rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
}

/// Lists reports for a stay, newest first.
pub fn list_by_stay(stay_id: Uuid) -> Result<Vec<ConsumableReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_REPORTS.with(|store| {
            store
                .borrow()
                .iter()
                .filter(|row| row.stay_id == stay_id)
                .cloned()
                .collect()
        })
    } else {
        let page = find::<ConsumableReport, ConsumableReport>(
            Query::<ConsumableReport>::new()
                .r#where(eq("stay_id", stay_id))
                .limit(200),
        )?;
        page.items
    };
    sort_newest_first(&mut rows);
    Ok(rows)
}

/// Lists the most recent reports for the property (host scope), newest first, max 40.
pub fn list_recent() -> Result<Vec<ConsumableReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_REPORTS.with(|store| store.borrow().clone())
    } else {
        let page =
            find::<ConsumableReport, ConsumableReport>(Query::<ConsumableReport>::new().limit(200))?;
        page.items
    };
    sort_newest_first(&mut rows);
    rows.truncate(40);
    Ok(rows)
}

/// Open (not restocked) reports, newest first, max 40.
pub fn list_open() -> Result<Vec<ConsumableReport>> {
    let mut rows = list_recent()?
        .into_iter()
        .filter(|row| row.status == status::DEFAULT)
        .collect::<Vec<_>>();
    sort_newest_first(&mut rows);
    rows.truncate(40);
    Ok(rows)
}

/// Count of open reports for the property.
pub fn count_open() -> Result<usize> {
    Ok(list_open()?.len())
}

/// Inserts a guest shortage report.
pub fn create_report(
    stay_id: Uuid,
    item_id: Uuid,
    item_label: String,
    level: String,
    note: Option<String>,
) -> Result<ConsumableReport> {
    let now = time::now()?;
    let row = ConsumableReport {
        id: Uuid::new_v4(),
        stay_id,
        item_id,
        item_label,
        level,
        note,
        status: status::DEFAULT.to_string(),
        created_at: now,
    };
    persist_report(row.clone())?;
    Ok(row)
}

/// Loads a report by id.
pub fn find_report(id: Uuid) -> Result<Option<ConsumableReport>> {
    if in_memory_enabled() {
        return Ok(TEST_REPORTS.with(|store| {
            store.borrow().iter().find(|row| row.id == id).cloned()
        }));
    }
    repo::find_by_id::<ConsumableReport, ConsumableReport>(id)
}

/// Updates the workflow status of an existing report (host).
pub fn update_status(id: Uuid, status: String) -> Result<ConsumableReport> {
    let mut row = find_report(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    row.status = if status.trim().is_empty() {
        status::DEFAULT.to_string()
    } else {
        status
    };
    persist_report(row.clone())?;
    Ok(row)
}

/// Replace items while keeping IDs when provided (preserves reports + other langs).
pub fn replace_items_preserving_ids(
    items: Vec<(Option<Uuid>, String, String, i32, i32)>,
) -> Result<()> {
    let existing = list_items()?;
    let keep_ids: std::collections::HashSet<Uuid> =
        items.iter().filter_map(|(id, _, _, _, _)| *id).collect();
    for row in &existing {
        if !keep_ids.contains(&row.id) {
            delete_item(row.id)?;
        }
    }
    let now = time::now()?;
    for (index, (id, label_fr, label_en, sort_order, low_threshold)) in items.into_iter().enumerate()
    {
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
        persist_item(ConsumableItem {
            id: item_id,
            label_fr,
            label_en,
            sort_order: order,
            low_threshold,
            created_at,
        })?;
    }
    Ok(())
}

fn persist_item(row: ConsumableItem) -> Result<()> {
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
    let _ = repo::create::<ConsumableItem, ConsumableItem, ConsumableItem>(row)?;
    Ok(())
}

fn delete_item(id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        TEST_ITEMS.with(|rows| rows.borrow_mut().retain(|item| item.id != id));
        return Ok(());
    }
    repo::delete::<ConsumableItem>(id)?;
    Ok(())
}

fn persist_report(row: ConsumableReport) -> Result<()> {
    if in_memory_enabled() {
        TEST_REPORTS.with(|store| {
            let mut rows = store.borrow_mut();
            if let Some(index) = rows.iter().position(|existing| existing.id == row.id) {
                rows[index] = row;
            } else {
                rows.push(row);
            }
        });
        return Ok(());
    }
    let _ = repo::create::<ConsumableReport, ConsumableReport, ConsumableReport>(row)?;
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
        let _ = persist_item(ConsumableItem {
            id,
            label_fr: (*fr).to_string(),
            label_en: (*en).to_string(),
            sort_order: index as i32,
            low_threshold: 0,
            created_at: now,
        });
    }
    ids
}
