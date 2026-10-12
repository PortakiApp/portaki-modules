//! Issue report persistence via `host::repo`.

use chrono::{DateTime, Utc};
use portaki_sdk::files::FileRef;
use portaki_sdk::host::repo::{self, eq, find, gte, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{IssueReport, IssueReportEvent};
use crate::status;

use std::cell::RefCell;

thread_local! {
    static TEST_ROWS: RefCell<Vec<IssueReport>> = const { RefCell::new(Vec::new()) };
    static TEST_EVENTS: RefCell<Vec<IssueReportEvent>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_ROWS.with(|rows| rows.borrow_mut().clear());
    TEST_EVENTS.with(|rows| rows.borrow_mut().clear());
}

/// Who wrote a history line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Guest,
    Host,
}

impl Actor {
    fn as_str(self) -> &'static str {
        match self {
            Self::Guest => "guest",
            Self::Host => "host",
        }
    }
}

/// The guest and the stay's dates, taken when the report is sent.
#[derive(Debug, Clone, Default)]
pub struct GuestSnapshot {
    pub name: String,
    pub check_in: Option<DateTime<Utc>>,
    pub check_out: Option<DateTime<Utc>>,
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

/// Inserts a new report for the stay, and its « Signalé » history line.
pub fn create(
    stay_id: Uuid,
    category: String,
    summary: String,
    details: Option<String>,
    photo: Option<FileRef>,
    actor: Actor,
    guest: GuestSnapshot,
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
        photo: photo.map(|photo| photo.to_string()),
        status: status::NEW.to_string(),
        guest_name: guest.name,
        stay_check_in: guest.check_in,
        stay_check_out: guest.check_out,
    };
    persist_row(row.clone())?;
    add_event(&row, "created", actor, "", None, now)?;
    Ok(row)
}

/// Moves a report to `to`. `resolved_at` follows: set on resolving (a second call keeps the
/// first time), cleared on reopening. `changed` is false when the report already had that status.
pub fn set_status(id: Uuid, to: &'static str) -> Result<(IssueReport, bool)> {
    let mut row = find_by_id(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    if status::of(&row) == to {
        return Ok((row, false));
    }
    let now = time::now()?;
    row.status = to.to_string();
    row.resolved_at = if to == status::RESOLVED {
        row.resolved_at.or(Some(now))
    } else {
        None
    };
    persist_row(row.clone())?;
    add_event(&row, "status", Actor::Host, to, None, now)?;
    Ok((row, true))
}

/// Writes the host's reply into the report's history.
pub fn add_reply(report: &IssueReport, text: String) -> Result<IssueReportEvent> {
    add_event(report, "reply", Actor::Host, "", Some(text), time::now()?)
}

/// A report's history, oldest first.
pub fn events_for_report(report_id: Uuid) -> Result<Vec<IssueReportEvent>> {
    list_events("report_id", report_id, |event| event.report_id == report_id)
}

/// A stay's history (every report), oldest first — what the booklet reads.
pub fn events_for_stay(stay_id: Uuid) -> Result<Vec<IssueReportEvent>> {
    list_events("stay_id", stay_id, |event| event.stay_id == stay_id)
}

fn list_events(
    column: &str,
    id: Uuid,
    keep: impl Fn(&IssueReportEvent) -> bool,
) -> Result<Vec<IssueReportEvent>> {
    let mut rows = if in_memory_enabled() {
        TEST_EVENTS.with(|store| store.borrow().iter().filter(|e| keep(e)).cloned().collect())
    } else {
        find::<IssueReportEvent, IssueReportEvent>(
            Query::<IssueReportEvent>::new()
                .r#where(eq(column, id))
                .limit(500),
        )?
        .items
    };
    rows.sort_by_key(|event| event.created_at);
    Ok(rows)
}

fn add_event(
    report: &IssueReport,
    kind: &str,
    actor: Actor,
    status: &str,
    text: Option<String>,
    at: DateTime<Utc>,
) -> Result<IssueReportEvent> {
    let event = IssueReportEvent {
        id: Uuid::new_v4(),
        report_id: report.id,
        stay_id: report.stay_id,
        kind: kind.to_string(),
        actor: actor.as_str().to_string(),
        status: status.to_string(),
        text,
        created_at: at,
    };
    if in_memory_enabled() {
        TEST_EVENTS.with(|store| store.borrow_mut().push(event.clone()));
    } else {
        let _ =
            repo::create::<IssueReportEvent, IssueReportEvent, IssueReportEvent>(event.clone())?;
    }
    Ok(event)
}

pub fn find_by_id(id: Uuid) -> Result<Option<IssueReport>> {
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
