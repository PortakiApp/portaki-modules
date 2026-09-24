//! Issue category wire values (ChoiceList / storage).

use portaki_sdk::prelude::*;

/// Allowed category values on the wire.
pub const WIRE_VALUES: &[&str] = &["appliance", "cleanliness", "noise", "access", "other"];

/// Issue category picked in the guest form; serde rejects any other value.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Appliance,
    Cleanliness,
    Noise,
    Access,
    Other,
}

impl Category {
    /// Wire value, as stored.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Appliance => "appliance",
            Self::Cleanliness => "cleanliness",
            Self::Noise => "noise",
            Self::Access => "access",
            Self::Other => "other",
        }
    }
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
