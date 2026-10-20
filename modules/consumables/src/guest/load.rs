//! Load catalog + stay reports for guest surfaces.

use portaki_sdk::prelude::*;

use uuid::Uuid;

use crate::{Entry, Restocks};

/// Un produit, dans la langue du voyageur.
pub struct Shown {
    pub id: Uuid,
    pub name: String,
    pub location: String,
    pub emoji: String,
}

pub struct GuestConsumablesData {
    pub items: Vec<Shown>,
    pub reports: Vec<Entry>,
    /// Le délai que l'hôte annonce, dans la langue du voyageur ; « dans la journée » quand il n'a
    /// rien écrit.
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
    let products = crate::Products::load(ctx)?;
    let items: Vec<Shown> = crate::domain::product::named_products(ctx)?
        .iter()
        .map(|product| Shown {
            id: product.uuid(),
            name: product.label.for_ctx(ctx).to_string(),
            location: product.location.for_ctx(ctx).to_string(),
            emoji: product.emoji.clone(),
        })
        .collect();
    if items.is_empty() {
        return Ok(None);
    }

    let reports = match ctx.guest.as_ref() {
        Some(guest) => Restocks::for_stay(ctx, guest.session_id)?,
        None => Vec::new(),
    };

    let settings = products.meta();
    let requests_disabled = !settings.enabled();
    let can_request = !requests_disabled && reports.len() < settings.max() as usize;
    Ok(Some(GuestConsumablesData {
        can_request,
        requests_disabled,
        items,
        reports,
        // Rien d'écrit par l'hôte : « dans la journée », le défaut de la spec (§2.2).
        restock_delay: Some(settings.restock_delay.get(&ctx.locale).trim().to_string())
            .filter(|delay| !delay.is_empty())
            .or_else(|| t!("form.restockDelay.default").ok()),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
    }))
}
