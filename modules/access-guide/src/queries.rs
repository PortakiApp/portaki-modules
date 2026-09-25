//! Module queries — read host configuration (shared + active locale texts).

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use serde::Serialize;

use crate::config::{is_configured, load_config, MethodFields, ModuleConfig};
use crate::i18n::text;
use crate::texts::{lang_code, load_texts_for_host, ModuleTexts};

#[derive(Debug, Clone, Serialize)]
pub struct GetConfigResponse {
    pub config: ModuleConfig,
    pub texts: ModuleTexts,
    /// Short language code used for `texts` (`fr`, `en`, …).
    pub lang: String,
}

#[portaki_sdk::query(name = "getConfig")]
pub fn get_config(ctx: Context) -> Result<GetConfigResponse> {
    let lang = lang_code(&ctx.locale);
    let config = load_config()?;
    let texts = load_texts_for_host(&ctx.locale)?;
    Ok(GetConfigResponse {
        config,
        texts,
        lang,
    })
}

/// Blocks publication until an access method is saved, and while a code-bearing one has no code.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    if !is_configured()? {
        return Ok(PublishReadiness {
            items: vec![PublishCheck {
                id: "access-method".into(),
                level: PublishLevel::Required,
                ok: false,
                label: text("publish.access-method.label"),
                hint: text("publish.access-method.hint"),
            }],
        });
    }
    let config = load_config()?;
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
