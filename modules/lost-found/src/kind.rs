//! Report kind wire values (ChoiceList / storage).

use portaki_sdk::prelude::*;

/// Allowed kind values on the wire.
pub const WIRE_VALUES: &[&str] = &["lost", "found"];

/// Validates and normalizes a kind string from the form.
pub fn parse_kind(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if WIRE_VALUES.contains(&trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(PortakiError::Host(format!("invalid_kind:{trimmed}")))
}
