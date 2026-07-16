//! Host dashboard surface — stub until React `appliances-editor-v1` (iteration C).
//!
//! Commands (`saveAppliance` / `deleteAppliance` / `reorderAppliances`) remain the API;
//! this Wasm form is not the long-term editor.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Page, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::description_plain_text;
use crate::store;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let payload = store::load_payload().unwrap_or_default();
    let count = payload.devices.len();
    let featured = payload.featured_count();

    let summary = format!(
        "{count} appareil(s) · {featured} mis en avant. L’éditeur React (appliances-editor-v1) remplace ce formulaire Wasm."
    );

    let mut stack_children: Vec<Component> = vec![
        Text::new()
            .text(json!("i18n:surface.host.main.subtitle"))
            .variant(json!("body"))
            .into(),
        Text::new()
            .text(json!(summary))
            .variant(json!("caption"))
            .into(),
        Text::new()
            .text(json!("Commands: saveAppliance, deleteAppliance, reorderAppliances."))
            .variant(json!("caption"))
            .into(),
    ];

    for device in payload.devices.iter().take(10) {
        let preview = description_plain_text(&device.description);
        let preview_snip: String = preview
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(48)
            .collect();
        let line = format!(
            "{} {} — {}{}",
            if device.emoji.is_empty() {
                "·"
            } else {
                device.emoji.as_str()
            },
            device.name,
            if device.featured { "featured" } else { "—" },
            if preview_snip.is_empty() {
                String::new()
            } else {
                format!(" · {preview_snip}")
            }
        );
        stack_children.push(
            Text::new()
                .text(json!(line))
                .variant(json!("body"))
                .into(),
        );
    }

    Surface::new(
        Page::new()
            .title(json!("i18n:surface.host.main.title"))
            .child(Stack::new().gap(json!(8)).children(stack_children)),
    )
    .with_id("main")
}
