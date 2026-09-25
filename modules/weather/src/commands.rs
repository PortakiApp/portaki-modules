//! Module commands — cache invalidation.

use portaki_sdk::prelude::*;

use crate::cache;

#[portaki_sdk::command(name = "refreshForecast")]
pub fn refresh_forecast(ctx: Context) -> Result<()> {
    match ctx.property.coordinates {
        Some(point) => cache::invalidate(point.lat, point.lng),
        // Not geocoded: nothing was ever cached.
        None => Ok(()),
    }
}
