//! Load catalog + stay reports for guest surfaces.

use portaki_sdk::prelude::*;

use crate::entities::{ConsumableItem, ConsumableReport};
use crate::storage;

pub struct GuestConsumablesData {
    pub items: Vec<ConsumableItem>,
    pub reports: Vec<ConsumableReport>,
    pub locale: String,
    pub property_locale: String,
}

/// The catalog and the stay's reports; `None` while the catalog is empty.
pub fn load_guest_consumables(ctx: &GuestContext) -> Result<Option<GuestConsumablesData>> {
    let items = storage::list_items()?;
    if items.is_empty() {
        return Ok(None);
    }

    let reports = match ctx.guest.as_ref() {
        Some(guest) => storage::list_by_stay(guest.session_id)?,
        None => Vec::new(),
    };

    Ok(Some(GuestConsumablesData {
        items,
        reports,
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
    }))
}
