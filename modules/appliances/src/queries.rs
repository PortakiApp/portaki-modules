//! Module queries — appliance guide content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{load_from_locale_slots, Appliance, AppliancesPayload};
use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetContentArgs {
    pub locale: Option<String>,
    /// Optional device id for detail views (also used when overlay args are forwarded).
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliancesContentView {
    pub devices: Vec<Appliance>,
    /// Canonical JSON (single language).
    pub content: String,
    /// Legacy slots — kept for host tooling during transition.
    pub content_fr: String,
    pub content_en: String,
}

#[portaki_sdk::query(name = "getContent")]
pub fn get_content(ctx: Context, args: GetContentArgs) -> Result<AppliancesContentView> {
    let _locale = args.locale.unwrap_or_else(|| ctx.locale.clone());
    let row = store::load_content()?;
    let (content_fr, content_en) = match row {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let payload = load_from_locale_slots(&content_fr, &content_en);
    let content = payload
        .to_json_string()
        .unwrap_or_else(|_| content_fr.clone());
    Ok(AppliancesContentView {
        devices: payload.devices,
        content,
        content_fr,
        content_en,
    })
}

pub fn load_payload(ctx: &Context) -> Result<AppliancesPayload> {
    let _ = ctx;
    store::load_payload()
}
