//! Load config for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::{ModuleConfig, Network};
use crate::reveal::{
    evaluate_reveal, format_available_from, locked_message, RevealDecision, SECRET_MASK,
};

pub struct GuestData {
    pub config: ModuleConfig,
    /// The networks with a name, in the host's order: the first is the one on the card.
    pub networks: Vec<Network>,
    /// The guest's locale, for the translated texts.
    pub locale: String,
    pub password_revealed: bool,
    pub reveal_locked_message: Option<String>,
    /// Le moment où le mot de passe s'ouvre, pour la ligne masquée.
    pub reveal_at_label: Option<String>,
}

/// What the guest surfaces show, or `None` while no network is filled in.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|s| s.checkout_at);
    let now = time::now()?;
    let decision = evaluate_reveal(
        config.reveal_policy,
        now,
        checkin_at,
        checkout_at,
        &property_timezone,
    );

    let networks = config
        .networks()
        .into_iter()
        .filter(|network| !network.ssid.trim().is_empty())
        .collect();
    Ok(Some(GuestData {
        config,
        networks,
        locale: ctx.locale.clone(),
        password_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone, &ctx.locale),
        reveal_at_label: reveal_at_label(&decision, &property_timezone),
    }))
}

/// The password as the guest may see it now: in clear once revealed, masked before ; nothing for
/// an open network, whatever the host typed before choosing it (spec §3).
pub fn password_display(data: &GuestData, network: &Network) -> String {
    if network.security == crate::config::WifiSecurity::Nopass {
        return String::new();
    }
    let password = network.password.trim();
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
    if decision.revealed || decision.ended {
        return None;
    }
    let when = decision
        .available_from
        .map(|at| format_available_from(at, property_timezone));
    Some(locked_message(when.as_deref(), locale))
}

/// Quand les secrets s'ouvrent, écrit pour le voyageur. `None` une fois révélés, une fois le séjour
/// fini, ou sans date calculable — la ligne masquée se passe alors de promesse plutôt que d'en
/// inventer une.
fn reveal_at_label(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed || decision.ended {
        return None;
    }
    decision
        .available_from
        .map(|at| format_available_from(at, property_timezone))
}
