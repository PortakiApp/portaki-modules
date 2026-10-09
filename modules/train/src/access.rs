//! Le temps d'accès à la gare (spec Trains §2.1, « Temps d'accès », calculé) : « Gare à 12 min à
//! pied ».
//!
//! ponytail: vol d'oiseau × 1,3 pour les détours, 4,8 km/h à pied et 35 km/h en voiture — une
//! estimation, pas un itinéraire ; un calcul de trajet (Mapbox Directions) si les hôtes la
//! trouvent trop fausse.

use portaki_sdk::sdui::common::GeoPoint;

use crate::sncf::Station;

const DETOUR: f64 = 1.3;
const WALK_KMH: f64 = 4.8;
const DRIVE_KMH: f64 = 35.0;
/// Au-delà, la marche ne se propose plus : on dit la voiture.
pub const WALK_LIMIT_MIN: u32 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    pub walk_min: u32,
    pub drive_min: u32,
}

/// Le temps d'accès, quand la gare et le logement sont placés.
pub fn access(station: &Station, home: Option<GeoPoint>) -> Option<Access> {
    let (lat, lng, home) = (station.lat?, station.lng?, home?);
    let km = haversine_km(lat, lng, home.lat, home.lng) * DETOUR;
    let minutes = |kmh: f64| ((km / kmh) * 60.0).ceil().max(1.0) as u32;
    Some(Access {
        walk_min: minutes(WALK_KMH),
        drive_min: minutes(DRIVE_KMH),
    })
}

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = (lat2 - lat1).to_radians();
    let dl = (lng2 - lng1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    6371.0 * 2.0 * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn antibes() -> Station {
        Station {
            id: "x".into(),
            label: "Antibes".into(),
            lat: Some(43.5859),
            lng: Some(7.1194),
        }
    }

    #[test]
    fn about_a_kilometre_is_a_quarter_hour_walk() {
        let home = GeoPoint {
            lat: 43.5800,
            lng: 7.1250,
        };
        let access = access(&antibes(), Some(home)).unwrap();
        assert!((12..=20).contains(&access.walk_min), "{access:?}");
        assert!(access.drive_min <= 3, "{access:?}");
    }

    #[test]
    fn nothing_without_both_positions() {
        assert_eq!(access(&antibes(), None), None);
        let unplaced = Station {
            lat: None,
            ..antibes()
        };
        assert_eq!(
            access(&unplaced, Some(GeoPoint { lat: 0.0, lng: 0.0 })),
            None
        );
    }
}
