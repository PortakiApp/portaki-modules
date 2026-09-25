//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod form;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::load_guest_consumables;

pub use form::render_guest_form;

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "consumables",
    label_key = "nav.consumables"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    Ok(match load_guest_consumables(&ctx)? {
        Some(data) => build_home_card(&data),
        None => empty_catalog_card(crate::ids::HOME_CARD),
    })
}

/// Nothing in the catalog: nothing the guest can report.
fn empty_catalog_card(surface_id: SurfaceId) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Package)
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.empty")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}
