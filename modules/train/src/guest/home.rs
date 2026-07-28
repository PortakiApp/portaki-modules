//! Guest home booklet card — mixed-destination departure board glance.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Card, Text, TimedEntry};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{home_board, station_caption, DEFAULT_STATION_LABEL, MODULE_ICON};

pub fn build_home_card(_ctx: &GuestContext) -> Surface {
    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(station_caption())
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )];
    children.extend(home_board().into_iter().map(board_entry_component));

    Surface::new(
        Card::new()
            .icon(MODULE_ICON)
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(MODULE_ICON)
                    .title("i18n:home.card.title"),
            ))
            .children(children),
    )
    .with_id(crate::ids::HOME_CARD)
}

/// Compact pre-arrival prep card — icon + title + a single next-departure line.
///
/// Deliberately small: no full board, just the nearest departure headline
/// (e.g. "Antibes → Nice-Ville · 08:12"). Falls back to the station label alone
/// when no departures are configured.
pub fn build_upcoming_card(_ctx: &GuestContext) -> Surface {
    let children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(upcoming_headline())
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )];

    Surface::new(
        Card::new()
            .icon(MODULE_ICON)
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(MODULE_ICON)
                    .title("i18n:home.card.title"),
            ))
            .children(children),
    )
    .with_id(crate::ids::UPCOMING_CARD)
}

/// One-line headline for the compact card: nearest departure, or the station
/// label alone when the board is empty.
fn upcoming_headline() -> String {
    match home_board().into_iter().next() {
        Some(entry) => format!(
            "{} → {} · {}",
            DEFAULT_STATION_LABEL, entry.destination, entry.time
        ),
        None => DEFAULT_STATION_LABEL.to_string(),
    }
}

fn board_entry_component(entry: crate::content::BoardEntry) -> Component {
    Component::TimedEntry(
        TimedEntry::new()
            .time(entry.time)
            .title(entry.destination)
            .subtitle(entry.platform),
    )
}
