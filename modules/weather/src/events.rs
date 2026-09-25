//! Platform event subscriptions.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::queries::{get_current, get_forecast, GetCurrentArgs, GetForecastArgs};
use crate::weather::has_open_weather;

/// Payload for `core.booking.confirmed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingConfirmedEvent {
    /// Booking identifier.
    pub id: Uuid,
    /// Property id (informational).
    pub property_id: Uuid,
}

// Wire string must match `ids::BOOKING_CONFIRMED` / `contracts::platform::BOOKING_CONFIRMED`.
// Macros cannot take `ids::CONST` paths (OUT_DIR emission needs the literal at expand).
#[portaki_sdk::event_handler(event_type = EventType::new("core.booking.confirmed"))]
pub fn on_booking_confirmed(ctx: Context, _event: BookingConfirmedEvent) -> Result<()> {
    // Not geocoded: nowhere to forecast, no call.
    if !has_open_weather(&ctx) || ctx.property.coordinates.is_none() {
        return Ok(());
    }

    let _ = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;
    let _ = get_forecast(
        ctx,
        GetForecastArgs {
            lat: None,
            lng: None,
            days: Some(5),
        },
    )?;
    Ok(())
}
