//! Module queries — house rules content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RulesBundle, RulesPayload};
use crate::store;

/// Arguments for `getContent` (locale optional — defaults to context locale).
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetContentArgs {
    pub locale: Option<String>,
}

/// Guest/host view of rules content for one locale.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RulesContentView {
    pub items: Vec<crate::content::RuleItem>,
    pub content_fr: String,
    pub content_en: String,
}

#[portaki_sdk::query(name = "getContent")]
pub fn get_content(ctx: Context, args: GetContentArgs) -> Result<RulesContentView> {
    let locale = args.locale.unwrap_or_else(|| ctx.locale.clone());
    let row = store::load_content()?;
    let (content_fr, content_en) = match row {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let bundle = RulesBundle::from_row(&content_fr, &content_en);
    let payload = bundle.pick(&locale, &ctx.property.locale);
    Ok(RulesContentView {
        items: payload.items,
        content_fr,
        content_en,
    })
}

/// Helper for guest surfaces.
pub fn load_payload(ctx: &Context) -> Result<RulesPayload> {
    let view = get_content(
        ctx.clone(),
        GetContentArgs {
            locale: Some(ctx.locale.clone()),
        },
    )?;
    Ok(RulesPayload { items: view.items })
}
