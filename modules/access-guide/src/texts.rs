//! Per-language guest/host copy stored in KV (`texts/{lang}`).
//!
//! Structural config stays language-invariant under the `config` key.
//! Language short codes are derived from BCP-47 locales (`fr-FR` → `fr`).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TEXTS_PREFIX: &str = "texts/";

/// Language-specific titles and free-text fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleTexts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building_note: Option<String>,
    #[serde(default)]
    pub parking_info: String,
    #[serde(default)]
    pub global_note: String,
    #[serde(default)]
    pub steps: Vec<StepText>,
}

impl ModuleTexts {
    pub fn is_empty(&self) -> bool {
        opt_empty(&self.method_instructions)
            && opt_empty(&self.building_note)
            && self.parking_info.trim().is_empty()
            && self.global_note.trim().is_empty()
            && self.steps.iter().all(|s| s.is_empty())
    }

    pub fn step_by_id(&self, id: &str) -> Option<&StepText> {
        self.steps.iter().find(|s| s.id == id)
    }
}

/// Title/detail for one arrival step (matched to shared config by `id`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StepText {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl StepText {
    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty() && opt_empty(&self.detail)
    }
}

/// Map `fr-FR` / `en_US` → short code `fr` / `en`.
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

fn texts_key(lang: &str) -> String {
    format!("{TEXTS_PREFIX}{}", lang_code(lang))
}

pub fn load_texts(lang: &str) -> Result<ModuleTexts> {
    let key = texts_key(lang);
    let Some(bytes) = host::kv::get(&key)? else {
        return Ok(ModuleTexts::default());
    };
    serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid texts JSON ({key}): {error}"))
    })
}

pub fn save_texts(lang: &str, texts: &ModuleTexts) -> Result<()> {
    let key = texts_key(lang);
    let bytes = serde_json::to_vec(texts).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("texts serialize ({key}): {error}"))
    })?;
    host::kv::set(&key, &bytes, None)
}

/// Host surfaces: texts for the active request locale only.
pub fn load_texts_for_host(locale: &str) -> Result<ModuleTexts> {
    load_texts(&lang_code(locale))
}

/// Guest surfaces: guest locale → property default → `fr` → first available.
pub fn load_texts_for_guest(guest_locale: &str, property_locale: &str) -> Result<ModuleTexts> {
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
        let texts = load_texts(lang)?;
        if !texts.is_empty() {
            return Ok(texts);
        }
    }

    let keys = host::kv::list(TEXTS_PREFIX).unwrap_or_default();
    let mut langs: Vec<String> = keys
        .into_iter()
        .filter_map(|k| k.strip_prefix(TEXTS_PREFIX).map(str::to_string))
        .filter(|l| !l.is_empty())
        .collect();
    langs.sort();
    for lang in langs {
        if !tried.insert(lang.clone()) {
            continue;
        }
        let texts = load_texts(&lang)?;
        if !texts.is_empty() {
            return Ok(texts);
        }
    }

    Ok(ModuleTexts::default())
}

/// Pull language strings still embedded in a legacy / pre-split `config` JSON document.
///
/// Returns `(fr, en)`. Plain strings land in `fr` only; `{ "fr", "en" }` objects split.
pub(crate) fn extract_embedded_texts(root: &Value) -> (ModuleTexts, ModuleTexts) {
    let mut fr = ModuleTexts::default();
    let mut en = ModuleTexts::default();

    if let Some(instr) = root.pointer("/method/instructions") {
        assign_localized_opt(
            instr,
            &mut fr.method_instructions,
            &mut en.method_instructions,
        );
    }

    if let Some(note) = root.pointer("/building_access/note") {
        assign_localized_opt(note, &mut fr.building_note, &mut en.building_note);
    }

    if let Some(info) = root
        .pointer("/parking/info")
        .or_else(|| root.get("parking_info"))
    {
        assign_localized_string(info, &mut fr.parking_info, &mut en.parking_info);
    }

    if let Some(note) = root
        .pointer("/arrival/global_note")
        .or_else(|| root.get("global_note"))
    {
        assign_localized_string(note, &mut fr.global_note, &mut en.global_note);
    }

    let step_values = step_values_from_root(root);
    for step in step_values {
        let Some(id) = step.get("id").and_then(|v| v.as_str()).map(str::trim) else {
            continue;
        };
        if id.is_empty() {
            continue;
        }
        let (title_fr, title_en) = localized_pair(step.get("title"));
        let (detail_fr_raw, detail_en_raw) = localized_pair(step.get("detail"));
        let detail_fr = nonempty_owned(&detail_fr_raw);
        let detail_en = nonempty_owned(&detail_en_raw);
        if !title_fr.is_empty() || detail_fr.is_some() {
            fr.steps.push(StepText {
                id: id.to_string(),
                title: title_fr,
                detail: detail_fr,
            });
        }
        if !title_en.is_empty() || detail_en.is_some() {
            en.steps.push(StepText {
                id: id.to_string(),
                title: title_en,
                detail: detail_en,
            });
        }
    }

    (fr, en)
}

fn step_values_from_root(root: &Value) -> Vec<Value> {
    if let Some(arr) = root
        .pointer("/arrival/steps")
        .and_then(|v| v.as_array())
        .filter(|a| !a.is_empty())
    {
        return arr.clone();
    }
    if let Some(arr) = root.get("steps").and_then(|v| v.as_array()) {
        if !arr.is_empty() {
            return arr.clone();
        }
    }
    if let Some(raw) = root.get("steps_json").and_then(|v| v.as_str()) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(trimmed) {
                return arr;
            }
        }
    }
    Vec::new()
}

fn assign_localized_opt(value: &Value, fr: &mut Option<String>, en: &mut Option<String>) {
    let (fr_s, en_s) = localized_pair(Some(value));
    if !fr_s.is_empty() {
        *fr = Some(fr_s);
    }
    if !en_s.is_empty() {
        *en = Some(en_s);
    }
}

fn assign_localized_string(value: &Value, fr: &mut String, en: &mut String) {
    let (fr_s, en_s) = localized_pair(Some(value));
    if fr.trim().is_empty() && !fr_s.is_empty() {
        *fr = fr_s;
    }
    if en.trim().is_empty() && !en_s.is_empty() {
        *en = en_s;
    }
}

/// Plain string → `(s, "")`. Object `{fr,en}` → both. Prefer non-empty sides.
fn localized_pair(value: Option<&Value>) -> (String, String) {
    let Some(value) = value else {
        return (String::new(), String::new());
    };
    match value {
        Value::String(s) => (s.trim().to_string(), String::new()),
        Value::Object(map) => {
            let fr = map
                .get("fr")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let en = map
                .get("en")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            (fr, en)
        }
        _ => (String::new(), String::new()),
    }
}

fn opt_empty(value: &Option<String>) -> bool {
    value.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
}

fn nonempty_owned(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Persist extracted texts when the KV slot is still empty (never overwrite).
pub(crate) fn seed_texts_if_absent(lang: &str, texts: &ModuleTexts) -> Result<()> {
    if texts.is_empty() {
        return Ok(());
    }
    let existing = load_texts(lang)?;
    if !existing.is_empty() {
        return Ok(());
    }
    save_texts(lang, texts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lang_code_from_bcp47() {
        assert_eq!(lang_code("fr-FR"), "fr");
        assert_eq!(lang_code("en-US"), "en");
        assert_eq!(lang_code("en_GB"), "en");
        assert_eq!(lang_code("FR"), "fr");
        assert_eq!(lang_code(""), "fr");
    }

    #[test]
    fn extract_splits_bilingual_steps_and_plain_strings() {
        let root = json!({
            "method": { "kind": "keybox", "instructions": "Tourner à droite" },
            "building_access": { "note": { "fr": "Sonnette", "en": "Bell" } },
            "parking": { "info": "Rue A" },
            "arrival": {
                "global_note": { "fr": "Note FR", "en": "Note EN" },
                "steps": [{
                    "id": "1",
                    "kind": "parking",
                    "title": { "fr": "Se garer", "en": "Park" },
                    "detail": { "fr": "Place résident", "en": "Resident spot" }
                }]
            }
        });
        let (fr, en) = extract_embedded_texts(&root);
        assert_eq!(fr.method_instructions.as_deref(), Some("Tourner à droite"));
        assert!(en.method_instructions.is_none());
        assert_eq!(fr.building_note.as_deref(), Some("Sonnette"));
        assert_eq!(en.building_note.as_deref(), Some("Bell"));
        assert_eq!(fr.parking_info, "Rue A");
        assert_eq!(fr.global_note, "Note FR");
        assert_eq!(en.global_note, "Note EN");
        assert_eq!(fr.steps[0].title, "Se garer");
        assert_eq!(en.steps[0].title, "Park");
        assert_eq!(fr.steps[0].detail.as_deref(), Some("Place résident"));
        assert_eq!(en.steps[0].detail.as_deref(), Some("Resident spot"));
    }
}
