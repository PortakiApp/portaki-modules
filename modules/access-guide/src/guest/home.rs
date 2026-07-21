//! Guest home booklet card.

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::body::build_access_glance;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    // Prefer nav.* — shell always ships `nav.access-guide`, so a colliding
    // `home.card.title` from another module bundle cannot overwrite this label.
    Surface::new(
        Card::new()
            .icon(json!("car"))
            .title(json!("i18n:nav.access-guide"))
            .action(json!({
                "type": "openOverlay",
                "presentation": "fullscreen",
                "surfaceRender": "explore.detail",
                "args": {
                    "icon": "car",
                    "title": "i18n:nav.access-guide"
                }
            }))
            .children(build_access_glance(data)),
    )
    .with_id("home.card")
}
