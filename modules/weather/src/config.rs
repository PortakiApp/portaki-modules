//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use serde::{Deserialize, Serialize};

use crate::entities::WeatherUnits;

/// Owner-configurable module settings. The keys are the names of the host form fields: the
/// platform takes `updateConfig`, and a missing key reads as its [`Default`].
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// Temperature display unit.
    #[field(kind = "select", options = ["celsius", "fahrenheit"], label = "host.units.label")]
    pub units: WeatherUnits,
    /// Cache refresh cadence label (`1h`, `3h`, `6h`).
    #[field(kind = "select", options = ["1h", "3h", "6h"], label = "host.refresh.label")]
    pub refresh_interval: String,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            units: WeatherUnits::Celsius,
            refresh_interval: "1h".to_string(),
        }
    }
}
