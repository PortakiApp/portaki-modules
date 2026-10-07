//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{ContactRow, ModuleConfig};

pub struct GuestData {
    pub contacts: Vec<ContactRow>,
    pub host_phone: String,
    pub locale: String,
    /// La locale du **logement**, d'où se tirent les numéros d'urgence du pays.
    ///
    /// Pas celle du lecteur : un voyageur francophone en Espagne verrait sinon le 15 et le 18,
    /// qui ne sonnent nulle part là-bas. Les libellés, eux, restent dans sa langue — ce sont des
    /// clés que le livret résout.
    pub property_locale: String,
    /// « Pharmacie de garde : 3237 · Hôpital d'Antibes à 3,1 km » — vide quand l'hôte n'a rien
    /// donné, et la carte se termine alors sur ses contacts.
    pub useful_line: String,
}

/// Ce qu'il y a à montrer — et il y a **toujours** quelque chose (§2.16).
///
/// Ce module ne se tait jamais : « aucun contact hôte → tuiles pays seules, jamais d'état vide,
/// "composez le 112" minimum ». Les numéros d'urgence ne viennent pas de l'hôte, ils se calculent
/// du pays du logement — un hôte qui n'a rien rempli n'est pas une raison de laisser un voyageur
/// sans numéro devant une porte.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    let host_phone = host_phone(&config, ctx);
    Ok(Some(GuestData {
        contacts: config.parse_contacts(),
        host_phone,
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        useful_line: useful_line(&config),
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

/// Les deux lignes utiles en une phrase, dans l'ordre où on les cherche : d'abord la pharmacie,
/// qu'on appelle, puis l'hôpital, où l'on va.
fn useful_line(config: &ModuleConfig) -> String {
    [
        ("guest.useful.pharmacy", config.pharmacy.trim()),
        ("guest.useful.hospital", config.hospital.trim()),
    ]
    .into_iter()
    .filter(|(_, value)| !value.is_empty())
    .filter_map(|(key, value)| t!(key, value = value).ok())
    .collect::<Vec<_>>()
    .join(" · ")
}
