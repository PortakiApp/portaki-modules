//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{ContactRow, ModuleConfig};

pub struct GuestData {
    pub contacts: Vec<ContactRow>,
    pub host_phone: String,
    pub locale: String,
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        contacts: config.parse_contacts(),
        host_phone: config.host_visible_phone.trim().to_string(),
        locale: ctx.locale.clone(),
    }))
}
