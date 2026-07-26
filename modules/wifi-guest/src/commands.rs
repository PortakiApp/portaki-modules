//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ModuleConfig, RevealPolicy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub ssid: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub hint: String,
    #[serde(default)]
    pub connection_steps: String,
    #[serde(default)]
    pub reveal_policy: RevealPolicy,
}

fn optional_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let existing = load_config().unwrap_or_default();
    let password = if args.password.trim().is_empty() {
        existing.password
    } else {
        args.password
    };
    save_config(&ModuleConfig {
        ssid: args.ssid.trim().to_string(),
        password,
        hint: optional_trimmed(&args.hint),
        connection_steps: optional_trimmed(&args.connection_steps),
        reveal_policy: args.reveal_policy,
    })
}
