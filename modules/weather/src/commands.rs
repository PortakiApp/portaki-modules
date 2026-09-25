//! Module commands — cache invalidation.

use portaki_sdk::prelude::*;

use crate::cache;

#[portaki_sdk::command(name = "refreshForecast")]
pub fn refresh_forecast(ctx: Context) -> Result<()> {
    cache::invalidate(ctx.property.lat, ctx.property.lng)
}
