//! Load config for guest surfaces.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::reveal::RevealPolicy;
use serde_json::{Map, Value};

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

    // Le calendrier d'Accès (§2.3, §7), sans réglage propre : la spec n'en prévoit pas.
    let now = time::now()?;
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let (revealed, available_from) = reveal(&shared_schedule(), now, checkin_at, ctx.property_tz());
    // Jamais après le départ : un ancien voyageur ne lit pas le code du suivant.
    let ended = checkout_at.is_some_and(|checkout| now > checkout);
    // En aperçu, la plateforme remplace les secrets par `***` : rien à copier.
    let masked_by_platform = config.bin_room_code.trim() == "***";
    let code_revealed = revealed && !ended && !masked_by_platform;
    let code_reveal_at = match (code_revealed || ended, available_from) {
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
        bin_room_hours: config.bin_room_hours(&ctx.locale),
    }))
}

/// Ce que partage le premier module qui fournit `access.reveal_schedule` sur ce logement (ses
/// valeurs publiées, jamais un secret) ; rien sans lui, ou si l'hôte ne répond pas : le livret ne
/// se bloque pas là-dessus.
fn shared_schedule() -> Map<String, Value> {
    host::module::list_by_capability(portaki_sdk::capability::access::REVEAL_SCHEDULE)
        .ok()
        .and_then(|peers| peers.into_iter().next())
        .map(|peer| peer.shared)
        .unwrap_or_default()
}

/// Le code se montre-t-il à `now`, et à partir de quand, selon le calendrier partagé : `custom`
/// compte `reveal_hours` avant l'arrivée (24 sans valeur), une valeur inconnue ou absente vaut le
/// défaut d'Accès (la veille à 16 h). Sans arrivée, verrouillé — sauf `always`.
fn reveal(
    shared: &Map<String, Value>,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    tz: Option<PropertyTz>,
) -> (bool, Option<DateTime<Utc>>) {
    let policy = shared.get("reveal_policy").and_then(Value::as_str);
    if policy == Some("custom") {
        let hours = shared
            .get("reveal_hours")
            .and_then(|h| h.as_u64().or_else(|| h.as_str()?.trim().parse().ok()))
            .unwrap_or(24);
        let from = checkin_at.map(|checkin| checkin - Duration::hours(hours as i64));
        return (from.is_some_and(|from| now >= from), from);
    }
    let policy = policy
        .and_then(|raw| serde_json::from_value(Value::String(raw.into())).ok())
        .unwrap_or_default();
    let decision = RevealPolicy::evaluate(policy, now, checkin_at, tz);
    (decision.revealed, decision.available_from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn shared(value: Value) -> Map<String, Value> {
        value.as_object().cloned().unwrap_or_default()
    }

    /// Le calendrier d'Accès quand il le partage, `custom` compris ; le défaut sinon.
    #[test]
    fn the_code_follows_the_shared_schedule() {
        let checkin = Some(at("2026-07-20T14:00:00Z"));
        let now = at("2026-07-20T09:00:00Z");
        // Six heures avant l'arrivée : ouvert à 8 h.
        let custom = shared(json!({ "reveal_policy": "custom", "reveal_hours": "6" }));
        assert_eq!(
            reveal(&custom, now, checkin, None),
            (true, Some(at("2026-07-20T08:00:00Z")))
        );
        let at_checkin = shared(json!({ "reveal_policy": "at_checkin" }));
        assert_eq!(reveal(&at_checkin, now, checkin, None), (false, checkin));
        // Rien de partagé, ou une valeur inconnue : la veille à 16 h.
        for silent in [json!({}), json!({ "reveal_policy": "weekly" })] {
            assert_eq!(
                reveal(&shared(silent), now, checkin, None),
                (true, Some(at("2026-07-19T16:00:00Z")))
            );
        }
        assert_eq!(
            reveal(
                &shared(json!({ "reveal_policy": "always" })),
                now,
                None,
                None
            ),
            (true, None)
        );
        assert_eq!(reveal(&custom, now, None, None), (false, None));
    }
}
