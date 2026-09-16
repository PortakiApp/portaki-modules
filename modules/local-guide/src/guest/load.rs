//! Load config for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::activities::{self, ActivitiesView};
use crate::config::{load_config, ModuleConfig, SpotRow};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub spots: Vec<SpotRow>,
    pub disclaimer: String,
    pub locale: String,
    pub property_locale: String,
    /// Section « Activités & billets », quand elle a une destination à proposer.
    pub activities: Option<ActivitiesView>,
}

pub enum GuestLoad {
    Ready(GuestData),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    let activities = activities::resolve(
        &config.activities,
        ctx.property.address.as_deref(),
        &ctx.locale,
        &ctx.property.locale,
    );

    // La section activités se suffit à elle-même : elle sort de l'adresse du logement,
    // donc un hôte qui n'a saisi aucune adresse a tout de même quelque chose à montrer.
    if config.is_empty() && activities.is_none() {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    Ok(GuestLoad::Ready(GuestData {
        spots: config.parse_spots(),
        disclaimer: config
            .disclaimer
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        activities,
    }))
}
