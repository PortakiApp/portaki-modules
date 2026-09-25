//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::activities::{self, ActivitiesView};
use crate::config::{valid_coords, ModuleConfig, SpotRow};
use crate::tiqets::{self, TiqetsView};

pub struct GuestData {
    pub spots: Vec<SpotRow>,
    pub disclaimer: String,
    pub locale: String,
    pub property_locale: String,
    /// Section « Activités & billets », quand elle a une destination à proposer.
    pub activities: Option<ActivitiesView>,
    /// Section Tiqets, quand elle a des produits à montrer.
    pub tiqets: Option<TiqetsView>,
    /// Repère du logement sur la carte, quand il est géocodé.
    pub property_coords: Option<(f64, f64)>,
    pub property_name: String,
}

/// Ce qu'il y a à montrer, ou `None` quand il n'y a rien : ni lieu, ni mention, ni section
/// activités ou Tiqets à proposer.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<Box<GuestData>>> {
    let config = ModuleConfig::read(ctx)?;
    let activities = activities::resolve(
        &config.activities(),
        ctx.property.address.as_deref(),
        &ctx.locale,
        &ctx.property.locale,
    );

    let tiqets = tiqets::resolve(ctx, &config.tiqets());

    // La section activités se suffit à elle-même : elle sort de l'adresse du logement,
    // donc un hôte qui n'a saisi aucune adresse a tout de même quelque chose à montrer.
    // Tiqets de même, depuis la position du logement.
    if config.is_empty() && activities.is_none() && tiqets.is_none() {
        return Ok(None);
    }

    Ok(Some(Box::new(GuestData {
        spots: config.parse_spots(),
        disclaimer: config
            .disclaimer
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        activities,
        tiqets,
        property_coords: ctx
            .property
            .coordinates
            .as_ref()
            .and_then(|point| valid_coords(point.lat, point.lng)),
        property_name: ctx.property.name.clone(),
    })))
}
