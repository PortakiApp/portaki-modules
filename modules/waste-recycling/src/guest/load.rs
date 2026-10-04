//! Load config for guest surfaces.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;

use crate::config::{BinRow, DropoffRow, ModuleConfig};

pub struct GuestData {
    pub bins: Vec<BinRow>,
    pub collection_schedule: String,
    /// Les jours cochés, `mon` … `sun` — vide tant que l'hôte n'en a coché aucun.
    pub collection_days: Vec<String>,
    pub takeout_note: String,
    pub locale: String,
    /// Le fuseau du logement : « demain » se compte à l'heure du lieu, pas à celle du serveur.
    pub timezone: String,
    /// Le départ du séjour, quand la plateforme en connaît un (§2.7).
    pub checkout_at: Option<DateTime<Utc>>,
    /// Les points d'apport nommés (§3.2).
    pub dropoff_points: Vec<DropoffRow>,
    /// L'emplacement du composteur — vide quand il n'y en a pas.
    pub compost_location: String,
    pub compost_accepted: Vec<String>,
    pub compost_refused: Vec<String>,
    /// La position du logement, pour mesurer la distance d'un point d'apport.
    pub property: Option<(f64, f64)>,
    /// Le chemin jusqu'au local poubelles, une étape par ligne.
    pub bin_room_steps: Vec<String>,
    /// Le code de la porte du local, vide quand il n'y en a pas.
    pub bin_room_code: String,
    /// Les heures d'ouverture du local, vides quand il est toujours accessible.
    pub bin_room_hours: String,
}

/// Un élément par ligne, les lignes vides sautées, dix au plus (§10).
fn lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(crate::config::MAX_COMPOST_LINES)
        .map(String::from)
        .collect()
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        bins: config.parse_bins(),
        collection_schedule: config.collection_schedule.get(&ctx.locale).to_string(),
        collection_days: config.collection_days(),
        takeout_note: config.takeout_note.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
        timezone: ctx.timezone.clone(),
        checkout_at: ctx.stay.as_ref().and_then(|stay| stay.checkout_at),
        dropoff_points: config.parse_dropoff_points(),
        compost_location: if config.has_compost() {
            config.compost_location.get(&ctx.locale).trim().to_string()
        } else {
            String::new()
        },
        compost_accepted: lines(config.compost_accepted.get(&ctx.locale)),
        compost_refused: lines(config.compost_refused.get(&ctx.locale)),
        property: ctx.property.coordinates.map(|point| (point.lat, point.lng)),
        bin_room_steps: lines(config.bin_room_steps.get(&ctx.locale)),
        bin_room_code: config.bin_room_code.trim().to_string(),
        bin_room_hours: config.bin_room_hours.get(&ctx.locale).trim().to_string(),
    }))
}
