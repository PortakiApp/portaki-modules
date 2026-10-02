//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{ContactRow, ModuleConfig};

pub struct GuestData {
    pub contacts: Vec<ContactRow>,
    pub host_phone: String,
    pub locale: String,
}

/// The config to show, or `None` when there is nothing to show at all.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    let host_phone = host_phone(&config, ctx);
    if config.parse_contacts().is_empty() && host_phone.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        contacts: config.parse_contacts(),
        host_phone,
        locale: ctx.locale.clone(),
    }))
}

/// Le numéro de l'hôte : celui qu'il a saisi ici, sinon celui de son profil.
///
/// Le champ du module existait parce que la plateforme ne portait pas le téléphone de l'hôte ;
/// elle le porte maintenant (`ctx.host`). Le garder en surcharge plutôt que le supprimer : un
/// hôte qui a saisi un numéro ici l'a fait exprès — une ligne dédiée aux voyageurs, par exemple —
/// et le lui changer sans le prévenir serait pire que le doublon.
fn host_phone(config: &ModuleConfig, ctx: &GuestContext) -> String {
    let chosen = config.host_visible_phone.trim();
    if !chosen.is_empty() {
        return chosen.to_string();
    }
    ctx.host
        .as_ref()
        .and_then(|host| host.phone.as_deref())
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}
