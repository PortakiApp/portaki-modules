//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::localized::deserialize_localized_field;

pub use crate::localized::Localized;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(required, label = "config.bins")]
    pub bins: Vec<BinRow>,
    /// A plain string from the form; a text per language in the old KV.
    #[field(kind = "textarea", label = "host.schedule.label")]
    #[serde(deserialize_with = "deserialize_localized_field")]
    pub collection_schedule: Localized,
}

impl ModuleConfig {
    /// The config of this install. The import skips what the old KV kept in another shape: the
    /// schedule as a text per language, and the `bins_json` string the form slots replaced.
    /// While the host has not saved that key, it is still read from the KV.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let Some(held) = ctx.module_config.as_ref().and_then(|c| c.as_object()) else {
            return Ok(config);
        };
        let (schedule_held, bins_held) = (
            held.contains_key("collection_schedule"),
            held.contains_key("bins"),
        );
        if schedule_held && bins_held {
            return Ok(config);
        }
        let legacy = portaki_sdk::config::legacy_config()?;
        if !schedule_held {
            if let Some(schedule) = legacy.get("collection_schedule") {
                config.collection_schedule = Localized::from_value(schedule);
            }
        }
        if !bins_held {
            if let Some(bins) = legacy
                .get("bins_json")
                .and_then(Value::as_str)
                .and_then(|raw| serde_json::from_str(raw).ok())
            {
                config.bins = bins;
            }
        }
        Ok(config)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_bins().is_empty() && self.collection_schedule.is_empty()
    }

    /// The named rows: the form sends its six slots, blank ones included.
    pub fn parse_bins(&self) -> Vec<BinRow> {
        self.bins
            .iter()
            .filter(|b| !b.title.is_empty())
            .cloned()
            .collect()
    }
}

/// A bin as the host form sends it (`title`, `items` and `color` as strings), or as the KV kept
/// it (`id`, a title per language, a list of items per language).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinRow {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub title: Localized,
    #[serde(default, deserialize_with = "deserialize_items")]
    pub items: Vec<Localized>,
    #[serde(default)]
    pub color: Option<String>,
}

/// One line of text (the form), or a list per language (the KV).
fn deserialize_items<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Vec<Localized>, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items.iter().map(Localized::from_value).collect(),
        line => Some(Localized::from_value(&line))
            .filter(|line| !line.is_empty())
            .into_iter()
            .collect(),
    })
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
                { "title": "Bac jaune", "items": "Emballages", "color": "yellow" },
                { "title": "", "items": "", "color": "" }
            ],
            "collection_schedule": "Mardi matin"
        }))
        .unwrap();
        let bins = config.parse_bins();
        assert_eq!(bins.len(), 1);
        assert_eq!(bins[0].title.pick("en"), "Bac jaune");
        assert_eq!(bins[0].items.len(), 1);
        assert_eq!(bins[0].items[0].pick("en"), "Emballages");
        assert_eq!(config.collection_schedule.pick("en"), "Mardi matin");
    }

    #[test]
    #[serial_test::serial]
    fn what_the_import_skips_survives() {
        let legacy = json!({
            "bins_json": r#"[{"id":"glass","title":{"fr":"Verre","en":"Glass"},"items":[]}]"#,
            "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.parse_bins()[0].title.pick("en"), "Glass");
                assert_eq!(config.collection_schedule.pick("en"), "Tuesday");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "bins": [], "collection_schedule": "" }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert!(config.bins.is_empty());
                assert!(config.collection_schedule.is_empty());
            });
    }
}
