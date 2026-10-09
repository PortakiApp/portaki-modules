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

/// Sorts newest first, and reads every status in the current set ([`status::normalize`]).
fn sort_newest_first(rows: &mut [LostFoundReport]) {
    rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
    rows.iter_mut().for_each(normalize);
}

fn normalize(row: &mut LostFoundReport) {
    row.status = status::normalize(&row.status, &row.kind).to_string();
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

/// Un signalement à écrire.
///
/// Une structure plutôt que des paramètres positionnels : `contact_hint`, `details` et maintenant
/// `return_address` sont trois `Option<String>` voisines, et les intervertir à l'appel compilerait
/// sans bruit.
#[derive(Debug, Clone, Default)]
pub struct ReportDraft {
    pub stay_id: Uuid,
    pub kind: String,
    pub item_description: String,
    pub contact_hint: Option<String>,
    pub details: Option<String>,
    pub return_address: Option<String>,
    pub category: Option<String>,
    pub room: Option<String>,
    pub photo: Option<String>,
    pub return_choice: Option<String>,
    pub status: String,
}

/// Inserts a new report for the stay.
pub fn create(draft: ReportDraft) -> Result<LostFoundReport> {
    let now = time::now()?;
    let row = LostFoundReport {
        id: Uuid::new_v4(),
        stay_id: draft.stay_id,
        kind: draft.kind,
        item_description: draft.item_description,
        contact_hint: draft.contact_hint,
        details: draft.details,
        return_address: draft.return_address,
        category: draft.category,
        room: draft.room,
        photo: draft.photo,
        return_choice: draft.return_choice,
        status: if draft.status.trim().is_empty() {
            status::DEFAULT.to_string()
        } else {
            draft.status
        },
        created_at: now,
    };
    persist_row(row.clone())?;
    Ok(row)
}

/// Loads a report by id.
pub fn find_by_id(id: Uuid) -> Result<Option<LostFoundReport>> {
    let mut row = if in_memory_enabled() {
        TEST_ROWS.with(|store| store.borrow().iter().find(|row| row.id == id).cloned())
    } else {
        repo::find_by_id::<LostFoundReport, LostFoundReport>(id)?
    };
    row.iter_mut().for_each(normalize);
    Ok(row)
}

/// Moves an existing report to `to` (host), refused when [`status::can_move`] says no.
pub fn update_status(id: Uuid, to: &str) -> Result<LostFoundReport> {
    let mut row = find_by_id(id)?.ok_or_else(|| PortakiError::Host("report_not_found".into()))?;
    if !status::can_move(&row.status, to) {
        return Err(PortakiError::Host(format!(
            "invalid_transition:{}->{to}",
            row.status
        )));
    }
    row.status = to.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;

    /// Rows written before the six statuses still read, as the nearest new status.
    #[test]
    fn legacy_rows_read_in_the_new_set() {
        reset_test_store();
        let stay_id = Uuid::new_v4();
        MockContext::host().run(|_| {
            for (kind, old) in [
                ("lost", "to_collect"),
                ("found", "to_collect"),
                ("lost", "sent"),
                ("found", "returned"),
            ] {
                create(ReportDraft {
                    stay_id,
                    kind: kind.into(),
                    item_description: old.into(),
                    status: old.into(),
                    ..ReportDraft::default()
                })
                .expect("create");
            }
            let read = |old: &str, kind: &str| {
                list_by_stay(stay_id)
                    .unwrap()
                    .into_iter()
                    .find(|row| row.item_description == old && row.kind == kind)
                    .unwrap()
                    .status
            };
            assert_eq!(read("to_collect", "lost"), "declared");
            assert_eq!(read("to_collect", "found"), "found");
            assert_eq!(read("sent", "lost"), "shipped");
            assert_eq!(read("returned", "found"), "picked_up");

            // A legacy « Envoyé » row is final: it cannot go back to « Trouvé ».
            let sent = list_by_stay(stay_id)
                .unwrap()
                .into_iter()
                .find(|row| row.item_description == "sent")
                .unwrap();
            assert_eq!(find_by_id(sent.id).unwrap().unwrap().status, "shipped");
            assert!(update_status(sent.id, "found").is_err());
        });
    }
}
