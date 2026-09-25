//! Module queries — the publication check the declared config cannot express.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{MethodFields, ModuleConfig};
use crate::i18n::text;

/// A code-bearing access method needs its code — the platform already blocks on the method.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::read(&ctx)?;
    let ok = match &config.method {
        MethodFields::Keybox { .. } => config.keybox_code().is_some(),
        MethodFields::DoorCode { code, .. } => !code.trim().is_empty(),
        // A provider module issues the codes; the manual one is only a fallback.
        MethodFields::SmartLock { .. } => {
            config.smart_lock_manual_code().is_some()
                || config
                    .smart_lock_provider_module_id
                    .as_deref()
                    .is_some_and(|id| !id.trim().is_empty())
        }
        _ => return Ok(PublishReadiness::default()),
    };
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "entry-code".into(),
            level: PublishLevel::Required,
            ok,
            label: text("publish.entry-code.label"),
            hint: text("publish.entry-code.hint"),
        }],
    })
}
