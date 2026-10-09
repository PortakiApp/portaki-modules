//! Ce qui est commun à la carte, à la liste et à la fiche : la rangée d'un itinéraire.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::common::{BadgeSpec, Leading, Tone, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::ListItem;

use super::load::GuestData;
use crate::config::{TrailRow, LEVELS};
use crate::format;

/// « 1 h · 2.6 km · ↑ 80 m », et la forme en plus dans la liste.
///
/// Ce qui manque ne laisse pas de trou : un itinéraire sans dénivelé renseigné affiche sa durée et
/// sa distance, pas « ↑  m ».
pub fn stats_line(trail: &TrailRow, with_shape: bool) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(minutes) = trail.duration() {
        parts.push(format::duration(minutes));
    }
    if let Some(km) = trail.distance() {
        parts.push(format::distance(km));
    }
    if let Some(metres) = trail.elevation() {
        parts.push(format!("↑ {}", format::elevation(metres)));
    }
    if with_shape {
        if let Some(shape) = trail.shape_key() {
            parts.push(format::shape(shape));
        }
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// La rangée d'un itinéraire : son nom, ses mesures, et son niveau en fin de ligne.
pub fn trail_row(data: &GuestData, trail: &TrailRow, index: usize, with_shape: bool) -> Component {
    let mut row = ListItem::new()
        .title(data.title(trail))
        .leading(Leading::Icon("mountain".into()))
        .chevron(true)
        .action(Action::navigate(
            NavigateTarget::path(format!("trails/{}", trail.route_id(index))),
            None,
        ));
    if let Some(line) = stats_line(trail, with_shape) {
        row = row.subtitle(line);
    }
    // Le niveau en texte, jamais en couleur : une difficulté n'est pas un statut (§2.23). Hors
    // saison, un badge neutre à côté : l'itinéraire reste listé.
    let badge = data
        .off_season(trail)
        .then(|| BadgeSpec::new(GuestData::off_season_label(), Tone::Neutral));
    let text = trail.level_key().map(format::level);
    if badge.is_some() || text.is_some() {
        row = row.trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge,
            text,
            ..TrailingVisual::default()
        })));
    }
    Component::ListItem(row)
}

/// Les niveaux présents, dans l'ordre croissant, avec leur compte.
///
/// L'ordre vient de [`LEVELS`] et non des données : un hôte qui saisit d'abord sa randonnée
/// sportive ne doit pas voir « Sportif » avant « Facile ».
pub fn levels_with_counts(trails: &[TrailRow]) -> Vec<(&'static str, usize)> {
    LEVELS
        .iter()
        .map(|level| {
            (
                *level,
                trails
                    .iter()
                    .filter(|trail| trail.level_key() == Some(level))
                    .count(),
            )
        })
        .filter(|(_, count)| *count > 0)
        .collect()
}
