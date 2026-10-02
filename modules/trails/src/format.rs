//! Les mesures d'un itinéraire, mises en mots.
//!
//! Chaque forme passe par une clé de traduction : « 2 h 30 » devient « 2 Std. 30 » et « 2 時間30 »
//! sans que ce fichier le sache. Les nombres, eux, s'écrivent avec un point décimal — comme la
//! distance de marche de `waste-recycling`, pour que deux modules n'annoncent pas « 2,6 km » et
//! « 2.6 km » dans le même livret.

use portaki_sdk::host::i18n::{translate, Vars};

/// La durée de marche, sans les pauses : « 45 min », « 1 h », « 2 h 30 ».
pub fn duration(minutes: u32) -> String {
    let (hours, rest) = (minutes / 60, minutes % 60);
    match (hours, rest) {
        (0, _) => text(
            "guest.duration.minutes",
            &[("minutes", &minutes.to_string())],
        ),
        (_, 0) => text("guest.duration.hours", &[("hours", &hours.to_string())]),
        _ => text(
            "guest.duration.hoursMinutes",
            &[
                ("hours", &hours.to_string()),
                ("minutes", &rest.to_string()),
            ],
        ),
    }
}

/// La distance : « 8 km », « 2.6 km ». Le zéro décimal tombe — « 8.0 km » se lit comme une mesure
/// au décimètre.
pub fn distance(km: f64) -> String {
    text("guest.distance.km", &[("value", &km_value(km))])
}

/// Le nombre d'une distance en kilomètres, au dixième, sans décimale inutile.
fn km_value(km: f64) -> String {
    let rounded = (km * 10.0).round() / 10.0;
    if rounded.fract().abs() < f64::EPSILON {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

/// Le dénivelé positif : « 600 m ».
pub fn elevation(metres: u32) -> String {
    text("guest.elevation.metres", &[("value", &metres.to_string())])
}

/// La distance à pied jusqu'au départ, arrondie à 50 m : annoncer « 412 m » sur une ligne droite
/// serait une fausse précision.
pub fn walking(metres: f64) -> String {
    let metres = metres.round() as i64;
    if metres < 1000 {
        let rounded = (((metres + 25) / 50) * 50).max(50);
        text("guest.distance.metres", &[("value", &rounded.to_string())])
    } else {
        distance(metres as f64 / 1000.0)
    }
}

/// Le libellé d'un niveau, forme comprise : « ▲▲ Moyen ».
pub fn level(key: &str) -> String {
    text(&format!("guest.level.{key}"), &[])
}

/// Le libellé d'une forme : « Boucle », « Aller-retour ».
pub fn shape(key: &str) -> String {
    text(&format!("guest.shape.{key}"), &[])
}

/// Un texte traduit, ou sa clé telle quelle — ce que le livret sait encore résoudre lui-même.
///
/// `t!` veut des noms de variables écrits en dur et une clé littérale ; ici la clé se compose
/// (`guest.level.{key}`) et les variables changent avec la forme. D'où l'appel direct.
fn text(key: &str, vars: &[(&str, &str)]) -> String {
    let mut map = Vars::new();
    for (name, value) in vars {
        map.set(*name, value);
    }
    translate(key, &map).unwrap_or_else(|_| format!("i18n:{key}"))
}

/// Haversine, rayon moyen de la Terre.
pub fn haversine_metres((lat1, lng1): (f64, f64), (lat2, lng2): (f64, f64)) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let delta_phi = phi2 - phi1;
    let delta_lambda = (lng2 - lng1).to_radians();
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sans traduction chargée, chaque forme rend sa clé : ce qui est vérifié ici, c'est la clé
    /// choisie, donc la forme. Le texte, lui, est vérifié par la batterie i18n.
    #[test]
    fn a_duration_picks_its_shape_from_the_minutes() {
        assert_eq!(duration(45), "i18n:guest.duration.minutes");
        assert_eq!(duration(60), "i18n:guest.duration.hours");
        assert_eq!(duration(150), "i18n:guest.duration.hoursMinutes");
        // Zéro minute n'est pas une heure ronde : « 0 min », pas « 0 h ».
        assert_eq!(duration(0), "i18n:guest.duration.minutes");
    }

    #[test]
    fn a_round_distance_drops_its_decimal() {
        assert_eq!(km_value(8.0), "8");
        assert_eq!(km_value(8.04), "8");
        assert_eq!(km_value(2.64), "2.6");
        assert_eq!(km_value(0.95), "1");
        assert_eq!(km_value(11.0), "11");
    }

    #[test]
    fn a_walk_under_a_kilometre_rounds_to_fifty_metres() {
        assert_eq!(walking(412.0), "i18n:guest.distance.metres");
        assert_eq!(walking(1200.0), "i18n:guest.distance.km");
    }

    #[test]
    fn haversine_measures_a_known_pair() {
        // Cap d'Antibes → Biot, environ 8,5 km à vol d'oiseau.
        let metres = haversine_metres((43.5600, 7.1300), (43.6280, 7.0980));
        assert!((7_000.0..10_000.0).contains(&metres), "{metres}");
        assert!(haversine_metres((43.56, 7.13), (43.56, 7.13)) < 1.0);
    }
}
