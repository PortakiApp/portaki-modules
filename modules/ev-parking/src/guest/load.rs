//! Load config for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::reveal::{
    evaluate_reveal, format_available_from, locked_message, RevealDecision, SECRET_MASK,
};

pub struct GuestData {
    pub config: ModuleConfig,
    /// The guest's locale, for the translated texts.
    pub locale: String,
    pub secrets_revealed: bool,
    pub reveal_locked_message: Option<String>,
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let now = time::now()?;
    let decision = evaluate_reveal(config.reveal_policy, now, checkin_at, &property_timezone);

    Ok(Some(GuestData {
        config,
        locale: ctx.locale.clone(),
        secrets_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone),
    }))
}

pub fn has_any_secret(config: &ModuleConfig) -> bool {
    !config.charger_pin.trim().is_empty() || !config.parking_code.trim().is_empty()
}

pub fn secret_display(data: &GuestData, plaintext: &str) -> String {
    let trimmed = plaintext.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if data.secrets_revealed {
        trimmed.to_string()
    } else {
        SECRET_MASK.to_string()
    }
}

fn property_timezone(ctx: &GuestContext) -> String {
    let from_property = ctx.property.timezone.trim();
    if !from_property.is_empty() {
        return from_property.to_string();
    }
    let from_ctx = ctx.timezone.trim();
    if !from_ctx.is_empty() {
        return from_ctx.to_string();
    }
    "Europe/Paris".to_string()
}

fn locked_banner(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed {
        return None;
    }
    let when = decision
        .available_from
        .map(|at| format_available_from(at, property_timezone));
    Some(locked_message(when.as_deref()))
}
