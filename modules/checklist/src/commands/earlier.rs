//! La reprise des checklists d'avant la collection : les lignes `checklist` et `checklist_item`,
//! et les réglages d'affichage rangés en KV, deviennent la collection `lists`, une seule fois —
//! `#[collection(legacy = …)]`.
//!
//! Les identifiants restent ceux des anciennes lignes : les coches du voyageur
//! (`ChecklistCompletion.item_id`) et l'état des tâches (`TaskItemState`, `task_id` =
//! `<listId>:<stayId>`) les désignent.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::kv;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::resource::{Choice, NoMeta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Audience, Deadline, GuestTrigger, HostTrigger, List, Step};

/// L'ancienne liste, gardée déclarée pour être lue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct Checklist {
    pub id: Uuid,
    pub name_fr: String,
    pub name_en: String,
    pub audience: String,
    pub icon: String,
    pub trigger: String,
    pub placement: String,
    pub assignee_name: Option<String>,
    pub assignee_role: Option<String>,
    pub deadline: Option<String>,
    pub notify_assignee: bool,
    pub alert_host: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(Checklist)]
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const CHECKLIST_INDEXES: &[&str] = &["sort_order"];

/// L'ancienne étape : `label_fr` porte la carte JSON de toutes les langues, `label_en` l'anglais
/// d'avant cette carte.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    #[serde(default)]
    pub group_i18n: String,
    #[serde(default)]
    pub description_i18n: String,
    #[serde(default)]
    pub photo_required: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ChecklistItem)]
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const CHECKLIST_ITEM_INDEXES: &[&str] = &["checklist_id", "sort_order"];

/// « Étapes visibles », « Message final », « Rappel » : la clé KV `display:{listId}`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Display {
    pub visible_limit: Option<u32>,
    pub done_message: String,
    pub remind: Option<bool>,
}

/// Les checklists d'avant, s'il y en a.
pub fn earlier_lists() -> portaki_sdk::Result<Option<(Vec<List>, NoMeta)>> {
    let lists = find::<Checklist, Checklist>(Query::new().limit(1000))?.items;
    if lists.is_empty() {
        return Ok(None);
    }
    let items = find::<ChecklistItem, ChecklistItem>(Query::new().limit(1000))?.items;
    let mut displays = BTreeMap::new();
    for list in &lists {
        let display = kv::get(&format!("display:{}", list.id))?
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        displays.insert(list.id, display);
    }
    let show_when = kv::get("config")?.map(|bytes| {
        serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|config| config["show_when"].as_str().map(str::to_string))
            .unwrap_or_default()
    });
    Ok(Some((
        convert(lists, items, &displays, show_when.as_deref()),
        NoMeta {},
    )))
}

/// Les listes de la collection : l'ordre de l'hôte, les ids gardés, une étape sans texte omise.
///
/// `show_when` : l'ancienne clé KV `config`, qu'un hôte d'avant les listes multiples avait pu
/// régler ; l'ancien module la reportait sur la première liste du voyageur.
pub fn convert(
    mut lists: Vec<Checklist>,
    mut items: Vec<ChecklistItem>,
    displays: &BTreeMap<Uuid, Display>,
    show_when: Option<&str>,
) -> Vec<List> {
    lists.sort_by_key(|a| (a.sort_order, a.created_at));
    items.sort_by_key(|a| (a.sort_order, a.created_at));
    let mut out: Vec<List> = lists
        .iter()
        .map(|row| {
            let steps = items
                .iter()
                .filter(|item| item.checklist_id == row.id)
                .filter_map(step)
                .collect();
            list(row, steps, displays.get(&row.id))
        })
        .collect();
    if let Some(show_when) = show_when {
        if let Some(first) = out.iter_mut().find(|list| list.is_guest()) {
            first.guest_trigger = match show_when {
                "always" => GuestTrigger::BeforeArrival,
                "before_checkout" => GuestTrigger::AtDeparture,
                "checkout_day" => GuestTrigger::DepartureDay,
                _ => GuestTrigger::DuringStay,
            };
        }
    }
    out
}

fn list(row: &Checklist, steps: Vec<Step>, display: Option<&Display>) -> List {
    let guest = row.audience != "host";
    let display = display.cloned().unwrap_or_default();
    let mut name = I18nText::default();
    for (lang, text) in [("fr", &row.name_fr), ("en", &row.name_en)] {
        if !text.trim().is_empty() {
            name = name.with(lang.to_string(), text.trim().to_string());
        }
    }
    let clean = |s: &Option<String>| s.as_deref().map(str::trim).unwrap_or_default().to_string();
    let assignee = match (clean(&row.assignee_name), clean(&row.assignee_role)) {
        (name, role) if role.is_empty() => name,
        (name, role) => format!("{name} · {role}").trim().to_string(),
    };
    let mut list = List {
        id: row.id.to_string(),
        name,
        audience: if guest {
            Audience::Guest
        } else {
            Audience::Host
        },
        icon: row.icon.clone(),
        notify_assignee: row.notify_assignee,
        alert_host: row.alert_host,
        steps,
        ..List::default()
    };
    if guest {
        list.guest_trigger = GuestTrigger::from_wire(&row.trigger).unwrap_or_default();
        // Tel que saisi, hors bornes compris : la publication dit « Entre 3 et 10. » comme avant.
        list.visible_limit = display.visible_limit;
        list.done_message = text_of(decode_map(&display.done_message));
        list.remind = display.remind;
    } else {
        list.host_trigger = HostTrigger::from_wire(&row.trigger).unwrap_or_default();
        list.deadline =
            Deadline::from_wire(row.deadline.as_deref().unwrap_or_default()).unwrap_or_default();
        list.assignee = assignee;
    }
    list
}

/// L'étape `item`, sauf si elle n'a de texte dans aucune langue.
fn step(item: &ChecklistItem) -> Option<Step> {
    let label = labels_of(item);
    if label.is_empty() {
        return None;
    }
    Some(Step {
        id: item.id.to_string(),
        label: text_of(label),
        detail: text_of(decode_map(&item.description_i18n)),
        zone: text_of(decode_map(&item.group_i18n)),
        photo: item.photo_required,
    })
}

fn text_of(map: BTreeMap<String, String>) -> I18nText {
    map.into_iter()
        .fold(I18nText::default(), |text, (lang, value)| {
            text.with(lang, value)
        })
}

fn lang_code(locale: &str) -> String {
    let lower = locale.trim().to_ascii_lowercase();
    let base = lower.split(['-', '_']).next().unwrap_or_default().trim();
    if base.is_empty() {
        "fr".to_string()
    } else {
        base.to_string()
    }
}

/// Une carte de langues JSON ; un texte nu est du français.
fn decode_map(raw: &str) -> BTreeMap<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return BTreeMap::new();
    }
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
        return map
            .into_iter()
            .filter_map(|(key, value)| {
                let text = value.as_str()?.trim();
                (!text.is_empty()).then(|| (lang_code(&key), text.to_string()))
            })
            .collect();
    }
    BTreeMap::from([("fr".to_string(), trimmed.to_string())])
}

/// Le libellé par langue : la carte de `label_fr`, ou le texte nu en français et `label_en`.
fn labels_of(item: &ChecklistItem) -> BTreeMap<String, String> {
    let mut out = decode_map(&item.label_fr);
    let english = item.label_en.trim();
    if !english.is_empty() {
        out.entry("en".to_string())
            .or_insert_with(|| english.to_string());
    }
    out
}
