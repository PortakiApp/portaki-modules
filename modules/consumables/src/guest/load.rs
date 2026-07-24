//! Load catalog + stay reports for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use super::empty::{empty_no_items_card, empty_state_if_module_not_ready};
use crate::entities::{ConsumableItem, ConsumableReport};
use crate::storage;

pub enum GuestLoad {
    Empty(Box<Surface>),
    Ready(GuestConsumablesData),
}

pub struct GuestConsumablesData {
    pub items: Vec<ConsumableItem>,
    pub reports: Vec<ConsumableReport>,
    pub locale: String,
    pub property_locale: String,
}

pub fn load_guest_consumables(ctx: &GuestContext) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(crate::ids::HOME_CARD)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let items = storage::list_items()?;
    if items.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_no_items_card(
            crate::ids::HOME_CARD,
        ))));
    }

    let reports = match ctx.guest.as_ref() {
        Some(guest) => storage::list_by_stay(guest.session_id)?,
        None => Vec::new(),
    };

    Ok(GuestLoad::Ready(GuestConsumablesData {
        items,
        reports,
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
    }))
}
