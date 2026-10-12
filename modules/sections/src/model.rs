//! Shared view types for sections.

use serde::{Deserialize, Serialize};

/// Guest view of a section, in the reader's language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionView {
    pub id: String,
    pub title: String,
    /// TipTap JSON, or plain text / Markdown from before the rich editor.
    pub body_markdown: String,
    /// Photo d'ouverture du panneau, `None` when the host gave none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
}

impl SectionView {
    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty()
            && crate::content::body_plain_text(&self.body_markdown)
                .trim()
                .is_empty()
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
}
