//! Issue category wire values (ChoiceList / storage).

use portaki_sdk::prelude::*;

/// Allowed category values on the wire.
pub const WIRE_VALUES: &[&str] = &["appliance", "cleanliness", "noise", "access", "other"];

/// Validates and normalizes a category string from the form.
pub fn parse_category(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if WIRE_VALUES.contains(&trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(PortakiError::Host(format!("invalid_category:{trimmed}")))
}

/// i18n key for a stored category wire value.
pub fn category_label_key(wire: &str) -> &'static str {
    match wire {
        "appliance" => "form.category.appliance",
        "cleanliness" => "form.category.cleanliness",
        "noise" => "form.category.noise",
        "access" => "form.category.access",
        "other" => "form.category.other",
        _ => "form.category.other",
    }
}

/// Stored value, or `other` for a value the form no longer offers.
pub fn normalize(wire: &str) -> &'static str {
    WIRE_VALUES
        .iter()
        .copied()
        .find(|value| *value == wire)
        .unwrap_or("other")
}
