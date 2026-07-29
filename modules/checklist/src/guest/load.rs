//! Load checklist data for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use super::empty::{empty_no_items_card, empty_state_if_module_not_ready};
use crate::config::load_config;
use crate::entities::ChecklistItem;
use crate::show_when::is_checklist_available;
use crate::storage;

pub enum GuestLoad {
    Empty(Box<Surface>),
    NotYet,
    Ready(GuestChecklistData),
}

pub struct GuestChecklistData {
    pub items: Vec<ChecklistItem>,
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

pub fn load_guest_checklist(ctx: &GuestContext) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(crate::ids::HOME_CARD)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let items = storage::list_items()?;
    if items.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_no_items_card(
            crate::ids::HOME_CARD,
        ))));
    }

    let config = load_config().unwrap_or_default();
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let now = time::now().unwrap_or_else(|_| chrono::Utc::now());
    if !is_checklist_available(config.show_when, now, checkin_at, checkout_at) {
        return Ok(GuestLoad::NotYet);
    }

    let stay_id = ctx.guest.as_ref().map(|guest| guest.session_id);
    let completed = match stay_id {
        Some(id) => storage::list_completed_item_ids(id)?,
        None => Vec::new(),
    };
    let total = items.len();
    let done = items
        .iter()
        .filter(|item| completed.contains(&item.id))
        .count();
    let percent = (done * 100).checked_div(total).unwrap_or(0) as u8;

    Ok(GuestLoad::Ready(GuestChecklistData {
        items,
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
