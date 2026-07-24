//! Report status wire values (host workflow).

use portaki_sdk::prelude::*;

/// Allowed status values on the wire.
pub const WIRE_VALUES: &[&str] = &["open", "restocked"];

/// Default when host omits status — open.
pub const DEFAULT: &str = "open";

/// Validates and normalizes a status string from the form / command.
pub fn parse_status(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if WIRE_VALUES.contains(&trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(PortakiError::Host(format!("invalid_status:{trimmed}")))
}

/// i18n key for a stored status wire value.
pub fn status_label_key(wire: &str) -> &'static str {
    match wire {
        "restocked" => "status.restocked",
        _ => "status.open",
    }
}
