//! Guest booklet surfaces, one per file. The SDK renders the inactive / incomplete / error states.

mod card;
pub(crate) mod depart;
mod home;
mod home_card;
mod load;
mod post_stay_card;

pub use home_card::render_home_card;
pub use post_stay_card::render_post_stay_card;
