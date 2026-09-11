//! Module commands — configuration and cache invalidation.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cache;
use crate::config::{save_config, ModuleConfig};
use crate::entities::WeatherUnits;

/// Arguments for `updateConfig`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    /// `celsius` or `fahrenheit`.
    pub units: String,
    /// `1h`, `3h`, or `6h`.
    pub refresh_interval: String,
}

impl UpdateConfigArgs {
    fn to_config(&self) -> Result<ModuleConfig> {
        let units = match self.units.as_str() {
            "celsius" => WeatherUnits::Celsius,
            "fahrenheit" => WeatherUnits::Fahrenheit,
            other => {
                return Err(PortakiError::Host(format!("unsupported units: {other}")));
            }
        };
        if !["1h", "3h", "6h"].contains(&self.refresh_interval.as_str()) {
            return Err(PortakiError::Host(format!(
                "unsupported refresh_interval: {}",
                self.refresh_interval
            )));
        }
        Ok(ModuleConfig {
            units,
            refresh_interval: self.refresh_interval.clone(),
        })
    }
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let config = args.to_config()?;
    save_config(&config)
}

#[portaki_sdk::command(name = "refreshForecast")]
pub fn refresh_forecast(ctx: Context) -> Result<()> {
    cache::invalidate(ctx.property.lat, ctx.property.lng)
}
