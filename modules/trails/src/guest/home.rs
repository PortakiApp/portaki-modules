//! Guest home booklet card — pastilles par niveau et les deux premiers itinéraires.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Badge, Card, Stack};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;
use super::rows::{levels_with_counts, trail_row};
use crate::format;

/// Deux itinéraires sur la carte : de quoi donner envie, pas de quoi remplacer la liste.
const CARD_GLANCE_LIMIT: usize = 2;

pub fn build_home_card(data: &GuestData) -> Surface {
    let levels = levels_with_counts(&data.trails);
    let badges: Vec<Component> = levels
        .iter()
        .map(|(level, count)| {
            Badge::new()
                .label(count_label(format::level(level), *count))
                .into()
        })
        .collect();

    let mut children: Vec<Component> = vec![Stack::new()
        .direction(StackDirection::Horizontal)
        .gap(8.0)
        .children(badges)
        .into()];
    children.extend(
        data.trails
            .iter()
            .enumerate()
            .take(CARD_GLANCE_LIMIT)
            // Sur la carte, pas la forme : la ligne est déjà à trois mesures.
            .map(|(index, trail)| trail_row(data, trail, index, false)),
    );

    Surface::new(
        Card::new()
            .icon(IconName::Mountain)
            .title("i18n:home.card.title")
            .subtitle(count_line(data.trails.len()))
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Mountain)
                    .title("i18n:nav.trails"),
            ))
            .children(children),
    )
    .with_id(crate::guest::HOME_CARD)
}

/// « ▲ Facile · 3 ».
fn count_label(label: String, count: usize) -> String {
    t!("guest.level.count", label = label.clone(), count = count)
        .unwrap_or_else(|_| format!("{label} · {count}"))
}

/// « 6 itinéraires choisis par votre hôte », au singulier quand il n'y en a qu'un.
fn count_line(count: usize) -> String {
    let key = if count == 1 {
        "home.card.subtitle.one"
    } else {
        "home.card.subtitle"
    };
    t!(key, count = count).unwrap_or_else(|_| format!("i18n:{key}"))
}
