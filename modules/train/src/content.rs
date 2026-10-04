//! Static v0.1 content — nearest station + mock TER schedule.
//!
//! No host configuration and no storage yet: station info and destinations
//! are hardcoded here. A future pass will read this from module config /
//! a Navitia connector instead.

use portaki_sdk::prelude::*;

/// Icon id used on the home card and module manifest.
pub const MODULE_ICON: portaki_sdk::vocab::IconName = portaki_sdk::vocab::IconName::Train;

/// Nearest station label (fallback until host config exists).
pub const DEFAULT_STATION_LABEL: &str = "Gare d'Antibes";

/// Nearest station distance via host i18n (`station.distance`).
pub fn default_station_distance() -> String {
    t!("station.distance").unwrap_or_else(|_| "2,3 km".into())
}

/// Station caption line, e.g. "Gare d'Antibes · 2,3 km".
pub fn station_caption() -> String {
    format!("{} · {}", DEFAULT_STATION_LABEL, default_station_distance())
}

/// Selectable destinations, in display order.
pub const DESTINATIONS: [&str; 4] = ["Nice-Ville", "Cannes", "Monaco", "Grasse"];

/// Default destination when none is selected via `ctx.input.dest`.
pub const DEFAULT_DESTINATION: &str = "Nice-Ville";

/// One scheduled departure: time + platform + note.
#[derive(Debug, Clone, Copy)]
pub struct Departure {
    pub time: &'static str,
    pub platform: &'static str,
    pub note: &'static str,
}

/// One mixed-destination row for the home card glance board.
#[derive(Debug, Clone, Copy)]
pub struct BoardEntry {
    pub time: &'static str,
    pub destination: &'static str,
    pub platform: &'static str,
    /// « direct », « 1 arrêt », « chgt Nice » — ce que la ligne porte en pastille de fin.
    pub note: &'static str,
}

/// Normalizes an incoming `dest` param against [`DESTINATIONS`], falling back to the default.
pub fn normalize_destination(dest: Option<&str>) -> &'static str {
    match dest {
        Some(value) => DESTINATIONS
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(value))
            .copied()
            .unwrap_or(DEFAULT_DESTINATION),
        None => DEFAULT_DESTINATION,
    }
}

/// Home card glance — next 4 departures across all destinations (mock TER board).
pub fn home_board() -> [BoardEntry; 4] {
    [
        BoardEntry {
            time: "08:12",
            destination: "Nice-Ville",
            platform: "quai 2",
            note: "direct",
        },
        BoardEntry {
            time: "08:42",
            destination: "Nice-Ville",
            platform: "quai 1",
            note: "direct",
        },
        BoardEntry {
            time: "09:05",
            destination: "Cannes",
            platform: "quai 3",
            note: "1 arrêt",
        },
        BoardEntry {
            time: "09:24",
            destination: "Monaco",
            platform: "quai 2",
            note: "chgt Nice",
        },
    ]
}

/// Next departures for a given destination (mock TER SUD PACA schedule).
pub fn schedule_for(destination: &str) -> Vec<Departure> {
    let raw: &[(&str, &str, &str)] = match destination {
        "Cannes" => &[
            ("08:20", "quai 3", "direct"),
            ("08:55", "quai 3", "direct"),
            ("09:31", "quai 3", "1 arrêt"),
        ],
        "Monaco" => &[
            ("08:12", "quai 2", "chgt Nice"),
            ("09:24", "quai 2", "chgt Nice"),
        ],
        "Grasse" => &[
            ("08:34", "quai 4", "1 arrêt"),
            ("09:40", "quai 4", "direct"),
        ],
        _ => &[
            ("08:12", "quai 2", "direct"),
            ("08:42", "quai 1", "direct"),
            ("09:05", "quai 2", "1 arrêt"),
            ("09:38", "quai 1", "direct"),
        ],
    };
    raw.iter()
        .map(|(time, platform, note)| Departure {
            time,
            platform,
            note,
        })
        .collect()
}

/// Les deux sens que le livret propose : depuis la gare du logement, ou vers elle.
///
/// ponytail: le sens « vers » rend le même tableau que « depuis ». Les horaires de ce module sont
/// encore écrits ici (§0.1) ; un vrai tableau inverse demande la source d'horaires, pas une
/// deuxième liste inventée. Le sens est dans l'interface pour que le branchement n'ait plus qu'à
/// remplir les deux.
pub const DIRECTIONS: [&str; 2] = ["from", "to"];

/// Le sens par défaut : on part du logement plus souvent qu'on y revient en train.
pub const DEFAULT_DIRECTION: &str = "from";

/// Ramène un `dir` reçu dans [`DIRECTIONS`], le défaut sinon.
pub fn normalize_direction(dir: Option<&str>) -> &'static str {
    match dir {
        Some(value) => DIRECTIONS
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(value))
            .copied()
            .unwrap_or(DEFAULT_DIRECTION),
        None => DEFAULT_DIRECTION,
    }
}

/// L'identifiant de route d'un départ : sa destination et son heure.
///
/// Ni un rang ni un index : une liste d'horaires se décale dès qu'un train part, et une fiche
/// ouverte sur « le troisième » montrerait un autre train une minute plus tard.
pub fn departure_id(destination: &str, time: &str) -> String {
    format!("{}-{}", slug(destination), time.replace(':', ""))
}

/// Le départ que cet identifiant désigne, parmi ceux de la destination.
pub fn departure_by_id(id: &str) -> Option<(&'static str, Departure)> {
    DESTINATIONS.iter().find_map(|destination| {
        schedule_for(destination)
            .into_iter()
            .find(|departure| departure_id(destination, departure.time) == id)
            .map(|departure| (*destination, departure))
    })
}

/// Minuscules, et tout ce qui n'est pas une lettre ou un chiffre devient un tiret.
fn slug(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identifier_points_back_at_its_departure() {
        let id = departure_id("Nice-Ville", "08:42");
        assert_eq!(id, "nice-ville-0842");
        let (destination, departure) = departure_by_id(&id).expect("le départ");
        assert_eq!(destination, "Nice-Ville");
        assert_eq!(departure.time, "08:42");
    }

    #[test]
    fn an_unknown_identifier_points_at_nothing() {
        assert!(departure_by_id("nice-ville-2359").is_none());
        assert!(departure_by_id("").is_none());
    }

    #[test]
    fn a_direction_falls_back_rather_than_inventing_one() {
        assert_eq!(normalize_direction(Some("to")), "to");
        assert_eq!(normalize_direction(Some("TO")), "to");
        assert_eq!(normalize_direction(Some("sideways")), DEFAULT_DIRECTION);
        assert_eq!(normalize_direction(None), DEFAULT_DIRECTION);
    }
}
