//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::activities::{self, ActivitiesView};
use crate::config::{valid_coords, HostActivityRow, ModuleConfig, SpotRow};
use crate::tiqets::{self, TiqetsView};
use crate::viator::{self, ViatorView};

pub struct GuestData {
    pub spots: Vec<SpotRow>,
    /// Les activités que l'hôte propose lui-même (§2.13) — celles qu'on réserve avec lui.
    pub host_activities: Vec<HostActivityRow>,
    pub disclaimer: String,
    pub locale: String,
    /// Section « Activités & billets », quand elle a une destination à proposer.
    pub activities: Option<ActivitiesView>,
    /// Section Tiqets, quand elle a des produits à montrer.
    pub tiqets: Option<TiqetsView>,
    /// Section Viator, quand elle a des produits à montrer.
    pub viator: Option<ViatorView>,
    /// Repère du logement sur la carte, quand il est géocodé.
    pub property_coords: Option<(f64, f64)>,
    pub property_name: String,
    /// Le prénom de l'hôte, pour « Les adresses de Claire ». Vide quand il n'est pas connu.
    pub host_name: String,
}

/// Ce qu'il y a à montrer, ou `None` quand il n'y a rien : ni lieu, ni mention, ni section
/// activités, Tiqets ou Viator à proposer.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<Box<GuestData>>> {
    let config = ModuleConfig::load(ctx)?;
    let activities = activities::resolve(
        ctx,
        &config.activities(),
        ctx.property.address.as_deref(),
        &ctx.locale,
    );

    let tiqets = tiqets::resolve(ctx, &config.tiqets());
    let viator = viator::resolve(ctx, &config.viator());

    // La section activités se suffit à elle-même : elle sort de l'adresse du logement,
    // donc un hôte qui n'a saisi aucune adresse a tout de même quelque chose à montrer.
    // Tiqets de même, depuis la position du logement, et Viator depuis sa ville.
    // Une activité de l'hôte suffit à faire exister la carte : elle ne dépend d'aucune adresse
    // géocodée ni d'aucun fournisseur.
    let host_activities: Vec<HostActivityRow> = config
        .host_activities
        .iter()
        .filter(|row| row.is_complete(&ctx.locale))
        .take(crate::config::MAX_HOST_ACTIVITIES)
        .cloned()
        .collect();

    if config.is_empty()
        && host_activities.is_empty()
        && activities.is_none()
        && tiqets.is_none()
        && viator.is_none()
    {
        return Ok(None);
    }

    Ok(Some(Box::new(GuestData {
        spots: config.parse_spots(),
        host_activities,
        disclaimer: config.disclaimer.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
        activities,
        tiqets,
        viator,
        property_coords: ctx
            .property
            .coordinates
            .as_ref()
            .and_then(|point| valid_coords(point.lat, point.lng)),
        property_name: ctx.property.name.clone(),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
    })))
}
