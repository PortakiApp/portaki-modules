//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

pub(crate) mod depart;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::{load_guest_checklist, GuestLoad};

/// Guest home card — the open guest lists, progress + inline toggles (no overlay).
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "checklist",
    label_key = "home.card.title"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, HOME_CARD, false)
}

/// End-of-stay card — the departure lists, still tickable once the stay is over.
#[portaki_sdk::surface(
    guest,
    id = "post-stay.card",
    path = "post-stay",
    label_key = "home.card.title",
    role = GuestRole::PostStay
)]
pub fn render_post_stay_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, POST_STAY_CARD, true)
}

fn render_card(ctx: &GuestContext, surface_id: SurfaceId, departure_only: bool) -> Result<Surface> {
    Ok(match load_guest_checklist(ctx, departure_only)? {
        GuestLoad::NoItems => message_card(surface_id, IconName::ListChecks, "home.card.empty"),
        GuestLoad::NotYet => message_card(surface_id, IconName::ClockCircle, "home.card.notYet"),
        GuestLoad::Ready(data) => build_home_card(&data, surface_id),
    })
}

/// The card with one line of text instead of lists.
fn message_card(surface_id: SurfaceId, icon: IconName, key: &str) -> Surface {
    Surface::new(
        Card::new().icon(icon).title("i18n:home.card.title").child(
            Text::new()
                .text(format!("i18n:{key}"))
                .variant(TextVariant::Body),
        ),
    )
    .with_id(surface_id)
}
