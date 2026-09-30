//! Load config for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::GeoPoint;

use crate::config::{has_content, HostConfig, ModuleConfig};
use crate::reveal::{evaluate_reveal, format_available_from, locked_message, RevealDecision};
use crate::texts::ModuleTexts;

pub struct GuestData {
    pub config: ModuleConfig,
    pub texts: ModuleTexts,
    pub address: String,
    /// The property on a map; `None` while it is not geocoded — no map then.
    pub coordinates: Option<GeoPoint>,
    pub secrets_revealed: bool,
    /// Preformatted guest message when secrets are locked (dated when possible).
    pub reveal_locked_message: Option<String>,
    /// The stay is over: codes are gone for good, and the status cell says so rather than
    /// falling back to the same wording as a code not yet due.
    pub reveal_ended: bool,
    /// When secrets open, written for the guest. `None` once revealed, once ended, or with no
    /// check-in to count from. The status cell puts it under the masked value.
    pub reveal_at_label: Option<String>,
    pub stay_id: Option<Uuid>,
}

pub enum GuestLoad {
    Ready(Box<GuestData>),
    /// Nothing written by the host yet.
    Empty,
}

pub fn load_guest_data(ctx: &GuestContext) -> Result<GuestLoad> {
    let host_config = HostConfig::load(ctx)?;
    let config = host_config.to_model(&ctx.locale);
    let texts = host_config.texts(&ctx.locale);
    if !has_content(&config, &texts) {
        return Ok(GuestLoad::Empty);
    }

    let configured_address = config.address().trim();
    let address = if configured_address.is_empty() {
        ctx.property.address.clone().unwrap_or_default()
    } else {
        configured_address.to_string()
    };

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|s| s.checkout_at);
    let stay_id = ctx.stay.as_ref().map(|s| s.stay_id);
    let now = time::now()?;
    let decision = evaluate_reveal(
        config.reveal_policy,
        now,
        checkin_at,
        checkout_at,
        &property_timezone,
    );

    Ok(GuestLoad::Ready(Box::new(GuestData {
        config,
        texts,
        address,
        coordinates: ctx.property.coordinates,
        secrets_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone),
        reveal_ended: decision.ended,
        reveal_at_label: reveal_at_label(&decision, &property_timezone),
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

fn locked_banner(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed || decision.ended {
        return None;
    }
    let when = decision
        .available_from
        .map(|at| format_available_from(at, property_timezone));
    Some(locked_message(when.as_deref()))
}

/// The reveal instant alone, without the sentence around it.
///
/// The banner reads « Disponible à partir du 23/08/2026 à 16:00 », which is too long for a cell
/// of the status strip: the cell shows the date on its own line under the masked value.
fn reveal_at_label(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed || decision.ended {
        return None;
    }
    decision
        .available_from
        .map(|at| format_available_from(at, property_timezone))
}
