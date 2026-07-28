//! Compact pre-arrival prep card for the upcoming timeline.
//!
//! Unlike the home card, this renders a small glanceable line — current
//! temperature plus a short condition label — never the full forecast body.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::weather::{convert_temp, format_temp_label};

use super::load::GuestWeatherData;

pub fn build_upcoming_card(data: &GuestWeatherData) -> Surface {
    let mut card = Card::new().icon("cloud-sun").title("i18n:nav.weather");
    if let Some(headline) = headline_value(data) {
        card = card.child(Text::new().text(headline).variant(TextVariant::Body));
    }
    Surface::new(card).with_id(crate::ids::UPCOMING_CARD)
}

/// One key live value: `"26°C · Ensoleillé"` from current conditions.
fn headline_value(data: &GuestWeatherData) -> Option<String> {
    let temp = convert_temp(data.current.temp_c, data.units);
    let temp_label = format_temp_label(temp, data.units.sdui_unit(), true);
    let condition = t!(&data.current.description_key)
        .ok()
        .map(|label| label.trim().to_string())
        .filter(|label| !label.is_empty());
    match condition {
        Some(condition) => Some(format!("{temp_label} · {condition}")),
        None => Some(temp_label),
    }
}
