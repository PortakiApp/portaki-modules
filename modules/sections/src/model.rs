//! Shared view / DTO types for sections.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One locale block for create/update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SectionLocaleInput {
    pub lang: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body_markdown: String,
}

/// Guest/host view of a section for one resolved locale.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionView {
    pub id: Uuid,
    pub sort_order: i32,
    pub title: String,
    pub body_markdown: String,
    pub locales: Vec<SectionLocaleInput>,
}

impl SectionView {
    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty() && self.body_markdown.trim().is_empty()
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

/// Guest/host: prefer request locale, then property default, then `fr`, then first non-empty.
pub fn pick_locale_fields(
    locales: &[SectionLocaleInput],
    guest_locale: &str,
    property_locale: &str,
) -> (String, String) {
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
        if let Some(row) = locales.iter().find(|l| lang_matches(&l.lang, lang)) {
            if !row.title.trim().is_empty() || !row.body_markdown.trim().is_empty() {
                return (row.title.clone(), row.body_markdown.clone());
            }
        }
    }
    for row in locales {
        if !row.title.trim().is_empty() || !row.body_markdown.trim().is_empty() {
            return (row.title.clone(), row.body_markdown.clone());
        }
    }
    if let Some(any) = locales.first() {
        return (any.title.clone(), any.body_markdown.clone());
    }
    (String::new(), String::new())
}

fn lang_matches(lang: &str, want: &str) -> bool {
    lang_code(lang) == lang_code(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_code_from_bcp47() {
        assert_eq!(lang_code("fr-FR"), "fr");
        assert_eq!(lang_code("en-US"), "en");
        assert_eq!(lang_code("de_DE"), "de");
        assert_eq!(lang_code(""), "fr");
    }

    #[test]
    fn pick_falls_back_through_candidates() {
        let locales = vec![
            SectionLocaleInput {
                lang: "en".into(),
                title: "Hello".into(),
                body_markdown: "Body EN".into(),
            },
            SectionLocaleInput {
                lang: "de".into(),
                title: "Hallo".into(),
                body_markdown: "Body DE".into(),
            },
        ];
        let (title, body) = pick_locale_fields(&locales, "it-IT", "en-US");
        assert_eq!(title, "Hello");
        assert_eq!(body, "Body EN");
    }
}
