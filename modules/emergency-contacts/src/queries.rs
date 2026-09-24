//! Module queries — read host configuration.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{load_config, ModuleConfig};
use crate::i18n::text;

#[portaki_sdk::query(name = "getConfig")]
pub fn get_config(_ctx: Context) -> Result<ModuleConfig> {
    load_config()
}

/// Blocks publication while the host line guests call during the stay is missing.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let config = load_config()?;
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "host-phone".into(),
            level: PublishLevel::Required,
            ok: !config.host_visible_phone.trim().is_empty(),
            label: text("publish.host-phone.label"),
            hint: text("publish.host-phone.hint"),
        }],
    })
}
