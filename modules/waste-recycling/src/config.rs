//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use std::collections::BTreeSet;

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(required, label = "config.bins")]
    pub bins: Vec<BinRow>,
    #[field(label = "host.schedule.label")]
    pub collection_schedule: I18nText,
}

/// The old KV blob: the bins as a JSON string, `bins_json`, before the form slots; each bin's
/// `items` as a list of per-language texts.
fn legacy(mut old: Value) -> Value {
    if let Some(object) = old.as_object_mut() {
        map_legacy(object);
    }
    old
}

fn map_legacy(old: &mut Map<String, Value>) {
    let listed = old
        .remove("bins_json")
        .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
    let held = old
        .get("bins")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty());
    if let (Some(bins), false) = (listed, held) {
        old.insert("bins".into(), bins);
    }
    for row in old
        .get_mut("bins")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object_mut)
    {
        if let Some(Value::Array(items)) = row.get("items") {
            let items = joined_lines(items);
            row.insert("items".into(), items);
        }
    }
}

/// A list of per-language texts as one text per language, one line each — an item missing in a
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
        self.parse_bins().is_empty() && self.collection_schedule.is_blank()
    }

    /// The named rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_bins(&self) -> Vec<BinRow> {
        self.bins
            .iter()
            .filter(|b| !b.title.is_blank())
            .cloned()
            .collect()
    }
}

/// A bin. The form sends `title`, `items` and `color` (and `id`); the platform keeps the other
/// languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct BinRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub title: I18nText,
    /// One item per line.
    pub items: I18nText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl BinRow {
    /// Nothing the form shows but a color: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank() && self.items.is_blank()
    }

    /// The non-blank items in `locale`.
    pub fn items(&self, locale: &str) -> Vec<String> {
        self.items
            .get(locale)
            .lines()
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(String::from)
            .collect()
    }
}

/// The bin tints the host can pick — each one a [`Swatch`] the booklet resolves in its theme.
///
/// A name, never a hex: the yellow bin is yellow because the municipality says so, but which
/// yellow is the shell's call — its palette, a dark theme, contrast.
const BIN_SWATCHES: [(&str, Swatch); 4] = [
    ("yellow", Swatch::Yellow),
    ("green", Swatch::Green),
    ("brown", Swatch::Brown),
    ("grey", Swatch::Grey),
];

/// The hex values stored before swatches, read back as the tint they stood for.
const LEGACY_HEX: [(&str, &str); 4] = [
    ("#f4c020", "yellow"),
    ("#3a8a4d", "green"),
    ("#8b5a2b", "brown"),
    ("#8b949e", "grey"),
];

/// The canonical tint name for a stored or submitted value — `None` for anything else.
///
/// Accepts the names the host select sends, `gray`, and the hex strings earlier versions
/// stored, so configurations saved before swatches keep their dots.
pub fn bin_color_name(value: Option<&str>) -> Option<&'static str> {
    let value = value?.trim().to_ascii_lowercase();
    let value = if value == "gray" {
        "grey".to_string()
    } else {
        value
    };
    LEGACY_HEX
        .iter()
        .find(|(hex, _)| *hex == value)
        .map(|(_, name)| *name)
        .or_else(|| {
            BIN_SWATCHES
                .iter()
                .find(|(name, _)| *name == value)
                .map(|(name, _)| *name)
        })
}

/// The swatch a bin's dot takes, if its color is one the booklet knows.
pub fn bin_swatch(value: Option<&str>) -> Option<Swatch> {
    let name = bin_color_name(value)?;
    BIN_SWATCHES
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, swatch)| *swatch)
}

#[cfg(test)]
mod bin_color_tests {
    use super::*;

    #[test]
    fn a_host_name_is_its_own_swatch() {
        assert_eq!(bin_color_name(Some("yellow")), Some("yellow"));
        assert_eq!(bin_color_name(Some(" Gray ")), Some("grey"));
        assert_eq!(bin_swatch(Some("brown")), Some(Swatch::Brown));
    }

    /// Configurations saved before swatches stored hex: their bins keep their dots.
    #[test]
    fn a_legacy_hex_reads_as_the_tint_it_stood_for() {
        assert_eq!(bin_color_name(Some("#F4C020")), Some("yellow"));
        assert_eq!(bin_swatch(Some("#3a8a4d")), Some(Swatch::Green));
    }

    /// An arbitrary color is not a tint the booklet knows — no dot rather than a guessed one.
    #[test]
    fn anything_else_has_no_swatch() {
        assert_eq!(bin_swatch(Some("#123456")), None);
        assert_eq!(bin_swatch(Some("")), None);
        assert_eq!(bin_swatch(None), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_plain_strings() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "bins": [
                { "title": "Bac jaune", "items": "Emballages\nCartons", "color": "yellow" },
                { "title": "", "items": "", "color": "" }
            ],
            "collection_schedule": "Mardi matin"
        }))
        .unwrap();
        let bins = config.parse_bins();
        assert_eq!(bins.len(), 1);
        assert_eq!(bins[0].title.get("en"), "Bac jaune");
        assert_eq!(bins[0].items("en"), ["Emballages", "Cartons"]);
        assert_eq!(config.collection_schedule.get("en"), "Mardi matin");
    }

    #[test]
    fn legacy_bins_json_becomes_bins() {
        let mapped = legacy(json!({
            "bins_json": r##"[{"id":"glass","title":{"fr":"Verre","en":"Glass"},"items":[],"color":"#3a8a4d"}]"##,
            "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
        }));
        assert_eq!(
            mapped,
            json!({
                "bins": [{ "id": "glass", "title": { "fr": "Verre", "en": "Glass" }, "items": {},
                           "color": "#3a8a4d" }],
                "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.parse_bins()[0].title.get("en"), "Glass");
        assert_eq!(
            bin_swatch(config.bins[0].color.as_deref()),
            Some(Swatch::Green)
        );
        assert_eq!(config.collection_schedule.get("en"), "Tuesday");
        // A list already there wins over the old string; an unreadable string imports nothing.
        let kept = legacy(json!({ "bins": [{ "title": "Verre" }], "bins_json": "[]" }));
        assert_eq!(kept, json!({ "bins": [{ "title": "Verre" }] }));
        assert_eq!(legacy(json!({ "bins_json": "[oops" })), json!({}));
        // An empty list (the old reader's « nothing saved yet ») still takes the string.
        let empty = legacy(json!({ "bins": [], "bins_json": r#"[{"title":"Verre"}]"# }));
        assert_eq!(empty, json!({ "bins": [{ "title": "Verre" }] }));
    }

    #[test]
    fn legacy_items_become_one_text_per_language() {
        let mapped = legacy(json!({
            "bins": [{
                "id": "yellow",
                "title": { "fr": "Bac jaune", "en": "Yellow bin" },
                "items": [
                    { "fr": "Plastique", "en": "Plastic" },
                    { "fr": "", "en": " " },
                    { "fr": "Carton" }
                ],
                "color": "yellow"
            }],
            "collection_schedule": "Mardi"
        }));
        assert_eq!(
            mapped["bins"][0]["items"],
            json!({ "fr": "Plastique\nCarton", "en": "Plastic\nCarton" })
        );
        assert_eq!(mapped["collection_schedule"], "Mardi");
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.bins[0].items("en-US"), ["Plastic", "Carton"]);
        assert_eq!(config.bins[0].color.as_deref(), Some("yellow"));
        assert_eq!(config.collection_schedule.get("en"), "Mardi");
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "bins_json": r#"[{"id":"glass","title":{"fr":"Verre","en":"Glass"},"items":[{"fr":"Bouteilles","en":"Bottles"}]}]"#,
            "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                let bins = config.parse_bins();
                assert_eq!(bins[0].title.get("en"), "Glass");
                assert_eq!(bins[0].items("en"), ["Bottles"]);
                assert_eq!(config.collection_schedule.get("en"), "Tuesday");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }
}
