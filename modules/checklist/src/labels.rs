//! N-language checklist labels stored in `label_fr` (JSON map) with `label_en` legacy.

use std::collections::BTreeMap;

use portaki_sdk::contracts::i18n::I18nText;
use serde_json::Value;

use crate::entities::{Checklist, ChecklistItem};

/// Label per language code (`fr`, `en`, …).
pub type Labels = BTreeMap<String, String>;

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

pub fn labels_from_item(item: &ChecklistItem) -> Labels {
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

/// Le nom d'une liste par langue, comme [`labels_from_item`] : `name_fr` porte la carte de
/// toutes les langues, `name_en` n'est plus que le repli des listes nommées avant ce changement.
///
/// Ce nom sert de titre de groupe au voyageur quand le logement a plusieurs listes
/// (`guest/home.rs`) : le borner à deux langues lui donnait un titre français en japonais.
pub fn labels_from_list(list: &Checklist) -> Labels {
    let mut out = decode_map(&list.name_fr);
    if !out.is_empty() {
        if !list.name_en.trim().is_empty() {
            out.entry("en".into())
                .or_insert_with(|| list.name_en.trim().to_string());
        }
        return out;
    }
    if !list.name_en.trim().is_empty() {
        out.insert("en".into(), list.name_en.trim().to_string());
    }
    out
}

/// Le nom d'une liste dans la langue du lecteur, avec les replis de [`pick_label`].
///
/// Le seul chemin vers le nom d'une liste : lire `name_fr` directement rendrait la carte JSON.
pub fn list_name(list: &Checklist, locale: &str, property_locale: &str) -> String {
    pick_label(&labels_from_list(list), locale, property_locale)
}

/// Repose le nom d'une liste dans une langue, en gardant les autres.
pub fn with_list_name(list: &Checklist, lang: &str, name: &str) -> (String, String) {
    let mut labels = labels_from_list(list);
    labels.insert(lang_code(lang), name.trim().to_string());
    encode_labels(&labels)
}

pub fn encode_labels(labels: &Labels) -> (String, String) {
    let cleaned: BTreeMap<String, String> = labels
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| (lang_code(k), v.trim().to_string()))
        .collect();
    let json = serde_json::to_string(&cleaned).unwrap_or_else(|_| "{}".into());
    (json, String::new())
}

/// Encode une carte de langues pour une colonne qui n'a pas d'héritage bilingue à traîner
/// (le groupe, la précision) : une seule colonne, une seule forme.
pub fn encode_map(values: &Labels) -> String {
    let cleaned: BTreeMap<String, String> = values
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| (lang_code(k), v.trim().to_string()))
        .collect();
    if cleaned.is_empty() {
        return String::new();
    }
    serde_json::to_string(&cleaned).unwrap_or_default()
}

/// Relit une telle colonne. Un texte nu est accepté et rangé en français : c'est ce qu'une ligne
/// écrite avant que la colonne existe contiendrait si elle était remplie à la main.
pub fn decode_map(raw: &str) -> Labels {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Labels::new();
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
    Labels::from([("fr".to_string(), trimmed.to_string())])
}

pub fn pick_label(labels: &Labels, guest_locale: &str, property_locale: &str) -> String {
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

pub fn get_label(item: &ChecklistItem, locale: &str) -> String {
    let labels = labels_from_item(item);
    labels.get(&lang_code(locale)).cloned().unwrap_or_default()
}

/// French and English label of an item, each falling back on the other.
pub fn i18n_label(item: &ChecklistItem) -> I18nText {
    let labels = labels_from_item(item);
    I18nText::new(
        pick_label(&labels, "fr", "en"),
        pick_label(&labels, "en", "fr"),
    )
}
