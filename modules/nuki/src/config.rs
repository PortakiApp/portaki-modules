//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use serde::{Deserialize, Serialize};

/// The keys are the names of the host form fields: the platform takes `updateConfig`.
/// Nothing is required on its own: « keypad code, or remote unlock (Nuki Web key + lock ID) »
/// spans the connector, so `publishReadiness` carries it. The platform does not trim: read
/// through the `*_trimmed` accessors.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(label = "host.smartlockId.label")]
    pub smartlock_id: String,
    #[field(secret, label = "host.keypadCode.label")]
    pub keypad_code: String,
    #[field(label = "host.deviceName.label")]
    pub device_name: String,
}

impl ModuleConfig {
    pub fn keypad_code_trimmed(&self) -> &str {
        self.keypad_code.trim()
    }

    pub fn smartlock_id_trimmed(&self) -> &str {
        self.smartlock_id.trim()
    }
}
