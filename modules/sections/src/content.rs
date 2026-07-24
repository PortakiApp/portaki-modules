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

/// Count top-level TipTap blocks (paragraph / heading / blockquote / list).
pub fn body_block_count(raw: &str) -> usize {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return 0;
    }
    if !is_tiptap_doc(trimmed) {
        return trimmed
            .split("\n\n")
            .filter(|part| !part.trim().is_empty())
            .count()
            .max(1);
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return 0;
    };
    value
        .get("content")
        .and_then(|c| c.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .filter(|block| {
                    let t = block.get("type").and_then(|x| x.as_str()).unwrap_or("");
                    matches!(
                        t,
                        "paragraph" | "heading" | "blockquote" | "bulletList" | "orderedList"
                    ) && block_has_text(block)
                })
                .count()
        })
        .unwrap_or(0)
}

fn block_has_text(node: &Value) -> bool {
    let mut out = Vec::new();
    collect_text(node, &mut out);
    out.iter().any(|s| !s.trim().is_empty())
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

    #[test]
    fn block_count_from_plain_paragraphs() {
        assert_eq!(body_block_count("a\n\nb\n\nc"), 3);
    }
}
