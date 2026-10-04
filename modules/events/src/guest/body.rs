//! Shared guest SDUI body for local events.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Leading, LeadingVisual};
use portaki_sdk::sdui::primitives::{
    Eyebrow, InfoBanner, Link, ListItem, Map, Pill, Pressable, Text,
};

use crate::time_format::{format_starts_at_display, parse_starts_at};

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

    for (rank, event) in data.events.iter().enumerate() {
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
        let when = format_starts_at_display(&event.starts_at);
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
            children.push(Component::ListItem(item));
        } else if let Some(url) = event
            .url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())
        {
            let action = Action::external(url);
            children.push(Component::Pressable(
                Pressable::new().action(action).child(item),
            ));
        } else {
            children.push(Component::ListItem(item));
        }
    }

    children
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
