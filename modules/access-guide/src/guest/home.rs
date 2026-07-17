//! Guest home booklet card.

use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::body::build_access_glance;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    let navigate = serde_json::to_value(Action::Navigate {
        to: "access-guide".to_string(),
        params: None,
    })
    .unwrap_or(json!({}));

    Surface::new(
        Card::new()
            .icon(json!("car"))
            .title(json!("i18n:home.card.title"))
            .action(navigate)
            .children(build_access_glance(data)),
    )
    .with_id("home.card")
}
