//! Load config for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{has_content, load_config, ModuleConfig};
use crate::reveal::{evaluate_reveal, format_available_from, locked_message, RevealDecision};
use crate::texts::{load_texts_for_guest, ModuleTexts};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub config: ModuleConfig,
    pub texts: ModuleTexts,
    pub address: String,
    pub locale: String,
    pub lat: f64,
    pub lng: f64,
    pub secrets_revealed: bool,
    /// Preformatted guest message when secrets are locked (dated when possible).
    pub reveal_locked_message: Option<String>,
    pub stay_id: Option<Uuid>,
}

pub enum GuestLoad {
    Ready(Box<GuestData>),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: &str) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    let texts = load_texts_for_guest(&ctx.locale, &ctx.property.locale).unwrap_or_default();
    if !has_content(&config, &texts) {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    let configured_address = config.address().trim();
    let address = if configured_address.is_empty() {
        ctx.property.address.clone().unwrap_or_default()
    } else {
        configured_address.to_string()
    };

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let stay_id = ctx.stay.as_ref().map(|s| s.stay_id);
    let now = time::now().unwrap_or_else(|_| Utc::now());
    let decision = evaluate_reveal(config.reveal_policy, now, checkin_at, &property_timezone);

    Ok(GuestLoad::Ready(Box::new(GuestData {
        config,
        texts,
        address,
        locale: ctx.locale.clone(),
        lat: ctx.property.lat,
        lng: ctx.property.lng,
        secrets_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone, &ctx.locale),
        stay_id,
    })))
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
        .map(|at| format_available_from(at, property_timezone, locale));
    Some(locked_message(locale, when.as_deref()))
}
