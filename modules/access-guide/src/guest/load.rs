//! Load config for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, AccessStep, ModuleConfig};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub steps: Vec<AccessStep>,
    pub parking_map_url: String,
    pub arrival_video_url: String,
    pub global_note: String,
    pub address: String,
    pub gate_code: String,
    pub keybox_code: String,
    pub parking_info: String,
    pub locale: String,
    pub lat: f64,
    pub lng: f64,
}

pub enum GuestLoad {
    Ready(Box<GuestData>),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: &str) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    if config.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    let address = if config.address.trim().is_empty() {
        ctx.property.address.clone().unwrap_or_default()
    } else {
        config.address.trim().to_string()
    };

    Ok(GuestLoad::Ready(Box::new(GuestData {
        steps: config.parse_steps(),
        parking_map_url: config.parking_map_url.trim().to_string(),
        arrival_video_url: config.arrival_video_url.trim().to_string(),
        global_note: config.global_note.trim().to_string(),
        address,
        gate_code: config.gate_code.trim().to_string(),
        keybox_code: config.keybox_code.trim().to_string(),
        parking_info: config.parking_info.trim().to_string(),
        locale: ctx.locale.clone(),
        lat: ctx.property.lat,
        lng: ctx.property.lng,
    })))
}
