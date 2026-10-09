//! Module queries — house rules content.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RulesBundle, RulesPayload, MAX_CARD_LIMIT, MIN_CARD_LIMIT};
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
    /// Règles sur la carte d'accueil.
    pub card_limit: usize,
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
        card_limit: bundle.card_limit(),
    })
}

/// Blocks publication until at least one rule exists, in any language — and on the field errors
/// the host form shows (lengths, rules on the card), each naming its field (`config.<clé>`).
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let bundle = store::load_content()?
        .map(|row| RulesBundle::from_row(&row.content_fr, &row.content_en))
        .unwrap_or_default();
    let ok = bundle.by_lang.values().any(|payload| !payload.is_empty());
    let mut items = vec![PublishCheck {
        id: "rules".into(),
        level: PublishLevel::Required,
        ok,
        label: text("publish.rules.label"),
        hint: text("publish.rules.hint"),
    }];
    if let Some(error) = bundle.card_limit.and_then(|n| {
        portaki_sdk::config::check::between(
            f64::from(n),
            f64::from(MIN_CARD_LIMIT),
            f64::from(MAX_CARD_LIMIT),
        )
    }) {
        items.push(PublishCheck {
            id: "config.card_limit".into(),
            level: PublishLevel::Required,
            ok: false,
            label: text("host.cardLimit.label"),
            hint: error,
        });
    }
    // Une erreur par champ, dans la première langue où elle se trouve : le formulaire montre la
    // langue de l'hôte, et deux fois le même message n'apprend rien.
    let mut seen = std::collections::BTreeSet::new();
    for payload in bundle.by_lang.values() {
        for (index, item) in payload.items.iter().enumerate() {
            for (field, error) in item.problems() {
                let id = format!("config.items.{index}.{field}");
                if seen.insert(id.clone()) {
                    items.push(PublishCheck {
                        id,
                        level: PublishLevel::Required,
                        ok: false,
                        label: text(&format!("host.rule.{field}")),
                        hint: error,
                    });
                }
            }
        }
    }
    Ok(PublishReadiness { items })
}

/// Helper for guest surfaces.
pub fn load_payload(ctx: &Context) -> Result<RulesPayload> {
    let view = get_content(
        ctx.clone(),
        GetContentArgs {
            locale: Some(ctx.locale.clone()),
        },
    )?;
    Ok(RulesPayload {
        items: view.items,
        card_limit: view.card_limit,
    })
}
