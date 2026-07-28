//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use super::body::build_events_body;
use super::load::GuestData;
use crate::time_format::format_starts_at_display;

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon("calendar")
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon("calendar")
                    .title("i18n:home.card.title"),
            ))
            .children(build_events_body(data, false)),
    )
    .with_id(crate::ids::HOME_CARD)
}

/// Compact pre-arrival prep card — icon + title + a single next-event headline.
///
/// Deliberately small: it does NOT reuse the full events body. It shows only the
/// next upcoming event (title + date). When no event is available it renders a
/// minimal card with just the title.
pub fn build_upcoming_card(data: &GuestData) -> Surface {
    let mut card = Card::new()
        .icon("calendar")
        .title("i18n:home.card.title")
        .action(Action::open_overlay(
            OverlayPresentation::BottomSheet,
            crate::ids::EXPLORE_DETAIL,
            OverlayArgs::new()
                .icon("calendar")
                .title("i18n:home.card.title"),
        ));

    if let Some(headline) = upcoming_headline(data) {
        card = card.children(vec![Component::Text(
            Text::new()
                .text(headline)
                .variant(TextVariant::Caption),
        )]);
    }

    Surface::new(card).with_id(crate::ids::UPCOMING_CARD)
}

/// One-line headline for the compact card: the next event's title, with its
/// date appended when known. `None` when there is nothing to show.
fn upcoming_headline(data: &GuestData) -> Option<String> {
    let event = data.events.first()?;
    let title = event
        .title
        .pick_with_fallback(&data.locale, &data.property_locale);
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    let when = format_starts_at_display(&event.starts_at);
    if when.trim().is_empty() {
        Some(title.to_string())
    } else {
        Some(format!("{} · {}", title, when.trim()))
    }
}
