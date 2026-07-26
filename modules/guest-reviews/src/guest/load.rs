//! Load config for guest surfaces — only feasible selected platforms.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, ModuleConfig};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub show_airbnb: bool,
    pub show_portaki: bool,
    pub show_qr: bool,
    pub airbnb_url: Option<String>,
    pub thank_you: String,
    pub property_name: String,
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
    let show_airbnb = config.airbnb_feasible();
    let show_portaki = config.portaki_feasible();
    if !show_airbnb && !show_portaki {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    Ok(GuestLoad::Ready(GuestData {
        show_airbnb,
        show_portaki,
        show_qr: config.show_qr_code && show_airbnb,
        airbnb_url: config.airbnb_url(),
        thank_you: config
            .thank_you_message
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        property_name: ctx.property.name.clone(),
    }))
}
