//! Module queries — house rules content.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RulesBundle, RulesPayload};
use crate::i18n::text;
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

#[portaki_sdk::query(
    name = "getContent",
    example(label = "Règlement en français", input = r#"{"locale":"fr-FR"}"#),
    example(label = "Règlement en anglais", input = r#"{"locale":"en-US"}"#)
)]
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

/// Blocks publication until at least one rule exists, in any language.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let ok = store::load_content()?.is_some_and(|row| {
        RulesBundle::from_row(&row.content_fr, &row.content_en)
            .by_lang
            .values()
            .any(|payload| !payload.is_empty())
    });
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "rules".into(),
            level: PublishLevel::Required,
            ok,
            label: text("publish.rules.label"),
            hint: text("publish.rules.hint"),
        }],
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
