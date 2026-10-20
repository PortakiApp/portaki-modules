//! Consumables persistence via `host::repo` (in-memory under tests / debug).

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::ConsumableReport;
use crate::status;

use std::cell::RefCell;

thread_local! {
    static TEST_REPORTS: RefCell<Vec<ConsumableReport>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_REPORTS.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
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
    let mut rows = list_all()?;
    rows.truncate(40);
    Ok(rows)
}

/// Every report of the property, newest first (host stats).
pub fn list_all() -> Result<Vec<ConsumableReport>> {
    let mut rows = if in_memory_enabled() {
        TEST_REPORTS.with(|store| store.borrow().clone())
    } else {
        // ponytail: one 1000-row page; paginate if a property outgrows it.
        find::<ConsumableReport, ConsumableReport>(Query::<ConsumableReport>::new().limit(1000))?
            .items
    };
    sort_newest_first(&mut rows);
    Ok(rows)
}

/// Pending (to handle or planned, not yet restocked) reports, newest first, max 40.
pub fn list_open() -> Result<Vec<ConsumableReport>> {
    let mut rows = list_all()?
        .into_iter()
        .filter(|row| status::is_pending(&row.status))
        .collect::<Vec<_>>();
    rows.truncate(40);
    Ok(rows)
}

/// Count of open reports for the property.
pub fn count_open() -> Result<usize> {
    Ok(list_open()?.len())
}

/// Who sent a request, as the booklet knew it then: the host's table shows it without asking the
/// platform again.
#[derive(Debug, Clone, Default)]
pub struct GuestSnapshot {
    pub name: String,
    pub check_in: Option<DateTime<Utc>>,
    pub check_out: Option<DateTime<Utc>>,
}

/// What the guest asks for one product: the level and the note.
pub struct Asked {
    pub level: String,
    pub note: Option<String>,
}

/// Inserts a guest shortage report.
pub fn create_report(
    stay_id: Uuid,
    item_id: Uuid,
    item_label: String,
    asked: &Asked,
    guest: &GuestSnapshot,
) -> Result<ConsumableReport> {
    let now = time::now()?;
    let row = ConsumableReport {
        id: Uuid::new_v4(),
        stay_id,
        item_id,
        item_label,
        level: asked.level.clone(),
        note: asked.note.clone(),
        status: status::DEFAULT.to_string(),
        created_at: now,
        restocked_at: None,
        host_reply: None,
        guest_name: guest.name.clone(),
        stay_check_in: guest.check_in,
        stay_check_out: guest.check_out,
    };
    persist_report(row.clone())?;
    Ok(row)
}

/// Loads a report by id.
pub fn find_report(id: Uuid) -> Result<Option<ConsumableReport>> {
    if in_memory_enabled() {
        return Ok(
            TEST_REPORTS.with(|store| store.borrow().iter().find(|row| row.id == id).cloned())
        );
    }
    repo::find_by_id::<ConsumableReport, ConsumableReport>(id)
}

/// Updates the workflow status of an existing report (host), and its reply when one is given
/// (`Some("")` clears it, `None` keeps it).
pub fn update_status(
    id: Uuid,
    status: String,
    host_reply: Option<String>,
) -> Result<ConsumableReport> {
    let mut row = find_report(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    row.status = if status.trim().is_empty() {
        status::DEFAULT.to_string()
    } else {
        status
    };
    // Seul « Livré » date un réassort : « Prévu » n'a encore rien remis dans le placard.
    row.restocked_at = if row.status == status::RESTOCKED {
        row.restocked_at.or(Some(time::now()?))
    } else {
        None
    };
    if let Some(reply) = host_reply {
        let reply = reply.trim();
        row.host_reply = (!reply.is_empty()).then(|| reply.to_string());
    }
    persist_report(row.clone())?;
    Ok(row)
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
