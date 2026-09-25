//! Guest booklet surfaces. The SDK's guest shell renders the inactive, incomplete and error
//! states.

mod form;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use home::{build_formalities_card, FormTaskState};
use load::{load_guest_pre_arrival, GuestLoad};

pub use form::render_guest_form;

/// Guest home card — Accueil formalities composer (police HostFragment + form task).
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "pre-arrival-form",
    label_key = "nav.pre-arrival-form",
    role = GuestRole::ArrivalFormality,
    embeds = HostFragmentId::PoliceForm
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    match load_guest_pre_arrival(&ctx)? {
        GuestLoad::NotYet => Ok(build_formalities_card(FormTaskState::NotYet)),
        GuestLoad::Form {
            completed: false, ..
        } => Ok(build_formalities_card(FormTaskState::Pending)),
        GuestLoad::Form {
            completed: true, ..
        }
        | GuestLoad::Locked { .. } => Ok(build_formalities_card(FormTaskState::Done)),
    }
}
