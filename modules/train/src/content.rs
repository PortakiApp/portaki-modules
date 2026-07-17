//! Static v0.1 content — nearest station + mock TER schedule.
//!
//! No host configuration and no storage yet: station info and destinations
//! are hardcoded here. A future pass will read this from module config /
//! a Navitia connector instead.

/// Icon id used on the home card and module manifest.
pub const MODULE_ICON: &str = "train";

/// Nearest station label (fallback until host config exists).
pub const DEFAULT_STATION_LABEL: &str = "Gare d'Antibes";

/// Nearest station distance, already localized per locale.
pub fn default_station_distance(locale: &str) -> &'static str {
    if locale.to_ascii_lowercase().starts_with("en") {
        "1.4 mi"
    } else {
        "2,3 km"
    }
}

/// Station caption line, e.g. "Gare d'Antibes · 2,3 km".
pub fn station_caption(locale: &str) -> String {
    format!(
        "{} · {}",
        DEFAULT_STATION_LABEL,
        default_station_distance(locale)
    )
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
        },
        BoardEntry {
            time: "08:42",
            destination: "Nice-Ville",
            platform: "quai 1",
        },
        BoardEntry {
            time: "09:05",
            destination: "Cannes",
            platform: "quai 3",
        },
        BoardEntry {
            time: "09:24",
            destination: "Monaco",
            platform: "quai 2",
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
