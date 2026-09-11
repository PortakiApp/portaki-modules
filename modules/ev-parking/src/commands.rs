//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ModuleConfig, RevealPolicy};

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub spot_label: String,
    #[serde(default)]
    pub charger_pin: String,
    #[serde(default)]
    pub parking_code: String,
    #[serde(default)]
    pub map_url: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub reveal_policy: RevealPolicy,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let existing = load_config().unwrap_or_default();
    let charger_pin = if args.charger_pin.trim().is_empty() {
        existing.charger_pin
    } else {
        args.charger_pin
    };
    let parking_code = if args.parking_code.trim().is_empty() {
        existing.parking_code
    } else {
        args.parking_code
    };
    let map_url = optional_trimmed(args.map_url);
    let instructions = optional_trimmed(args.instructions);
    save_config(&ModuleConfig {
        spot_label: args.spot_label.trim().to_string(),
        charger_pin,
        parking_code,
        map_url,
        instructions,
        reveal_policy: args.reveal_policy,
    })
}

fn optional_trimmed(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
