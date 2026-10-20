//! La liste complète : une carte par pièce, et un filtre par pièce quand elle est longue (§2.4).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::SurfaceLevel;
use portaki_sdk::sdui::primitives::{Card, EmptyState, FilterBar, FilterChip, Stack};

use super::{device_list_item, no_appliances};
use crate::domain::Guide;

/// Au-delà, la liste gagne un filtre par pièce en tête.
const FILTER_THRESHOLD: usize = 20;

#[portaki_sdk::wire(serialize)]
struct RoomParams<'a> {
    room: &'a str,
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "appliances",
    label_key = "nav.appliances"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let guide = Guide::load(&ctx)?;
    if guide.is_empty() {
        return Ok(no_appliances(EXPLORE_DETAIL));
    }
    // `room` arrive par les paramètres de route quand le voyageur a touché un filtre.
    let wanted = ctx.input.get("room").and_then(|v| v.as_str());
    let rooms: Vec<String> = guide.by_room().into_iter().filter_map(|(r, _)| r).collect();
    let selected = wanted
        .map(str::trim)
        .filter(|room| !room.is_empty() && rooms.iter().any(|name| name == room));
    let mut children: Vec<Component> = Vec::new();
    if guide.devices.len() > FILTER_THRESHOLD && rooms.len() > 1 {
        children.push(room_filter(&rooms, selected));
    }
    children.extend(room_cards(&guide, selected));
    Ok(Surface::new(Stack::new().gap(12.0).children(children)).with_id(EXPLORE_DETAIL))
}

/// Une carte par pièce ; un seul groupe : pas de titre, la pièce est déjà sur chaque ligne.
fn room_cards(guide: &Guide, only: Option<&str>) -> Vec<Component> {
    let rooms: Vec<_> = guide
        .by_room()
        .into_iter()
        .filter(|(room, _)| only.is_none() || room.as_deref() == only)
        .collect();
    if rooms.is_empty() {
        return vec![EmptyState::new()
            .title("i18n:explore.detail.empty.title")
            .description("i18n:explore.detail.empty.description")
            .icon(IconName::Plug)
            .into()];
    }
    let titled = rooms.len() > 1;
    rooms
        .into_iter()
        .map(|(room, devices)| {
            let mut card = Card::new().surface(SurfaceLevel::Elevated);
            if titled {
                card = card
                    .icon(IconName::Plug)
                    .title(room.unwrap_or_else(|| "i18n:explore.detail.room.other".to_string()));
            }
            card.children(devices.into_iter().map(device_list_item).collect())
                .into()
        })
        .collect()
}

/// « Toutes · Cuisine · Salon » ; toucher une pastille recharge la page.
fn room_filter(rooms: &[String], selected: Option<&str>) -> Component {
    let mut chips = vec![chip("i18n:explore.detail.room.all", "", selected.is_none())];
    chips.extend(
        rooms
            .iter()
            .map(|room| chip(room, room, selected == Some(room.as_str()))),
    );
    FilterBar::new().children(chips).into()
}

fn chip(label: &str, room: &str, selected: bool) -> Component {
    FilterChip::new()
        .label(label.to_string())
        .selected(selected)
        .action(Action::navigate(
            NavigateTarget::path("appliances"),
            Some(json_value(RoomParams { room })),
        ))
        .into()
}
