//! Guest booklet surfaces. The SDK's guest shell renders the inactive, incomplete and error
//! states.

mod form;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::load_guest_reports;

pub use form::render_guest_form;

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "issue-report",
    label_key = "nav.issue-report"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let config = crate::config::ModuleConfig::load(&ctx)?;
    let stay = ctx.stay.as_ref();
    let open = config.open_at(
        portaki_sdk::host::time::now()?,
        stay.and_then(|stay| stay.checkin_at),
        stay.and_then(|stay| stay.checkout_at),
    );
    let auto_reply =
        Some(config.auto_reply.get(&ctx.locale).trim().to_string()).filter(|text| !text.is_empty());
    Ok(build_home_card(
        &load_guest_reports(&ctx)?,
        open,
        auto_reply,
    ))
}
