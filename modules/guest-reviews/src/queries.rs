//! Module queries — publication readiness.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

/// Rules across fields, which the declared config cannot say: at least one platform ticked
/// (the module does nothing without one), and the Airbnb review URL while Airbnb is.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items = vec![PublishCheck {
        id: "platform".into(),
        level: PublishLevel::Required,
        ok: config.platform_airbnb || config.platform_portaki,
        label: text("publish.platform.label", &[]),
        hint: text("host.platforms.none", &[]),
    }];
    if config.platform_airbnb {
        items.push(PublishCheck {
            id: "airbnb-review-url".into(),
            level: PublishLevel::Recommended,
            ok: !config.airbnb_needs_url(),
            label: text("host.airbnb.label", &[]),
            hint: text("host.airbnb.urlRequired", &[]),
        });
    }
    Ok(PublishReadiness { items })
}
