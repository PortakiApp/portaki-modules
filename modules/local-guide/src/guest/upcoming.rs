//! Compact pre-arrival prep card for the upcoming timeline.
//!
//! Shows a single glanceable line — the number of curated spots — instead of
//! the full spot list rendered by the home card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;

pub fn build_upcoming_card(data: &GuestData) -> Surface {
    let mut card = Card::new().icon("map-pin").title("i18n:nav.local-guide");
    if let Some(headline) = headline_value(data) {
        card = card.child(Text::new().text(headline).variant(TextVariant::Body));
    }
    Surface::new(card).with_id(crate::ids::UPCOMING_CARD)
}

/// One key live value: `"6 adresses"` — the count of curated spots.
fn headline_value(data: &GuestData) -> Option<String> {
    let count = data.spots.len();
    if count == 0 {
        return None;
    }
    let count_str = count.to_string();
    Some(t!("guest.upcoming.spotCount", count = &count_str).unwrap_or_else(|_| count_str.clone()))
}
