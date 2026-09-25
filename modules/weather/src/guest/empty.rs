//! Guest content state when there is no weather to show — written for the guest.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

/// No forecast here: `description` says why, in guest words.
pub fn no_weather(surface_id: SurfaceId, description: &str) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.unavailable")
            .description(description)
            .icon(IconName::CloudSun),
    )
    .with_id(surface_id)
}
