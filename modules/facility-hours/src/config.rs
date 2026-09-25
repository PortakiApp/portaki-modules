//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use std::collections::BTreeSet;

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(required, label = "config.facilities")]
    pub facilities: Vec<FacilityRow>,
    #[field(label = "host.note.label")]
    pub general_note: I18nText,
}

/// The old KV blob: the facilities as a JSON string, `facilities_json`, before the form slots;
/// each row's `lines` as a list of per-language texts; a row named `name` by an early form.
fn legacy(mut old: Value) -> Value {
    if let Some(object) = old.as_object_mut() {
        map_legacy(object);
    }
    old
}

fn map_legacy(old: &mut Map<String, Value>) {
    let listed = old
        .remove("facilities_json")
        .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
    let held = old
        .get("facilities")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty());
    if let (Some(facilities), false) = (listed, held) {
        old.insert("facilities".into(), facilities);
    }
    for row in old
        .get_mut("facilities")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object_mut)
    {
        if let Some(name) = row.remove("name") {
            row.entry("title").or_insert(name);
        }
        if let Some(Value::Array(lines)) = row.get("lines") {
            let lines = joined_lines(lines);
            row.insert("lines".into(), lines);
        }
    }
}

/// A list of per-language lines as one text per language, one line each — a line missing in a
/// language falls back as it did when shown alone.
fn joined_lines(lines: &[Value]) -> Value {
    let lines: Vec<I18nText> = lines
        .iter()
        .filter_map(|line| serde_json::from_value(line.clone()).ok())
        .collect();
    let languages: BTreeSet<&str> = lines
        .iter()
        .flat_map(|line| {
            ["fr", "en"]
                .into_iter()
                .chain(line.others.keys().map(String::as_str))
        })
        .collect();
    let by_language: Map<String, Value> = languages
        .into_iter()
        .map(|language| {
            let text = lines
                .iter()
                .map(|line| line.get(language).trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            (language.to_string(), Value::String(text))
        })
        .collect();
    Value::Object(by_language)
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_facilities().is_empty() && self.general_note.is_blank()
    }

    /// The named rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_facilities(&self) -> Vec<FacilityRow> {
        self.facilities
            .iter()
            .filter(|f| !f.title.is_blank())
            .cloned()
            .collect()
    }
}

/// A facility, as the form sends it (and its `id`); the platform keeps the other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FacilityRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub title: I18nText,
    /// One line of text per line.
    pub lines: I18nText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<String>,
    pub note: I18nText,
}

impl FacilityRow {
    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.hours.as_deref().is_none_or(|h| h.trim().is_empty())
            && self.lines.is_blank()
            && self.note.is_blank()
    }

    /// The non-blank lines in `locale`.
    pub fn lines(&self, locale: &str) -> Vec<String> {
        self.lines
            .get(locale)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_blank_slots_left_out() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "facilities": [{ "title": "Piscine", "hours": "9 h – 20 h" }, { "title": "", "hours": "" }],
            "general_note": "Horaires indicatifs"
        }))
        .unwrap();
        let facilities = config.parse_facilities();
        assert_eq!(facilities.len(), 1);
        assert_eq!(facilities[0].title.get("en"), "Piscine");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    fn legacy_facilities_json_becomes_facilities() {
        let mapped = legacy(json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00","note":{"fr":"Bonnet"}}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "id": "pool", "title": { "fr": "Piscine", "en": "Pool" },
                                 "hours": "08:00 – 20:00", "note": { "fr": "Bonnet" } }],
                "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("en"), "Pool");
        assert_eq!(config.facilities[0].note.get("en"), "Bonnet");
        assert_eq!(config.general_note.get("en"), "Indicative hours");
        // A list already there wins over the old string; an unreadable string imports nothing.
        let kept = legacy(json!({ "facilities": [{ "title": "Spa" }], "facilities_json": "[]" }));
        assert_eq!(kept, json!({ "facilities": [{ "title": "Spa" }] }));
        assert_eq!(legacy(json!({ "facilities_json": "[oops" })), json!({}));
        // An empty list (the old reader's « nothing saved yet ») still takes the string.
        let empty = legacy(json!({ "facilities": [], "facilities_json": r#"[{"title":"Spa"}]"# }));
        assert_eq!(empty, json!({ "facilities": [{ "title": "Spa" }] }));
    }

    #[test]
    fn legacy_lines_become_one_text_per_language() {
        let mapped = legacy(json!({ "facilities": [{
            "id": "pool",
            "title": { "fr": "Piscine", "en": "Pool", "de": "Schwimmbad" },
            "lines": [
                { "fr": "Tous les jours", "en": "Every day" },
                { "fr": "  " , "en": "" },
                { "fr": "Enfants accompagnés", "de": "Kinder begleitet" }
            ]
        }] }));
        assert_eq!(
            mapped["facilities"][0]["lines"],
            json!({
                "fr": "Tous les jours\nEnfants accompagnés",
                "en": "Every day\nEnfants accompagnés",
                "de": "Tous les jours\nKinder begleitet"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(
            config.facilities[0].lines("en-US"),
            ["Every day", "Enfants accompagnés"]
        );
        assert_eq!(config.facilities[0].title.get("de"), "Schwimmbad");
    }

    #[test]
    fn legacy_name_becomes_title_and_a_plain_note_stays() {
        let mapped = legacy(json!({
            "facilities": [{ "name": "Accueil", "hours": "16:00" }, { "name": "x", "title": { "fr": "Spa" } }],
            "general_note": "Horaires indicatifs"
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "title": "Accueil", "hours": "16:00" }, { "title": { "fr": "Spa" } }],
                "general_note": "Horaires indicatifs"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("fr"), "Accueil");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"lines":[{"fr":"Tous les jours","en":"Every day"}],"hours":"08:00 – 20:00"}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                let facilities = config.parse_facilities();
                assert_eq!(facilities[0].title.get("en"), "Pool");
                assert_eq!(facilities[0].lines("en"), ["Every day"]);
                assert_eq!(config.general_note.get("en"), "Indicative hours");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }
}
