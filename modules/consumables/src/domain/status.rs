//! Report status wire values (host workflow): À traiter → Prévu → Livré.

use portaki_sdk::prelude::*;

/// Allowed status values on the wire.
pub const WIRE_VALUES: &[&str] = &["open", "planned", "restocked"];

/// Default when host omits status — open (« À traiter »).
pub const DEFAULT: &str = "open";

/// « Prévu » : l'hôte a vu la demande et passera.
pub const PLANNED: &str = "planned";

/// « Livré ».
pub const RESTOCKED: &str = "restocked";

/// Validates and normalizes a status string from the form / command.
pub fn parse_status(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if WIRE_VALUES.contains(&trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(PortakiError::Host(format!("invalid_status:{trimmed}")))
}

/// Pas encore livré (à traiter ou prévu) : le produit manque toujours, un doublon n'a pas lieu
/// d'être.
pub fn is_pending(wire: &str) -> bool {
    wire != RESTOCKED
}

/// i18n key for a stored status wire value.
pub fn status_label_key(wire: &str) -> &'static str {
    match wire {
        RESTOCKED => "status.restocked",
        PLANNED => "status.planned",
        _ => "status.open",
    }
}
