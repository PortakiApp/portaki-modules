//! TipTap helpers for host `RichTextEditor` / guest `RichText`.

use portaki_sdk::sdui::common::RichTextDoc;
use serde_json::Value;

/// Empty TipTap doc for a blank editor.
pub fn empty_editor_value() -> String {
    r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string()
}

/// Value for [`RichTextEditor`] — TipTap JSON as-is, plain text wrapped.
pub fn editor_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return empty_editor_value();
    }
    if is_tiptap_doc(trimmed) {
        return trimmed.to_string();
    }
    RichTextDoc::new().paragraph(trimmed).to_json_string()
}

/// True when `raw` is a TipTap `doc` JSON document.
pub fn is_tiptap_doc(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };
    value.get("type").and_then(|t| t.as_str()) == Some("doc")
}

/// Plain-text extract (list subtitles / blank checks).
pub fn body_plain_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if !is_tiptap_doc(trimmed) {
        return trimmed.to_string();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return trimmed.to_string();
    };
    let mut out = Vec::new();
    collect_text(&value, &mut out);
    out.join("\n")
}

fn collect_text(node: &Value, out: &mut Vec<String>) {
    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        if !text.is_empty() {
            out.push(text.to_string());
        }
    }
    if let Some(children) = node.get("content").and_then(|c| c.as_array()) {
        for child in children {
            collect_text(child, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_value_wraps_plain_text() {
        let v = editor_value("Hello");
        assert!(is_tiptap_doc(&v));
        assert!(body_plain_text(&v).contains("Hello"));
    }
}
