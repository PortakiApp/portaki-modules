//! Load checklist data for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use super::empty::{empty_no_items_card, empty_state_if_module_not_ready};
use crate::entities::{Checklist, ChecklistItem};
use crate::lists;
use crate::show_when::is_checklist_available;
use crate::storage;

pub enum GuestLoad {
    Empty(Box<Surface>),
    NotYet,
    Ready(GuestChecklistData),
}

pub struct GuestChecklistData {
    /// Guest lists open right now, each with its items.
    pub lists: Vec<(Checklist, Vec<ChecklistItem>)>,
    pub completed: Vec<Uuid>,
    pub locale: String,
    pub property_locale: String,
    pub done: usize,
    pub total: usize,
    pub percent: u8,
    /// Departure instant (UTC) — rendered as the card title in the property timezone.
    pub checkout_at: Option<chrono::DateTime<chrono::Utc>>,
    pub property_timezone: String,
}

/// `departure_only` keeps the `atDeparture` lists (post-stay card).
pub fn load_guest_checklist(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    departure_only: bool,
) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let mut guest_lists = Vec::new();
    for list in storage::list_checklists()? {
        let wanted = list.audience == lists::GUEST
            && (!departure_only || list.trigger == lists::AT_DEPARTURE);
        if !wanted {
            continue;
        }
        let items = storage::items_of(list.id)?;
        if !items.is_empty() {
            guest_lists.push((list, items));
        }
    }
    if guest_lists.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_no_items_card(surface_id))));
    }

    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let now = time::now()?;
    guest_lists
        .retain(|(list, _)| is_checklist_available(&list.trigger, now, checkin_at, checkout_at));
    if guest_lists.is_empty() {
        return Ok(GuestLoad::NotYet);
    }

    let stay_id = ctx.guest.as_ref().map(|guest| guest.session_id);
    let completed: Vec<Uuid> = match stay_id {
        Some(id) => storage::list_completions(Some(id))?
            .into_iter()
            .map(|row| row.item_id)
            .collect(),
        None => Vec::new(),
    };
    let items = guest_lists.iter().flat_map(|(_, items)| items);
    let total = items.clone().count();
    let done = items.filter(|item| completed.contains(&item.id)).count();
    let percent = (done * 100).checked_div(total).unwrap_or(0) as u8;

    Ok(GuestLoad::Ready(GuestChecklistData {
        lists: guest_lists,
        completed,
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        done,
        total,
        percent,
        checkout_at,
        property_timezone: ctx.property.timezone.clone(),
    }))
}
