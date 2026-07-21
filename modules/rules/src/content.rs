//! Structured house-rules payload (not TipTap).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One rule row shown in the guest booklet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleItem {
    /// Lucide / design icon name (`clock-circle`, `x`, …).
    #[serde(default)]
    pub icon: String,
    /// Short rule title.
    #[serde(default)]
    pub title: String,
    /// Optional supporting line.
    #[serde(default)]
    pub subtitle: String,
}

/// Locale payload for one language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulesPayload {
    #[serde(default)]
    pub items: Vec<RuleItem>,
}

impl RulesPayload {
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        serde_json::from_str(trimmed).unwrap_or_default()
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|item| item.title.trim().is_empty())
    }
}

/// N-language storage written into `content_fr` (`content_en` cleared after migrate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulesBundle {
    #[serde(default)]
    pub by_lang: BTreeMap<String, RulesPayload>,
}

impl RulesBundle {
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

    pub fn from_row(content_fr: &str, content_en: &str) -> Self {
        if let Ok(value) = serde_json::from_str::<Value>(content_fr.trim()) {
            if value.get("by_lang").is_some() {
                if let Ok(bundle) = serde_json::from_value::<RulesBundle>(value.clone()) {
                    return bundle;
                }
            }
            // Legacy single-locale payload in content_fr.
            if value.get("items").is_some() {
                let mut bundle = RulesBundle::default();
                let fr = RulesPayload::parse(content_fr);
                if !fr.is_empty() {
                    bundle.by_lang.insert("fr".into(), fr);
                }
                let en = RulesPayload::parse(content_en);
                if !en.is_empty() {
                    bundle.by_lang.insert("en".into(), en);
                }
                return bundle;
            }
        }
        let mut bundle = RulesBundle::default();
        let fr = RulesPayload::parse(content_fr);
        if !fr.is_empty() {
            bundle.by_lang.insert("fr".into(), fr);
        }
        let en = RulesPayload::parse(content_en);
        if !en.is_empty() {
            bundle.by_lang.insert("en".into(), en);
        }
        bundle
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get(&self, lang: &str) -> RulesPayload {
        self.by_lang
            .get(&Self::lang_code(lang))
            .cloned()
            .unwrap_or_default()
    }

    pub fn set(&mut self, lang: &str, payload: RulesPayload) {
        let code = Self::lang_code(lang);
        if payload.is_empty() {
            self.by_lang.remove(&code);
        } else {
            self.by_lang.insert(code, payload);
        }
    }

    /// Sync shared icons from `source` into every language payload (by index).
    pub fn sync_icons_from(&mut self, source: &RulesPayload) {
        for payload in self.by_lang.values_mut() {
            for (index, item) in payload.items.iter_mut().enumerate() {
                if let Some(src) = source.items.get(index) {
                    if !src.icon.trim().is_empty() {
                        item.icon = src.icon.clone();
                    }
                }
            }
        }
    }

    pub fn pick(&self, guest_locale: &str, property_locale: &str) -> RulesPayload {
        let candidates = [
            Self::lang_code(guest_locale),
            Self::lang_code(property_locale),
            "fr".to_string(),
        ];
        let mut tried = std::collections::BTreeSet::new();
        for lang in &candidates {
            if !tried.insert(lang.clone()) {
                continue;
            }
            let payload = self.get(lang);
            if !payload.is_empty() {
                return payload;
            }
        }
        for payload in self.by_lang.values() {
            if !payload.is_empty() {
                return payload.clone();
            }
        }
        RulesPayload::default()
    }
}

/// Legacy helper — prefer [`RulesBundle::pick`].
pub fn pick_locale(content_fr: &str, content_en: &str, locale: &str) -> RulesPayload {
    RulesBundle::from_row(content_fr, content_en).pick(locale, "fr")
}
