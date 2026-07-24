//! Load config for guest surfaces.

use chrono::Utc;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, ModuleConfig};
use crate::reveal::{
    evaluate_reveal, format_available_from, locked_message, RevealDecision, SECRET_MASK,
};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub config: ModuleConfig,
    pub password_revealed: bool,
    pub reveal_locked_message: Option<String>,
}

pub enum GuestLoad {
    Ready(GuestData),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    if config.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let now = time::now().unwrap_or_else(|_| Utc::now());
    let decision = evaluate_reveal(config.reveal_policy, now, checkin_at, &property_timezone);

    Ok(GuestLoad::Ready(GuestData {
        config,
        password_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone, &ctx.locale),
    }))
}

pub fn password_display(data: &GuestData) -> String {
    let password = data.config.password.trim();
    if password.is_empty() {
        return String::new();
    }
    if data.password_revealed {
        password.to_string()
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

fn locked_banner(
    decision: &RevealDecision,
    property_timezone: &str,
    locale: &str,
) -> Option<String> {
    if decision.revealed {
        return None;
    }
    let when = decision
        .available_from
        .map(|at| format_available_from(at, property_timezone));
    Some(locked_message(when.as_deref(), locale))
}
