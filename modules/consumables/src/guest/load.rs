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
    /// Les demandes sont ouvertes, et le séjour n'a pas atteint son plafond.
    pub can_request: bool,
    /// L'hôte a fermé les demandes : la carte informe seulement.
    pub requests_disabled: bool,
}

/// The catalog and the stay's reports; `None` while the catalog is empty.
pub fn load_guest_consumables(ctx: &GuestContext) -> Result<Option<GuestConsumablesData>> {
    // Un produit sans nom n'a rien à afficher sur sa tuile : il attend que l'hôte le nomme.
    let items: Vec<ConsumableItem> = storage::list_items()?
        .into_iter()
        .filter(|item| !crate::labels::labels_from_item(item).is_empty())
        .collect();
    if items.is_empty() {
        return Ok(None);
    }

    let reports = match ctx.guest.as_ref() {
        Some(guest) => storage::list_by_stay(guest.session_id)?,
        None => Vec::new(),
    };

    let settings = storage::settings::read();
    let requests_disabled = !settings.requests_enabled();
    let can_request = !requests_disabled && reports.len() < settings.max_requests() as usize;
    Ok(Some(GuestConsumablesData {
        can_request,
        requests_disabled,
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
