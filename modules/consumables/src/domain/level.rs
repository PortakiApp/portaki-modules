//! Report level wire values (ChoiceList / storage).

use portaki_sdk::prelude::*;

/// Allowed level values on the wire.
pub const WIRE_VALUES: &[&str] = &["missing", "low"];

/// Default when guest omits level — all out.
pub const DEFAULT: &str = "missing";

/// Validates and normalizes a level string from the form / command.
pub fn parse_level(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if WIRE_VALUES.contains(&trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(PortakiError::Host(format!("invalid_level:{trimmed}")))
}

/// i18n key for a stored level wire value.
pub fn level_label_key(wire: &str) -> &'static str {
    match wire {
        "low" => "level.low",
        _ => "level.missing",
    }
}
