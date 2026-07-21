//! Module queries — appliance guide content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{Appliance, AppliancesBundle, AppliancesPayload};
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
    /// Global TipTap JSON safety notice.
    #[serde(rename = "safetyNotice")]
    pub safety_notice: String,
    /// Canonical JSON for the resolved locale.
    pub content: String,
    /// Legacy slots — kept for host tooling during transition.
    pub content_fr: String,
    pub content_en: String,
}

#[portaki_sdk::query(name = "getContent")]
pub fn get_content(ctx: Context, args: GetContentArgs) -> Result<AppliancesContentView> {
    let locale = args.locale.unwrap_or_else(|| ctx.locale.clone());
    let row = store::load_content()?;
    let (content_fr, content_en) = match row {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let payload =
        AppliancesBundle::from_row(&content_fr, &content_en).pick(&locale, &ctx.property.locale);
    let content = payload
        .to_json_string()
        .unwrap_or_else(|_| content_fr.clone());
    Ok(AppliancesContentView {
        devices: payload.devices,
        safety_notice: payload.safety_notice,
        content,
        content_fr,
        content_en,
    })
}

pub fn load_payload(ctx: &Context) -> Result<AppliancesPayload> {
    store::load_payload_for(&ctx.locale, &ctx.property.locale)
}
