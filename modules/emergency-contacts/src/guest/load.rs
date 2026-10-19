//! Load config for guest surfaces.

use chrono::{Datelike, Timelike};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::hours;

use crate::config::{ContactRow, ModuleConfig};

pub struct GuestData {
    pub contacts: Vec<ContactRow>,
    pub host_phone: String,
    pub locale: String,
    /// Le pays du **logement** (`ctx.property.country`), d'où se tirent les numéros d'urgence
    /// ([`crate::numbers::country_of`]).
    ///
    /// Pas la langue du lecteur : un anglophone en France doit voir le 15 et le 18 autant qu'un
    /// francophone. Les libellés, eux, restent dans sa langue — ce sont des clés que le livret
    /// résout.
    pub property_country: Option<String>,
    /// « Pharmacie : Pharmacie du Port · Hôpital : CHU d'Antibes (3.1 km) » — les lieux sans
    /// numéro ; vide quand l'hôte n'en a pas donné.
    pub useful_line: String,
    /// La plage où l'hôte répond, quand il n'est pas joignable 24 h/24.
    pub host_hours: Option<(String, String)>,
    /// Pharmacie, hôpital, médecin avec un numéro : `(clé du libellé, nom, téléphone, distance)`.
    /// La distance quand la place et le logement sont tous deux placés (§2.3 : « Rangée +
    /// distance »).
    pub health: Vec<(&'static str, String, String, Option<String>)>,
    /// « Ouverte jusqu'à 19:30 » ou « Fermée », sous la pharmacie, quand l'hôte a donné ses
    /// horaires.
    pub pharmacy_state: Option<String>,
    /// La pharmacie de garde du pays (le 3237 en France), une rangée qu'on appelle — même sans
    /// pharmacie saisie (spec §9 cas 4).
    pub on_duty_pharmacy: Option<&'static str>,
}

/// Ce qu'il y a à montrer — et il y a **toujours** quelque chose (§2.16).
///
/// Ce module ne se tait jamais : « aucun contact hôte → tuiles pays seules, jamais d'état vide,
/// "composez le 112" minimum ». Les numéros d'urgence ne viennent pas de l'hôte, ils se calculent
/// du pays du logement — un hôte qui n'a rien rempli n'est pas une raison de laisser un voyageur
/// sans numéro devant une porte.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    let home = ctx.property.coordinates.map(|home| (home.lat, home.lng));
    let away = |lat: Option<f64>, lng: Option<f64>| {
        Some((lat?, lng?))
            .zip(home)
            .map(|(place, home)| distance(home, place))
    };
    // « Afficher mon numéro » décoché : pas de rangée hôte, les tuiles et les contacts restent.
    let host_phone = if config.show_host() {
        host_phone(&config, ctx)
    } else {
        String::new()
    };
    Ok(Some(GuestData {
        contacts: config.parse_contacts(),
        host_phone,
        locale: ctx.locale.clone(),
        property_country: ctx.property.country.clone(),
        useful_line: useful_line(&config, &away),
        host_hours: config
            .host_hours()
            .map(|(from, to)| (from.to_string(), to.to_string())),
        health: [
            (
                "guest.health.pharmacy",
                &config.pharmacy,
                &config.pharmacy_phone,
                away(config.pharmacy_lat, config.pharmacy_lng),
            ),
            (
                "guest.health.hospital",
                &config.hospital,
                &config.hospital_phone,
                away(config.hospital_lat, config.hospital_lng),
            ),
            (
                "guest.health.doctor",
                &config.doctor,
                &config.doctor_phone,
                away(config.doctor_lat, config.doctor_lng),
            ),
        ]
        .into_iter()
        .filter(|(_, _, phone, _)| !phone.trim().is_empty())
        .map(|(key, name, phone, distance)| {
            (
                key,
                name.trim().to_string(),
                phone.trim().to_string(),
                distance,
            )
        })
        .collect(),
        pharmacy_state: pharmacy_state(&config.pharmacy_hours, &ctx.property.timezone),
        on_duty_pharmacy: crate::numbers::on_duty_pharmacy(ctx.property.country.as_deref()),
    }))
}

/// « Ouverte jusqu'à 19:30 » ou « Fermée », à l'heure du logement. Sans horaires, rien.
fn pharmacy_state(week: &str, timezone: &str) -> Option<String> {
    if week.trim().is_empty() {
        return None;
    }
    let now = time::now().ok()?;
    let local = match PropertyTz::parse(timezone) {
        Some(tz) => tz.to_local(now).naive_local(),
        None => now.naive_utc(),
    };
    let day = local.weekday().num_days_from_monday() as usize;
    let minutes = local.hour() * 60 + local.minute();
    match open_until(&hours::parse_week(week), day, minutes) {
        Some(close) => t!("guest.pharmacy.openUntil", time = close).ok(),
        None => t!("guest.pharmacy.closed").ok(),
    }
}

/// L'heure de fermeture si c'est ouvert le jour `day` (lundi = 0) à `minutes` après minuit. Une
/// plage qui passe minuit court sur le lendemain matin.
fn open_until(week: &[Vec<hours::Range>; 7], day: usize, minutes: u32) -> Option<String> {
    // `parse_week` rend des heures `HH:MM` valides.
    let at = |t: &str| {
        let (h, m) = t.split_once(':').unwrap_or(("0", "0"));
        h.parse::<u32>().unwrap_or(0) * 60 + m.parse::<u32>().unwrap_or(0)
    };
    let today = week[day].iter().find(|(start, end)| {
        let (s, e) = (at(start), at(end));
        if e > s {
            (s..e).contains(&minutes)
        } else {
            minutes >= s
        }
    });
    let yesterday = || {
        week[(day + 6) % 7].iter().find(|(start, end)| {
            let (s, e) = (at(start), at(end));
            e < s && minutes < e
        })
    };
    today.or_else(yesterday).map(|(_, end)| end.clone())
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
/// qu'on appelle, puis l'hôpital, où l'on va. Celles qui ont un numéro sont des rangées à part.
///
/// « Hôpital : CHU d'Antibes (3.1 km) » quand la place et le logement sont placés.
fn useful_line(
    config: &ModuleConfig,
    away: &dyn Fn(Option<f64>, Option<f64>) -> Option<String>,
) -> String {
    [
        (
            "guest.useful.pharmacy",
            config.pharmacy.trim(),
            &config.pharmacy_phone,
            away(config.pharmacy_lat, config.pharmacy_lng),
        ),
        (
            "guest.useful.hospital",
            config.hospital.trim(),
            &config.hospital_phone,
            away(config.hospital_lat, config.hospital_lng),
        ),
    ]
    .into_iter()
    .filter(|(_, value, phone, _)| !value.is_empty() && phone.trim().is_empty())
    .map(|(key, value, _, distance)| match distance {
        Some(distance) => (key, format!("{value} ({distance})")),
        None => (key, value.to_string()),
    })
    .filter_map(|(key, value)| t!(key, value = value).ok())
    .collect::<Vec<_>>()
    .join(" · ")
}

/// La distance à vol d'oiseau, en mètres sous le kilomètre et en kilomètres au-delà — copiée de
/// `waste-recycling`, pour que deux modules l'écrivent pareil dans le même livret.
///
/// ponytail: à vol d'oiseau, pas par la route — un itinéraire coûterait un appel réseau par place.
fn distance(from: (f64, f64), to: (f64, f64)) -> String {
    let metres = haversine_metres(from, to).round() as i64;
    if metres < 1000 {
        // Arrondi à 50 m : annoncer « 643 m » sur une ligne droite serait une fausse précision.
        let rounded = (((metres + 25) / 50) * 50).max(50);
        t!("guest.distance.metres", value = rounded).unwrap_or_else(|_| format!("{rounded} m"))
    } else {
        let km = (metres as f64) / 1000.0;
        t!("guest.distance.km", value = format!("{km:.1}"))
            .unwrap_or_else(|_| format!("{km:.1} km"))
    }
}

/// Haversine, rayon moyen de la Terre.
fn haversine_metres((lat1, lng1): (f64, f64), (lat2, lng2): (f64, f64)) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let a = ((phi2 - phi1) / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * ((lng2 - lng1).to_radians() / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pharmacy_is_open_until_its_closing_time() {
        let week = hours::parse_week("mon=09:00-12:00,14:00-19:30;sat=22:00-02:00");
        assert_eq!(open_until(&week, 0, 10 * 60), Some("12:00".into()));
        assert_eq!(open_until(&week, 0, 13 * 60), None);
        assert_eq!(open_until(&week, 0, 19 * 60), Some("19:30".into()));
        assert_eq!(open_until(&week, 1, 10 * 60), None, "mardi : fermé");
        // Samedi 22:00 – 02:00 : encore ouverte dimanche à 01:00.
        assert_eq!(open_until(&week, 5, 23 * 60), Some("02:00".into()));
        assert_eq!(open_until(&week, 6, 60), Some("02:00".into()));
        assert_eq!(open_until(&week, 6, 3 * 60), None);
    }
}
