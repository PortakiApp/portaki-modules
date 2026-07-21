//! Appliances payload — single-language device list (TipTap description).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MAX_APPLIANCES: usize = 10;
pub const MAX_FEATURED: usize = 5;

/// Guest-visible vs host-only hidden.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApplianceStatus {
    #[default]
    Active,
    Hidden,
}

/// One appliance / device guide entry (v2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Appliance {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub emoji: String,
    /// TipTap JSON document string.
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub location: String,
    #[serde(default, rename = "manualUrl")]
    pub manual_url: String,
    #[serde(default, rename = "safetyNote")]
    pub safety_note: String,
    #[serde(default)]
    pub status: ApplianceStatus,
}

/// Root payload stored as JSON in `content_fr` (canonical; single language).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppliancesPayload {
    #[serde(default)]
    pub devices: Vec<Appliance>,
    /// Global TipTap JSON safety notice (shown above the guest device list).
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: String,
}

impl AppliancesPayload {
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            return Self::default();
        };
        if looks_legacy(&value) {
            return migrate_legacy(&value);
        }
        serde_json::from_value(value).unwrap_or_default()
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn is_empty(&self) -> bool {
        !self.devices.iter().any(|d| !d.name.trim().is_empty())
    }

    pub fn is_empty_for_guest(&self) -> bool {
        self.guest_devices().is_empty()
    }

    pub fn find_device(&self, device_id: &str) -> Option<&Appliance> {
        self.devices.iter().find(|d| d.id == device_id)
    }

    /// Active, named devices for guest surfaces (hidden excluded). Sorted by `order`.
    pub fn guest_devices(&self) -> Vec<&Appliance> {
        let mut devices: Vec<&Appliance> = self
            .devices
            .iter()
            .filter(|d| d.status == ApplianceStatus::Active && !d.name.trim().is_empty())
            .collect();
        devices.sort_by_key(|d| d.order);
        devices
    }

    /// Featured + active for home card (max [`MAX_FEATURED`]).
    pub fn featured_guest_devices(&self) -> Vec<&Appliance> {
        self.guest_devices()
            .into_iter()
            .filter(|d| d.featured)
            .take(MAX_FEATURED)
            .collect()
    }

    pub fn featured_count(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| d.featured && d.status != ApplianceStatus::Hidden)
            .count()
    }

    pub fn sort_by_order(&mut self) {
        self.devices.sort_by_key(|d| d.order);
    }
}

/// Prefer canonical `content_fr`; fall back to `content_en` for legacy EN-only rows.
pub fn load_from_locale_slots(content_fr: &str, content_en: &str) -> AppliancesPayload {
    AppliancesBundle::from_row(content_fr, content_en).pick("fr", "fr")
}

/// N-language storage written into `content_fr` (`content_en` cleared after migrate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppliancesBundle {
    #[serde(default)]
    pub by_lang: std::collections::BTreeMap<String, AppliancesPayload>,
}

impl AppliancesBundle {
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
                if let Ok(bundle) = serde_json::from_value::<AppliancesBundle>(value.clone()) {
                    return bundle;
                }
            }
            if value.get("devices").is_some() || value.get("safetyNotice").is_some() {
                let mut bundle = Self::default();
                let fr = AppliancesPayload::parse(content_fr);
                if !fr.is_empty() || !fr.safety_notice.trim().is_empty() {
                    bundle.by_lang.insert("fr".into(), fr);
                }
                let en = AppliancesPayload::parse(content_en);
                if !en.is_empty() || !en.safety_notice.trim().is_empty() {
                    bundle.by_lang.insert("en".into(), en);
                }
                return bundle;
            }
        }
        let mut bundle = Self::default();
        let fr = AppliancesPayload::parse(content_fr);
        if !fr.is_empty() || !fr.safety_notice.trim().is_empty() {
            bundle.by_lang.insert("fr".into(), fr);
        }
        let en = AppliancesPayload::parse(content_en);
        if !en.is_empty() || !en.safety_notice.trim().is_empty() {
            bundle.by_lang.insert("en".into(), en);
        }
        bundle
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get(&self, lang: &str) -> AppliancesPayload {
        self.by_lang
            .get(&Self::lang_code(lang))
            .cloned()
            .unwrap_or_default()
    }

    pub fn set(&mut self, lang: &str, payload: AppliancesPayload) {
        let code = Self::lang_code(lang);
        self.by_lang.insert(code, payload);
    }

    /// Copy shared device fields from `source` into other language payloads (by device id).
    pub fn sync_shared_from(&mut self, source: &AppliancesPayload) {
        for payload in self.by_lang.values_mut() {
            for device in &mut payload.devices {
                if let Some(src) = source.find_device(&device.id) {
                    device.emoji = src.emoji.clone();
                    device.featured = src.featured;
                    device.order = src.order;
                    device.manual_url = src.manual_url.clone();
                    device.status = src.status;
                }
            }
            // Align device list order/ids with source when missing.
            for src in &source.devices {
                if !payload.devices.iter().any(|d| d.id == src.id) {
                    let mut clone = src.clone();
                    clone.name.clear();
                    clone.description.clear();
                    clone.location.clear();
                    clone.safety_note.clear();
                    payload.devices.push(clone);
                }
            }
            payload
                .devices
                .retain(|d| source.find_device(&d.id).is_some());
            payload.sort_by_order();
        }
    }

    pub fn pick(&self, guest_locale: &str, property_locale: &str) -> AppliancesPayload {
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
            if !payload.is_empty() || !payload.safety_notice.trim().is_empty() {
                return payload;
            }
        }
        for payload in self.by_lang.values() {
            if !payload.is_empty() || !payload.safety_notice.trim().is_empty() {
                return payload.clone();
            }
        }
        AppliancesPayload::default()
    }
}

fn looks_legacy(value: &Value) -> bool {
    let Some(devices) = value.get("devices").and_then(|d| d.as_array()) else {
        // Legacy empty payload that only carried a snake_case safety_notice.
        return value.get("safety_notice").is_some() && value.get("safetyNotice").is_none();
    };
    if devices.is_empty() {
        return value.get("safety_notice").is_some() && value.get("safetyNotice").is_none();
    }
    devices.iter().any(|device| {
        let has_title = device.get("title").is_some();
        let has_name = device.get("name").is_some();
        let has_steps = device.get("steps").is_some();
        (has_title && !has_name) || has_steps
    })
}

fn migrate_legacy(value: &Value) -> AppliancesPayload {
    let empty = Vec::new();
    let devices = value
        .get("devices")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty);

    let mut migrated = Vec::new();
    for (index, device) in devices.iter().enumerate() {
        let title = string_field(device, "title");
        let id = {
            let raw = string_field(device, "id");
            if raw.is_empty() {
                format!("device-{}", index + 1)
            } else {
                raw
            }
        };
        if title.trim().is_empty() && id.trim().is_empty() {
            continue;
        }
        let steps = device
            .get("steps")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let tip = string_field(device, "tip");
        let icon = string_field(device, "icon");

        migrated.push(Appliance {
            id,
            name: title,
            emoji: emoji_from_legacy_icon(&icon),
            description: steps_and_tip_to_tiptap(&steps, &tip),
            featured: false,
            order: index as i32,
            location: string_field(device, "subtitle"),
            manual_url: string_field(device, "manualUrl"),
            safety_note: String::new(),
            status: ApplianceStatus::Active,
        });
    }

    let legacy_safety = string_field(value, "safety_notice");
    AppliancesPayload {
        devices: migrated,
        safety_notice: plain_text_to_tiptap(&legacy_safety),
    }
}

fn plain_text_to_tiptap(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [{ "type": "text", "text": trimmed }]
        }]
    })
    .to_string()
}

/// Lucide-style names stay empty; emoji / other glyphs are kept.
fn emoji_from_legacy_icon(icon: &str) -> String {
    let trimmed = icon.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return String::new();
    }
    trimmed.to_string()
}

fn steps_and_tip_to_tiptap(steps: &[String], tip: &str) -> String {
    let mut content: Vec<Value> = Vec::new();
    let items: Vec<Value> = steps
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            json!({
                "type": "listItem",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": s }]
                }]
            })
        })
        .collect();
    if !items.is_empty() {
        content.push(json!({
            "type": "bulletList",
            "content": items
        }));
    }
    let tip = tip.trim();
    if !tip.is_empty() {
        content.push(json!({
            "type": "paragraph",
            "content": [{ "type": "text", "text": tip }]
        }));
    }
    if content.is_empty() {
        content.push(json!({ "type": "paragraph" }));
    }
    json!({ "type": "doc", "content": content }).to_string()
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Rough plain-text extract from TipTap JSON (host preview / fallbacks).
pub fn description_plain_text(description: &str) -> String {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return trimmed.to_string();
    };
    if value.get("type").and_then(|t| t.as_str()) != Some("doc") {
        return trimmed.to_string();
    }
    let mut out = Vec::new();
    collect_text(&value, &mut out);
    out.join("\n")
}

/// TipTap JSON → HTML for guest [`RichText`] (bullet lists → numbered steps).
pub fn description_to_html(description: &str) -> String {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return wrap_paragraph(trimmed);
    };
    if value.get("type").and_then(|t| t.as_str()) != Some("doc") {
        return wrap_paragraph(trimmed);
    }
    let mut out = String::new();
    if let Some(children) = value.get("content").and_then(|c| c.as_array()) {
        for child in children {
            render_block(child, &mut out);
        }
    }
    out
}

/// Extract ordered how-to steps from TipTap bullet/ordered lists (guest detail SDUI).
pub fn extract_howto_steps(description: &str) -> Vec<String> {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };
    if value.get("type").and_then(|t| t.as_str()) != Some("doc") {
        return Vec::new();
    }
    let mut steps = Vec::new();
    let Some(children) = value.get("content").and_then(|c| c.as_array()) else {
        return steps;
    };
    for child in children {
        let node_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if node_type != "bulletList" && node_type != "orderedList" {
            continue;
        }
        let Some(items) = child.get("content").and_then(|c| c.as_array()) else {
            continue;
        };
        for item in items {
            let mut parts = Vec::new();
            collect_text(item, &mut parts);
            let text = parts.join(" ").trim().to_string();
            if !text.is_empty() {
                steps.push(text);
            }
        }
    }
    steps
}

fn wrap_paragraph(text: &str) -> String {
    format!("<p>{}</p>", escape_html(text))
}

fn render_block(node: &Value, out: &mut String) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "paragraph" => {
            out.push_str("<p>");
            render_inline_children(node, out);
            out.push_str("</p>");
        }
        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(2)
                .clamp(1, 3);
            out.push_str(&format!("<h{level}>"));
            render_inline_children(node, out);
            out.push_str(&format!("</h{level}>"));
        }
        "bulletList" | "orderedList" => {
            // Guest styles `.appliance-steps` as numbered how-to steps.
            out.push_str("<ol class=\"appliance-steps\">");
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for item in items {
                    out.push_str("<li>");
                    render_list_item_body(item, out);
                    out.push_str("</li>");
                }
            }
            out.push_str("</ol>");
        }
        "blockquote" => {
            out.push_str("<blockquote>");
            if let Some(children) = node.get("content").and_then(|c| c.as_array()) {
                for child in children {
                    render_block(child, out);
                }
            }
            out.push_str("</blockquote>");
        }
        "codeBlock" => {
            out.push_str("<pre><code>");
            render_inline_children(node, out);
            out.push_str("</code></pre>");
        }
        "hardBreak" => out.push_str("<br/>"),
        "horizontalRule" => out.push_str("<hr/>"),
        _ => {
            if let Some(children) = node.get("content").and_then(|c| c.as_array()) {
                for child in children {
                    render_block(child, out);
                }
            } else if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
                out.push_str(&escape_html(text));
            }
        }
    }
}

fn render_list_item_body(item: &Value, out: &mut String) {
    let Some(children) = item.get("content").and_then(|c| c.as_array()) else {
        return;
    };
    for (index, child) in children.iter().enumerate() {
        let child_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if child_type == "paragraph" {
            if index > 0 {
                out.push_str("<br/>");
            }
            render_inline_children(child, out);
        } else {
            render_block(child, out);
        }
    }
}

fn render_inline_children(node: &Value, out: &mut String) {
    let Some(children) = node.get("content").and_then(|c| c.as_array()) else {
        return;
    };
    for child in children {
        render_inline(child, out);
    }
}

fn render_inline(node: &Value, out: &mut String) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "text" => {
            let text = node.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let mut html = escape_html(text);
            if let Some(marks) = node.get("marks").and_then(|m| m.as_array()) {
                for mark in marks {
                    let mark_type = mark.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    html = match mark_type {
                        "bold" | "strong" => format!("<strong>{html}</strong>"),
                        "italic" | "em" => format!("<em>{html}</em>"),
                        "underline" => format!("<u>{html}</u>"),
                        "code" => format!("<code>{html}</code>"),
                        "link" => {
                            let href = mark
                                .get("attrs")
                                .and_then(|a| a.get("href"))
                                .and_then(|h| h.as_str())
                                .unwrap_or("#");
                            format!(
                                "<a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\">{html}</a>",
                                escape_attr(href)
                            )
                        }
                        _ => html,
                    };
                }
            }
            out.push_str(&html);
        }
        "hardBreak" => out.push_str("<br/>"),
        _ => render_inline_children(node, out),
    }
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

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(input: &str) -> String {
    escape_html(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_fr_slots() {
        let raw = r#"{
            "safety_notice": "Coupez l'eau.",
            "devices": [
                {
                    "id": "tv",
                    "icon": "📺",
                    "title": "Télévision",
                    "subtitle": "Salon",
                    "steps": ["Allumez", "HDMI 1"],
                    "tip": "Remote on stand"
                },
                {
                    "id": "washer",
                    "icon": "washing-machine",
                    "title": "Lave-linge",
                    "steps": ["ECO 30"]
                }
            ]
        }"#;
        let payload = AppliancesPayload::parse(raw);
        assert_eq!(payload.devices.len(), 2);
        assert_eq!(payload.devices[0].name, "Télévision");
        assert_eq!(payload.devices[0].emoji, "📺");
        assert!(!payload.devices[0].featured);
        assert_eq!(payload.devices[0].location, "Salon");
        assert!(payload.devices[0].description.contains("bulletList"));
        assert!(payload.devices[0].description.contains("Allumez"));
        assert!(payload.devices[0].description.contains("Remote on stand"));
        assert_eq!(payload.devices[1].emoji, "");
        assert_eq!(payload.devices[1].order, 1);
        assert!(payload.safety_notice.contains("Coupez l'eau."));
        assert!(payload.safety_notice.contains("paragraph"));
    }

    #[test]
    fn parses_v2_without_migration() {
        let raw = r#"{
            "safetyNotice": "{\"type\":\"doc\",\"content\":[{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"text\":\"Global\"}]}]}",
            "devices": [{
                "id": "a1",
                "name": "Oven",
                "emoji": "🔥",
                "description": "{\"type\":\"doc\",\"content\":[{\"type\":\"paragraph\"}]}",
                "featured": true,
                "order": 0,
                "location": "Kitchen",
                "manualUrl": "https://example.com",
                "safetyNote": "Hot",
                "status": "active"
            }]
        }"#;
        let payload = AppliancesPayload::parse(raw);
        assert_eq!(payload.devices.len(), 1);
        assert_eq!(payload.devices[0].name, "Oven");
        assert!(payload.devices[0].featured);
        assert_eq!(payload.devices[0].manual_url, "https://example.com");
        assert!(payload.safety_notice.contains("Global"));
    }

    #[test]
    fn guest_devices_exclude_hidden() {
        let payload = AppliancesPayload {
            devices: vec![
                Appliance {
                    id: "1".into(),
                    name: "Visible".into(),
                    status: ApplianceStatus::Active,
                    featured: true,
                    order: 1,
                    ..Default::default()
                },
                Appliance {
                    id: "2".into(),
                    name: "Hidden".into(),
                    status: ApplianceStatus::Hidden,
                    featured: true,
                    order: 0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let guest = payload.guest_devices();
        assert_eq!(guest.len(), 1);
        assert_eq!(guest[0].id, "1");
        assert_eq!(payload.featured_guest_devices().len(), 1);
    }

    #[test]
    fn description_to_html_renders_steps_and_marks() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Avant",
                        "marks": [{ "type": "bold" }]
                    }]
                },
                {
                    "type": "bulletList",
                    "content": [{
                        "type": "listItem",
                        "content": [{
                            "type": "paragraph",
                            "content": [{ "type": "text", "text": "Allumez" }]
                        }]
                    }]
                }
            ]
        })
        .to_string();
        let html = description_to_html(&doc);
        assert!(html.contains("<strong>Avant</strong>"));
        assert!(html.contains("class=\"appliance-steps\""));
        assert!(html.contains("<li>Allumez</li>"));
    }
}
