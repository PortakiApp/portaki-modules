//! Load catalog + stay reports for guest surfaces.

use portaki_sdk::prelude::*;

use crate::entities::{ConsumableItem, ConsumableReport};
use crate::storage;

pub struct GuestConsumablesData {
    pub items: Vec<ConsumableItem>,
    pub reports: Vec<ConsumableReport>,
    pub locale: String,
    pub property_locale: String,
    /// Le délai que l'hôte annonce, dans la langue du voyageur. `None` quand il n'a rien promis.
    pub restock_delay: Option<String>,
    /// Le prénom de l'hôte, pour « Claire réapprovisionne sous 24 h ». Vide quand il est inconnu.
    pub host_name: String,
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
        restock_delay: storage::restock_delay::read()
            .map(|text| text.get(&ctx.locale).trim().to_string())
            .filter(|delay| !delay.is_empty()),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
    }))
}
