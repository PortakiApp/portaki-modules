//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    UPCOMING_CARD = "upcoming.card",
    EXPLORE_FORECAST = "explore.forecast",
    HOST_MAIN = "main",
}

define_operation_names! {
    EMAIL_CONTEXT = "emailContext",
    GET_CURRENT = "getCurrent",
    GET_FORECAST = "getForecast",
    REFRESH_FORECAST = "refreshForecast",
    UPDATE_CONFIG = "updateConfig",
}

// Must stay aligned with `portaki_sdk::contracts::platform::BOOKING_CONFIRMED`.
define_event_types! {
    BOOKING_CONFIRMED = "core.booking.confirmed",
}

/// Catalog module id — kept for SDUI action builders.
#[allow(dead_code)]
pub fn module_id() -> ModuleId {
    ModuleId::from_static("weather")
}

#[cfg(test)]
mod tests {
    use portaki_sdk::contracts::platform;

    #[test]
    fn booking_confirmed_matches_platform_contract() {
        assert_eq!(super::BOOKING_CONFIRMED, platform::BOOKING_CONFIRMED);
    }
}
