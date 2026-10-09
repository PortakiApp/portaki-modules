//! Module queries — publication readiness.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

/// The drawer's one rule: a review link must be https (spec §6). Without a link the module still
/// works — the rating alone (spec cas 1).
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items: Vec<PublishCheck> = check::https_url(config.review_url.trim())
        .map(|hint: I18nText| PublishCheck {
            id: "config.review_url".into(),
            level: PublishLevel::Required,
            ok: false,
            label: text("host.reviewUrl.label", &[]),
            hint,
        })
        .into_iter()
        .collect();
    // Page publique : un avertissement, pas un blocage. Le bloc se masque de lui-même tant
    // qu'il manque des avis consentis ; le livret, lui, n'en dépend pas.
    if config.public_enabled {
        let consented = crate::commands::consented_reviews()?;
        if let Some(key) = crate::host::public_reviews_problem(&config, &consented) {
            items.push(PublishCheck {
                id: "config.public_reviews".into(),
                level: PublishLevel::Recommended,
                ok: false,
                label: text("host.public.reviews.label", &[]),
                hint: text(key, &[]),
            });
        }
    }
    Ok(PublishReadiness { items })
}
