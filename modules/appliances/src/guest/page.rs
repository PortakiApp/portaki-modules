//! Guest explore detail — full appliance list (Booklet page body).

use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::primitives::{FilterBar, FilterChip, Stack};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::Component;
use serde::Serialize;

use super::home::devices_list;
use crate::content::AppliancesPayload;

/// Au-delà de ce nombre, la liste gagne un filtre par pièce en tête (§2.4).
///
/// En dessous, les cartes de pièces tiennent à l'écran et le filtre ne ferait que s'ajouter à ce
/// qu'on lit déjà.
const FILTER_THRESHOLD: usize = 20;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomParams<'a> {
    room: &'a str,
}

/// La liste complète : une carte par pièce (§2.4), précédée d'un filtre quand il y a trop
/// d'appareils pour tout parcourir. Les cartes viennent de `devices_list`, qui sert aussi l'état
/// vide — la page ne les enveloppe plus dans une carte de plus.
pub fn build_detail_page(payload: &AppliancesPayload, room: Option<&str>) -> Surface {
    let total = payload.guest_devices().len();
    let rooms = room_names(payload);
    let selected = room
        .map(str::trim)
        .filter(|room| !room.is_empty() && rooms.iter().any(|name| name == room));

    let mut children: Vec<Component> = Vec::new();
    if total > FILTER_THRESHOLD && rooms.len() > 1 {
        children.push(room_filter(&rooms, selected));
    }
    children.extend(devices_list(payload, selected));

    Surface::new(Stack::new().gap(12.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}

/// Les pièces, dans l'ordre où elles apparaissent.
fn room_names(payload: &AppliancesPayload) -> Vec<String> {
    payload
        .guest_devices_by_room()
        .into_iter()
        .filter_map(|(room, _)| room)
        .collect()
}

/// « Toutes · Cuisine · Salon » en tête de liste.
///
/// Toucher une pastille recharge la page : le livret rend, le module ne garde pas d'état.
fn room_filter(rooms: &[String], selected: Option<&str>) -> Component {
    let mut chips: Vec<Component> =
        vec![chip("i18n:explore.detail.room.all", "", selected.is_none())];
    chips.extend(
        rooms
            .iter()
            .map(|room| chip(room, room, selected == Some(room.as_str()))),
    );
    Component::FilterBar(FilterBar::new().children(chips))
}

fn chip(label: &str, room: &str, is_selected: bool) -> Component {
    Component::FilterChip(
        FilterChip::new()
            .label(label.to_string())
            .selected(is_selected)
            .action(Action::navigate(
                NavigateTarget::path("appliances"),
                Some(portaki_sdk::sdui::json_value(RoomParams { room })),
            )),
    )
}
