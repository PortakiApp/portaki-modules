//! Module queries — read host configuration.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{load_config, ModuleConfig};
use crate::i18n::text;

#[portaki_sdk::query(name = "getConfig")]
pub fn get_config(_ctx: Context) -> Result<ModuleConfig> {
    load_config()
}

/// Blocks publication without a network name. The password is only recommended: an open
/// network (captive portal) has none.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let config = load_config()?;
    let check = |id: &str, level, ok| PublishCheck {
        id: id.into(),
        level,
        ok,
        label: text(&format!("publish.{id}.label")),
        hint: text(&format!("publish.{id}.hint")),
    };
    Ok(PublishReadiness {
        items: vec![
            check(
                "ssid",
                PublishLevel::Required,
                !config.ssid.trim().is_empty(),
            ),
            check(
                "password",
                PublishLevel::Recommended,
                !config.password.trim().is_empty(),
            ),
        ],
    })
}
