//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ModuleConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub ical_url_primary: String,
    #[serde(default)]
    pub ical_url_secondary: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let existing = load_config().unwrap_or_default();
    save_config(&ModuleConfig {
        ical_url_primary: args.ical_url_primary.trim().to_string(),
        ical_url_secondary: args.ical_url_secondary.trim().to_string(),
        last_sync_at: existing.last_sync_at,
        sync_summary: existing.sync_summary,
    })
}
