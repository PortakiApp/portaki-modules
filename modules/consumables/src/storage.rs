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
        return Ok(TEST_ITEMS.with(|store| store.borrow().iter().find(|row| row.id == id).cloned()));
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
        restocked_at: None,
        host_reply: None,
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

/// Une ligne du catalogue telle que la commande la reconstruit.
///
/// Un gabarit nommé et non un n-uplet : trois des champs sont des entiers ou des chaînes
/// interchangeables, et l'ordre finissait par être le seul garde-fou.
#[derive(Debug, Clone)]
pub struct ItemDraft {
    pub id: Option<Uuid>,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
    pub low_threshold: i32,
    pub emoji: String,
}

/// Replace items while keeping IDs when provided (preserves reports + other langs).
pub fn replace_items_preserving_ids(items: Vec<ItemDraft>) -> Result<()> {
    let existing = list_items()?;
    let keep_ids: std::collections::HashSet<Uuid> =
        items.iter().filter_map(|draft| draft.id).collect();
    for row in &existing {
        if !keep_ids.contains(&row.id) {
            delete_item(row.id)?;
        }
    }
    let now = time::now()?;
    for (index, draft) in items.into_iter().enumerate() {
        let order = if draft.sort_order == 0 && index > 0 {
            index as i32
        } else {
            draft.sort_order
        };
        let item_id = draft.id.unwrap_or_else(Uuid::new_v4);
        let created_at = existing
            .iter()
            .find(|row| row.id == item_id)
            .map(|row| row.created_at)
            .unwrap_or(now);
        persist_item(ConsumableItem {
            id: item_id,
            label_fr: draft.label_fr,
            label_en: draft.label_en,
            sort_order: order,
            low_threshold: draft.low_threshold,
            emoji: draft.emoji,
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
            emoji: String::new(),
            created_at: now,
        });
    }
    ids
}

/// Le délai de réapprovisionnement que l'hôte annonce, dans la langue du voyageur (§2.5).
///
/// En KV et non en entité : c'est une valeur unique par logement, pas une ligne de catalogue, et
/// une table d'une seule ligne se migrerait pour rien. Pas de TTL — un réglage d'hôte ne périme
/// pas.
pub mod restock_delay {
    use portaki_sdk::contracts::i18n::I18nText;
    use portaki_sdk::host::kv;

    const KEY: &str = "restock_delay";

    /// Ce que l'hôte a écrit, ou `None` quand il n'a rien annoncé — le livret dit alors sa phrase
    /// générique plutôt qu'un délai inventé.
    pub fn read() -> Option<I18nText> {
        let bytes = kv::get(KEY).ok()??;
        let text: I18nText = serde_json::from_slice(&bytes).ok()?;
        (!text.is_blank()).then_some(text)
    }

    /// Un délai vide efface le réglage : l'hôte doit pouvoir reprendre sa promesse.
    pub fn write(text: &I18nText) -> portaki_sdk::Result<()> {
        if text.is_blank() {
            return kv::delete(KEY);
        }
        let bytes = serde_json::to_vec(text).map_err(|error| {
            portaki_sdk::PortakiError::Storage(format!("{KEY} serialize: {error}"))
        })?;
        kv::set(KEY, &bytes, None)
    }
}

/// Les réglages des demandes (spec Consommables §2.2) : le voyageur peut-il demander, et combien
/// de fois par séjour. En KV comme le délai, pour la même raison : une valeur par logement.
pub mod settings {
    use portaki_sdk::host::kv;
    use serde::{Deserialize, Serialize};

    const KEY: &str = "settings";
    /// Demandes par séjour : le défaut et les bornes.
    pub const DEFAULT_MAX_REQUESTS: u32 = 5;
    pub const MIN_MAX_REQUESTS: u32 = 1;
    pub const MAX_MAX_REQUESTS: u32 = 20;

    #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Settings {
        /// Absent : oui, comme avant ce réglage.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub requests_enabled: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub max_requests: Option<u32>,
    }

    impl Settings {
        pub fn requests_enabled(&self) -> bool {
            self.requests_enabled.unwrap_or(true)
        }

        pub fn max_requests(&self) -> u32 {
            self.max_requests.map_or(DEFAULT_MAX_REQUESTS, |n| {
                n.clamp(MIN_MAX_REQUESTS, MAX_MAX_REQUESTS)
            })
        }
    }

    pub fn read() -> Settings {
        kv::get(KEY)
            .ok()
            .flatten()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn write(settings: &Settings) -> portaki_sdk::Result<()> {
        let bytes = serde_json::to_vec(settings).map_err(|error| {
            portaki_sdk::PortakiError::Storage(format!("{KEY} serialize: {error}"))
        })?;
        kv::set(KEY, &bytes, None)
    }
}
