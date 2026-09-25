//! Module queries — publication readiness.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

/// Recommends the Airbnb review URL while Airbnb is selected — a rule on two fields, which the
/// declared config cannot say.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::read(&ctx)?;
    if !config.platform_airbnb {
        return Ok(PublishReadiness::default());
    }
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "airbnb-review-url".into(),
            level: PublishLevel::Recommended,
            ok: !config.airbnb_needs_url(),
            label: text("host.airbnb.label", &[]),
            hint: text("host.airbnb.urlRequired", &[]),
        }],
    })
}
