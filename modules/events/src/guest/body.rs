//! Shared guest SDUI body for local events.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Emphasis, Leading, LeadingVisual, Tone};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Eyebrow, InfoBanner, Link, ListItem, Map, Pill, Stack, Text,
};

use crate::time_format::parse_starts_at;

use super::load::GuestData;

pub fn build_events_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if enriched && data.show_map {
        if let Some(map) = events_map(&data.events) {
            children.push(map);
        }
    }

    if !data.disclaimer.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.disclaimer.title")
                .message(data.disclaimer.clone()),
        ));
    }

    // Sur la carte d'accueil, le prochain événement est mis en vedette : son heure en pastille,
    // son titre en grand, son lieu dessous (§2.11). En ligne comme les autres, il se lisait comme
    // le premier d'une liste alors que c'est celui qui arrive.
    let featured = (!enriched).then(|| data.events.first()).flatten();
    if let Some(event) = featured {
        children.push(headline(data, event));
        children.push(Component::Button(
            Button::new()
                .label("i18n:guest.seeAll")
                .variant(ButtonVariant::Outline)
                .action(Action::open_overlay(
                    OverlayPresentation::BottomSheet,
                    crate::guest::EXPLORE_DETAIL,
                    OverlayArgs::new()
                        .icon(IconName::Calendar)
                        .title("i18n:home.card.title"),
                )),
        ));
    }

    for (rank, event) in data.events.iter().enumerate() {
        if featured.is_some() && rank == 0 {
            continue;
        }
        // « Ensuite » sépare le premier événement des suivants (§2.11) : sur la carte, le premier
        // est mis en avant et les autres se lisent comme une suite. Pas d'intertitre s'il n'y a
        // qu'un événement — il annoncerait une suite qui n'existe pas.
        if rank == 1 {
            children.push(Component::Eyebrow(Eyebrow::new().text("i18n:guest.next")));
        }
        let title = event.title.get(&data.locale);
        let place = event.place.get(&data.locale).to_string();

        let mut subtitle_parts = Vec::new();
        if !place.trim().is_empty() {
            subtitle_parts.push(place);
        }
        let when = crate::time_format::event_when(event);
        if !when.trim().is_empty() {
            subtitle_parts.push(when);
        }
        let subtitle = subtitle_parts.join(" · ");

        let mut item = ListItem::new().title(title);
        if !subtitle.is_empty() {
            item = item.subtitle(subtitle);
        }

        if let Some(at) = parse_starts_at(&event.starts_at) {
            // L'heure a son emplacement dans un ListItem : `Leading::Icon` est la forme d'avant,
            // où un libellé passait pour un nom d'icône. La maquette montre bien une heure.
            item = item.leading(Leading::Visual(Box::new(LeadingVisual {
                time: Some(at.format("%H:%M").to_string()),
                ..LeadingVisual::default()
            })));
        } else if !enriched {
            item = item.child(Pill::new().label("i18n:guest.event.dateTbd"));
        }
        if data.is_tonight(event) {
            item = item.child(tonight_badge());
        }

        // La fiche, pas le lien tiers : la maquette donne à chaque ligne `action: detail`, et la
        // page de l'organisateur se rejoint depuis la fiche, après l'heure et le plan.
        item = item.chevron(true).action(Action::navigate(
            NavigateTarget::path(format!("events/detail/{}", event.route_id(rank))),
            None,
        ));

        if enriched {
            if let Some(note) = event.note.as_ref() {
                let text = note.get(&data.locale);
                if !text.trim().is_empty() {
                    item = item.child(Text::new().text(text).variant(TextVariant::Caption));
                }
            }
            if let Some(url) = event
                .url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
            {
                let action = Action::external(url);
                item = item.child(
                    Link::new()
                        .label("i18n:guest.openLink")
                        .href(url)
                        .action(action),
                );
            }
        }
        children.push(Component::ListItem(item));
    }

    children
}

/// L'événement qui arrive, en vedette : son heure, son titre, son lieu.
fn headline(data: &GuestData, event: &crate::config::EventRow) -> Component {
    let mut children: Vec<Component> = Vec::new();
    let when = crate::time_format::event_when(event);
    if when.trim().is_empty() {
        children.push(Component::Pill(
            Pill::new().label("i18n:guest.event.dateTbd"),
        ));
    } else {
        children.push(Component::Badge(Badge::new().label(when)));
    }
    if data.is_tonight(event) {
        children.push(tonight_badge());
    }
    children.push(Component::Text(
        Text::new()
            .text(event.title.get(&data.locale))
            .variant(TextVariant::Title),
    ));
    let place = event.place.get(&data.locale).trim().to_string();
    if !place.is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(place)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle),
        ));
    }
    Component::Stack(Stack::new().gap(4.0).children(children))
}

/// Le badge « Ce soir », en couleur de marque (§9 #2).
pub(super) fn tonight_badge() -> Component {
    Component::Badge(
        Badge::new()
            .label("i18n:guest.event.tonight")
            .tone(Tone::Primary),
    )
}

fn events_map(events: &[crate::config::EventRow]) -> Option<Component> {
    let located: Vec<_> = events.iter().filter(|e| e.has_coords()).collect();
    if located.is_empty() {
        return None;
    }

    let mut markers = Vec::new();
    let mut lat_sum = 0.0;
    let mut lng_sum = 0.0;
    let mut count = 0.0;

    for event in &located {
        let (Some(lat), Some(lng)) = (event.lat, event.lng) else {
            continue;
        };
        lat_sum += lat;
        lng_sum += lng;
        count += 1.0;
        let label = event.title.get("fr");
        let label = if label.trim().is_empty() {
            event.id.clone()
        } else {
            label.to_string()
        };
        markers.push(
            MapMarker::new(event.id.clone(), lat, lng)
                .label(label)
                .kind(MapMarkerKind::Poi),
        );
    }

    let center_lat = lat_sum / count;
    let center_lng = lng_sum / count;

    Some(Component::Map(
        Map::new()
            .viewport(MapViewport::new(center_lat, center_lng, Some(13.0)))
            .markers(markers)
            .isStatic(true)
            .interactionMode(MapInteractionMode::None),
    ))
}
