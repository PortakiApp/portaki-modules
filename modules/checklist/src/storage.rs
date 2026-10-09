//! Checklist persistence via `host::repo` (in-memory under tests / debug).

use std::cell::RefCell;
use std::thread::LocalKey;

use chrono::{DateTime, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::{kv, time};
use portaki_sdk::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::{Checklist, ChecklistCompletion, ChecklistItem, TaskItemState};
use crate::labels::{self, Labels};
use crate::lists::{self, Template};

thread_local! {
    static TEST_LISTS: RefCell<Vec<Checklist>> = const { RefCell::new(Vec::new()) };
    static TEST_ITEMS: RefCell<Vec<ChecklistItem>> = const { RefCell::new(Vec::new()) };
    static TEST_COMPLETIONS: RefCell<Vec<ChecklistCompletion>> = const { RefCell::new(Vec::new()) };
    static TEST_TASK_STATES: RefCell<Vec<TaskItemState>> = const { RefCell::new(Vec::new()) };
}

/// Clears in-memory rows used by unit tests.
pub fn reset_test_store() {
    TEST_LISTS.with(|rows| rows.borrow_mut().clear());
    TEST_ITEMS.with(|rows| rows.borrow_mut().clear());
    TEST_COMPLETIONS.with(|rows| rows.borrow_mut().clear());
    TEST_TASK_STATES.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

trait Row: Clone + Serialize + DeserializeOwned {
    fn id(&self) -> Uuid;
}

macro_rules! row {
    ($($ty:ty),*) => {$(impl Row for $ty { fn id(&self) -> Uuid { self.id } })*};
}
row!(Checklist, ChecklistItem, ChecklistCompletion, TaskItemState);

type Table<T> = LocalKey<RefCell<Vec<T>>>;

/// Rows of the property scope, or those whose `field` equals `value`.
fn select<T: Row>(table: &'static Table<T>, filter: Option<(&str, String)>) -> Result<Vec<T>> {
    if in_memory_enabled() {
        return Ok(table.with(|rows| rows.borrow().clone()));
    }
    let mut query = Query::<T>::new().limit(1000);
    if let Some((field, value)) = filter {
        query = query.r#where(eq(field, value));
    }
    // ponytail: one 1000-row page; paginate if a property outgrows it.
    Ok(find::<T, T>(query)?.items)
}

fn upsert<T: Row>(table: &'static Table<T>, row: T) -> Result<()> {
    if in_memory_enabled() {
        table.with(|rows| {
            let mut rows = rows.borrow_mut();
            match rows.iter().position(|existing| existing.id() == row.id()) {
                Some(index) => rows[index] = row,
                None => rows.push(row),
            }
        });
        return Ok(());
    }
    // Gateway `repo_create` upserts on primary key (`id`).
    let _ = repo::create::<T, T, T>(row)?;
    Ok(())
}

fn remove<T: Row>(table: &'static Table<T>, id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        table.with(|rows| rows.borrow_mut().retain(|row| row.id() != id));
        return Ok(());
    }
    repo::delete::<T>(id)?;
    Ok(())
}

// --- Checklists -----------------------------------------------------------------------------

/// The property's lists, in display order.
pub fn list_checklists() -> Result<Vec<Checklist>> {
    let mut rows = select(&TEST_LISTS, None)?;
    adopt_legacy_config(&mut rows)?;
    rows.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(rows)
}

pub fn save_checklist(list: Checklist) -> Result<()> {
    upsert(&TEST_LISTS, list)
}

/// Deletes a list with its items and its display settings.
pub fn delete_checklist(id: Uuid) -> Result<()> {
    for item in all_items_of(id)? {
        remove(&TEST_ITEMS, item.id)?;
    }
    display::delete(id)?;
    remove(&TEST_LISTS, id)
}

/// Creates a list from `template`, placed after the existing ones.
pub fn create_from_template(template: &Template) -> Result<Checklist> {
    let (name_fr, name_en) = template.name();
    let host = template.audience == lists::HOST;
    let list = Checklist {
        id: Uuid::new_v4(),
        name_fr,
        name_en,
        audience: template.audience.to_string(),
        icon: template.icon.to_string(),
        trigger: template.trigger.to_string(),
        placement: lists::BOOKLET.to_string(),
        assignee_name: None,
        assignee_role: None,
        deadline: template.deadline.map(str::to_string),
        notify_assignee: host,
        alert_host: host,
        sort_order: list_checklists()?.len() as i32,
        created_at: time::now()?,
    };
    save_checklist(list.clone())?;
    let groups = template.item_groups();
    let items = template
        .item_labels()
        .into_iter()
        .enumerate()
        .map(|(index, (fr, en, photo))| {
            let labels = Labels::from([("fr".to_string(), fr), ("en".to_string(), en)]);
            let group = match groups.get(index) {
                Some((fr, en)) if !fr.trim().is_empty() => Labels::from([
                    ("fr".to_string(), fr.clone()),
                    ("en".to_string(), en.clone()),
                ]),
                _ => Labels::new(),
            };
            ItemDraft {
                id: None,
                labels,
                group,
                description: Labels::new(),
                photo_required: photo,
            }
        })
        .collect();
    replace_items(list.id, items)?;
    Ok(list)
}

/// The v3 migration names every former list « duringStay »: a host who had picked another
/// moment in the old KV config gets it back once, then the key goes.
fn adopt_legacy_config(rows: &mut [Checklist]) -> Result<()> {
    const LEGACY_KEY: &str = "config";
    let Some(list) = rows.iter_mut().find(|list| list.audience == lists::GUEST) else {
        return Ok(());
    };
    let Some(bytes) = kv::get(LEGACY_KEY)? else {
        return Ok(());
    };
    let show_when = serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|config| config["show_when"].as_str().map(str::to_string))
        .unwrap_or_default();
    let trigger = match show_when.as_str() {
        "always" => lists::BEFORE_ARRIVAL,
        "before_checkout" | "checkout_day" => lists::AT_DEPARTURE,
        _ => lists::DURING_STAY,
    };
    if list.trigger != trigger {
        list.trigger = trigger.to_string();
        save_checklist(list.clone())?;
    }
    kv::delete(LEGACY_KEY)
}

// --- Items ----------------------------------------------------------------------------------

/// Every item of the property, in display order — a step left blank by the host excepted.
///
/// A blank step is stored so that `publishReadiness` can say « Écrivez l'étape. » under it; until
/// written it is no one's step: not ticked, not counted, not in a task.
pub fn list_items() -> Result<Vec<ChecklistItem>> {
    Ok(all_items()?
        .into_iter()
        .filter(|item| !labels::labels_from_item(item).is_empty())
        .collect())
}

/// Every item, blank steps included: what the editor shows and the publish checks read.
pub fn all_items() -> Result<Vec<ChecklistItem>> {
    let mut items = select(&TEST_ITEMS, None)?;
    items.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(items)
}

pub fn items_of(checklist_id: Uuid) -> Result<Vec<ChecklistItem>> {
    Ok(list_items()?
        .into_iter()
        .filter(|item| item.checklist_id == checklist_id)
        .collect())
}

/// [`items_of`], blank steps included.
pub fn all_items_of(checklist_id: Uuid) -> Result<Vec<ChecklistItem>> {
    Ok(all_items()?
        .into_iter()
        .filter(|item| item.checklist_id == checklist_id)
        .collect())
}

/// Replaces the items of a list; an item keeping its id keeps its guest completions.
/// Une étape telle que le formulaire hôte la rend : ce qui identifie la ligne, ses textes par
/// langue, et les deux cases du §2.9.
pub struct ItemDraft {
    pub id: Option<Uuid>,
    pub labels: Labels,
    pub group: Labels,
    pub description: Labels,
    pub photo_required: bool,
}

pub fn replace_items(checklist_id: Uuid, items: Vec<ItemDraft>) -> Result<()> {
    let existing = all_items_of(checklist_id)?;
    let kept: Vec<Uuid> = items.iter().filter_map(|draft| draft.id).collect();
    for row in &existing {
        if !kept.contains(&row.id) {
            remove(&TEST_ITEMS, row.id)?;
        }
    }
    let now = time::now()?;
    for (index, draft) in items.into_iter().enumerate() {
        let previous = draft
            .id
            .and_then(|id| existing.iter().find(|row| row.id == id));
        let (label_fr, label_en) = labels::encode_labels(&draft.labels);
        upsert(
            &TEST_ITEMS,
            ChecklistItem {
                id: previous.map_or_else(Uuid::new_v4, |row| row.id),
                checklist_id,
                label_fr,
                label_en,
                group_i18n: labels::encode_map(&draft.group),
                description_i18n: labels::encode_map(&draft.description),
                photo_required: draft.photo_required,
                sort_order: index as i32,
                created_at: previous.map_or(now, |row| row.created_at),
            },
        )?;
    }
    Ok(())
}

// --- Guest completions ----------------------------------------------------------------------

/// Completions of the property (`None`) or of one stay.
pub fn list_completions(stay_id: Option<Uuid>) -> Result<Vec<ChecklistCompletion>> {
    let rows = select(
        &TEST_COMPLETIONS,
        stay_id.map(|id| ("stay_id", id.to_string())),
    )?;
    Ok(rows
        .into_iter()
        .filter(|row| stay_id.is_none_or(|id| row.stay_id == id))
        .collect())
}

/// Marks an item complete for the stay (idempotent).
pub fn complete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    if !list_items()?.iter().any(|item| item.id == item_id) {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    if list_completions(Some(stay_id))?
        .iter()
        .any(|row| row.item_id == item_id)
    {
        return Ok(());
    }
    upsert(
        &TEST_COMPLETIONS,
        ChecklistCompletion {
            id: Uuid::new_v4(),
            stay_id,
            item_id,
            completed_at: time::now()?,
        },
    )
}

/// Removes a completion for the stay (idempotent).
pub fn uncomplete_item(stay_id: Uuid, item_id: Uuid) -> Result<()> {
    for row in list_completions(Some(stay_id))? {
        if row.item_id == item_id {
            remove(&TEST_COMPLETIONS, row.id)?;
        }
    }
    Ok(())
}

// --- Host task state ------------------------------------------------------------------------

/// Ticked states of the property (`None`) or of one task.
pub fn task_states(task_id: Option<&str>) -> Result<Vec<TaskItemState>> {
    let rows = select(
        &TEST_TASK_STATES,
        task_id.map(|id| ("task_id", id.to_string())),
    )?;
    Ok(rows
        .into_iter()
        .filter(|row| task_id.is_none_or(|id| row.task_id == id))
        .collect())
}

/// Ticks or unticks one item of a task; unticking drops the photo.
pub fn set_task_item(
    task_id: &str,
    item_id: Uuid,
    done: bool,
    photo: Option<String>,
    now: DateTime<Utc>,
) -> Result<()> {
    let existing = task_states(Some(task_id))?
        .into_iter()
        .find(|row| row.item_id == item_id);
    upsert(
        &TEST_TASK_STATES,
        TaskItemState {
            id: existing.as_ref().map_or_else(Uuid::new_v4, |row| row.id),
            task_id: task_id.to_string(),
            item_id,
            done,
            photo: if done {
                photo.or_else(|| existing.and_then(|row| row.photo))
            } else {
                None
            },
            done_at: done.then_some(now),
        },
    )
}

// --- Display settings of a guest list ------------------------------------------------------

/// « Étapes visibles » and « Message final » of a guest list (spec Checklist §2.1).
///
/// In KV, one key per list, not on the `Checklist` entity: a new column would need a
/// `schema_version` the platform does not migrate yet. Pas de TTL — un réglage d'hôte ne périme pas.
pub mod display {
    use portaki_sdk::host::kv;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub const DEFAULT_VISIBLE_LIMIT: u32 = 5;
    pub const MIN_VISIBLE_LIMIT: u32 = 3;
    pub const MAX_VISIBLE_LIMIT: u32 = 10;

    #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Display {
        /// As typed, even out of bounds: the editor shows « Entre 3 et 10. » under it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub visible_limit: Option<u32>,
        /// JSON map of every language ([`crate::labels::encode_map`]); empty = the default thanks.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        pub done_message: String,
    }

    impl Display {
        pub fn visible_limit(&self) -> u32 {
            self.visible_limit.map_or(DEFAULT_VISIBLE_LIMIT, |n| {
                n.clamp(MIN_VISIBLE_LIMIT, MAX_VISIBLE_LIMIT)
            })
        }
    }

    fn key(list_id: Uuid) -> String {
        format!("display:{list_id}")
    }

    pub fn read(list_id: Uuid) -> Display {
        kv::get(&key(list_id))
            .ok()
            .flatten()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn write(list_id: Uuid, display: &Display) -> portaki_sdk::Result<()> {
        let bytes = serde_json::to_vec(display).map_err(|error| {
            portaki_sdk::PortakiError::Storage(format!("display serialize: {error}"))
        })?;
        kv::set(&key(list_id), &bytes, None)
    }

    pub fn delete(list_id: Uuid) -> portaki_sdk::Result<()> {
        kv::delete(&key(list_id))
    }
}
