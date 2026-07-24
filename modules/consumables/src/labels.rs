//! N-language consumable labels stored in `label_fr` (JSON map) with `label_en` legacy.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::entities::ConsumableItem;

pub fn lang_code(locale: &str) -> String {
    let trimmed = locale.trim();
    if trimmed.is_empty() {
        return "fr".to_string();
    }
    let lower = trimmed.to_ascii_lowercase();
    let base = lower.split(['-', '_']).next().unwrap_or("fr").trim();
    if base.is_empty() {
        "fr".to_string()
    } else {
        base.to_string()
    }
}

pub fn labels_from_item(item: &ConsumableItem) -> BTreeMap<String, String> {
    let trimmed = item.label_fr.trim();
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
        let mut out = BTreeMap::new();
        for (key, val) in map {
            if let Some(s) = val.as_str() {
                if !s.trim().is_empty() {
                    out.insert(lang_code(&key), s.trim().to_string());
                }
            }
        }
        if !out.is_empty() {
            if !item.label_en.trim().is_empty() {
                out.entry("en".into())
                    .or_insert_with(|| item.label_en.trim().to_string());
            }
            return out;
        }
    }
    let mut out = BTreeMap::new();
    if !trimmed.is_empty() {
        out.insert("fr".into(), trimmed.to_string());
    }
    if !item.label_en.trim().is_empty() {
        out.insert("en".into(), item.label_en.trim().to_string());
    }
    out
}

pub fn encode_labels(labels: &BTreeMap<String, String>) -> (String, String) {
    let cleaned: BTreeMap<String, String> = labels
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| (lang_code(k), v.trim().to_string()))
        .collect();
    let json = serde_json::to_string(&cleaned).unwrap_or_else(|_| "{}".into());
    (json, String::new())
}

pub fn pick_label(
    labels: &BTreeMap<String, String>,
    guest_locale: &str,
    property_locale: &str,
) -> String {
    let candidates = [
        lang_code(guest_locale),
        lang_code(property_locale),
        "fr".to_string(),
    ];
    let mut tried = std::collections::BTreeSet::new();
    for lang in &candidates {
        if !tried.insert(lang.clone()) {
            continue;
        }
        if let Some(value) = labels.get(lang) {
            if !value.trim().is_empty() {
                return value.clone();
            }
        }
    }
    labels
        .values()
        .find(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

pub fn get_label(item: &ConsumableItem, locale: &str) -> String {
    let labels = labels_from_item(item);
    labels.get(&lang_code(locale)).cloned().unwrap_or_default()
}
