//! Load config for guest surfaces.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::reveal::RevealPolicy;

use crate::config::{BinRow, DropoffRow, ModuleConfig};

pub struct GuestData {
    pub bins: Vec<BinRow>,
    pub collection_schedule: String,
    /// Les jours cochés, `mon` … `sun` — vide tant que l'hôte n'en a coché aucun.
    pub collection_days: Vec<String>,
    /// Quand sortir les bacs (`evening`, `before7`, `before9`).
    pub put_out: &'static str,
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
    /// Le bloc « Le local » est proposé.
    pub bin_room: bool,
    /// Où se trouve le local.
    pub bin_room_where: String,
    /// Le code de la porte du local, vide quand il n'y en a pas — en clair même masqué : c'est
    /// [`Self::code_revealed`] qui décide s'il part dans l'arbre.
    pub bin_room_code: String,
    /// Le code peut se montrer (spec Tri §2.3, « comme Accès »).
    pub code_revealed: bool,
    /// Quand il s'ouvrira, pour la tuile masquée ; `None` sans arrivée connue ou séjour fini.
    pub code_reveal_at: Option<String>,
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

    // ponytail: le calendrier par défaut d'Accès (veille 16 h), sans réglage propre — la spec
    // n'en prévoit pas (§2.3, §7) ; un champ `reveal_policy` le jour où un hôte en demande un.
    let now = time::now()?;
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let decision = RevealPolicy::default().evaluate_for(ctx, now);
    // Jamais après le départ : un ancien voyageur ne lit pas le code du suivant.
    let ended = checkout_at.is_some_and(|checkout| now > checkout);
    // En aperçu, la plateforme remplace les secrets par `***` : rien à copier.
    let masked_by_platform = config.bin_room_code.trim() == "***";
    let code_revealed = decision.revealed && !ended && !masked_by_platform;
    let code_reveal_at = match (code_revealed || ended, decision.available_from) {
        (false, Some(from)) => {
            let local = ctx.property_tz().map(|tz| tz.to_local(from));
            Some(match local {
                Some(local) => time::date_time(local, &ctx.lang()),
                None => time::date_time(from, &ctx.lang()),
            })
        }
        _ => None,
    };

    Ok(Some(GuestData {
        bins: config.parse_bins(),
        // Sans ramassage (zone rurale) : ni jours ni phrase de collecte, donc aucun bandeau.
        collection_schedule: if config.has_collection() {
            config.collection_schedule.get(&ctx.locale).to_string()
        } else {
            String::new()
        },
        collection_days: if config.has_collection() {
            config.collection_days()
        } else {
            Vec::new()
        },
        put_out: config.put_out(),
        takeout_note: config.takeout_note.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
        timezone: ctx.timezone.clone(),
        checkout_at,
        dropoff_points: config.parse_dropoff_points(),
        compost_location: if config.has_compost() {
            config.compost_location.get(&ctx.locale).trim().to_string()
        } else {
            String::new()
        },
        compost_accepted: lines(config.compost_accepted.get(&ctx.locale)),
        compost_refused: lines(config.compost_refused.get(&ctx.locale)),
        property: ctx.property.coordinates.map(|point| (point.lat, point.lng)),
        bin_room: config.has_bin_room(),
        bin_room_where: config.bin_room_where.get(&ctx.locale).trim().to_string(),
        bin_room_steps: lines(config.bin_room_steps.get(&ctx.locale)),
        bin_room_code: config.bin_room_code.trim().to_string(),
        code_revealed,
        code_reveal_at,
        bin_room_hours: config.bin_room_hours.get(&ctx.locale).trim().to_string(),
    }))
}
