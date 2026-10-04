//! Ce que le livret a besoin de savoir : le tableau, les destinations qu'il contient, et le jour.

use chrono::Datelike;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::sncf::{self, BoardError, Station, Stop, Way};

/// La valeur que le combobox renvoie pour « toutes les gares ».
pub const ALL_STATIONS: &str = "all";

/// Destinations proposées au voyageur. Au-delà, la liste se lit moins vite qu'elle ne sert.
const MAX_DESTINATIONS: usize = 12;

/// Le tableau prêt à dessiner.
pub struct BoardView {
    pub station: Station,
    /// Les lignes du sens demandé, filtrées par la destination choisie.
    pub stops: Vec<Stop>,
    /// Les destinations distinctes du tableau **entier**, pour que le filtre ne se vide pas
    /// lui-même : filtrer sur Cannes ne doit pas faire disparaître Nice de la liste.
    pub destinations: Vec<String>,
    /// La date locale de la gare, `AAAA-MM-JJ`. `None` sans horloge.
    pub today: Option<String>,
    /// La phrase que l'hôte a écrite sous le tableau.
    pub note: String,
}

impl BoardView {
    /// Ce départ est-il pour demain ou plus tard ? Sans horloge, on ne dit rien.
    pub fn is_later_day(&self, stop: &Stop) -> bool {
        self.today
            .as_deref()
            .is_some_and(|today| stop.date.as_str() > today)
    }
}

/// Le tableau, ou pourquoi il n'y en a pas.
pub fn load(
    ctx: &GuestContext,
    way: Way,
    destination: &str,
) -> std::result::Result<BoardView, BoardError> {
    let config = ModuleConfig::load(ctx).map_err(|_| BoardError::NoStation)?;
    let station_name = config.station_name().ok_or(BoardError::NoStation)?;
    let (station, all) = sncf::board(station_name, way)?;

    let destinations = distinct_destinations(&all);
    let stops = match destination {
        ALL_STATIONS | "" => all,
        wanted => all
            .into_iter()
            .filter(|stop| stop.direction.eq_ignore_ascii_case(wanted))
            .collect(),
    };

    Ok(BoardView {
        station,
        stops,
        destinations,
        today: today_at_property(ctx),
        note: config.note.get(&ctx.locale).trim().to_string(),
    })
}

/// Les destinations du tableau, dans l'ordre où elles y apparaissent — c'est celui de la
/// prochaine fois qu'on peut y aller, ce qui vaut mieux que l'alphabet.
fn distinct_destinations(stops: &[Stop]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for stop in stops {
        if !seen
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&stop.direction))
        {
            seen.push(stop.direction.clone());
        }
        if seen.len() == MAX_DESTINATIONS {
            break;
        }
    }
    seen
}

/// Le jour qu'il est dans le fuseau du logement.
///
/// Dans son fuseau, pas dans celui du téléphone : un voyageur qui regarde ses trains depuis Tokyo
/// ne doit pas lire « demain » devant le prochain train de ce matin.
fn today_at_property(ctx: &GuestContext) -> Option<String> {
    let now = time::now().ok()?;
    let local = match time::PropertyTz::parse(&ctx.timezone) {
        Some(tz) => tz.to_local(now).date_naive(),
        None => now.date_naive(),
    };
    Some(format!(
        "{:04}-{:02}-{:02}",
        local.year(),
        local.month(),
        local.day()
    ))
}

/// La ligne demandée par l'adresse, parmi celles du tableau.
pub fn stop_by_route_id(view: &BoardView, wanted: &str) -> Option<Stop> {
    view.stops
        .iter()
        .find(|stop| stop.route_id() == wanted)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(direction: &str, time: &str, date: &str) -> Stop {
        Stop {
            time: time.to_string(),
            date: date.to_string(),
            direction: direction.to_string(),
            headsign: Some(format!("TER {time}").replace(':', "")),
            mode: Some("TER".to_string()),
            network: None,
            realtime: true,
        }
    }

    #[test]
    fn destinations_keep_the_order_of_the_board_without_repeating() {
        let stops = [
            stop("Nice-Ville", "08:12", "2026-10-04"),
            stop("Cannes", "08:20", "2026-10-04"),
            stop("nice-ville", "08:42", "2026-10-04"),
            stop("Nice-Ville", "09:05", "2026-10-04"),
        ];
        assert_eq!(
            distinct_destinations(&stops),
            ["Nice-Ville", "Cannes"],
            "la casse ne crée pas une deuxième destination"
        );
    }

    #[test]
    fn a_departure_after_midnight_is_another_day() {
        let view = BoardView {
            station: Station {
                id: "stop_area:SNCF:87756056".to_string(),
                label: "Antibes".to_string(),
            },
            stops: Vec::new(),
            destinations: Vec::new(),
            today: Some("2026-10-04".to_string()),
            note: String::new(),
        };
        assert!(!view.is_later_day(&stop("Nice-Ville", "23:48", "2026-10-04")));
        // La nuit sans train : le premier résultat des 24 h suivantes est celui du matin (§2.17).
        assert!(view.is_later_day(&stop("Nice-Ville", "05:21", "2026-10-05")));

        // Sans horloge, aucun jour n'est annoncé plutôt qu'un jour deviné.
        let blind = BoardView {
            today: None,
            ..view
        };
        assert!(!blind.is_later_day(&stop("Nice-Ville", "05:21", "2026-10-05")));
    }

    #[test]
    fn a_route_id_points_back_at_its_departure() {
        let one = stop("Nice-Ville", "08:42", "2026-10-04");
        let id = one.route_id();
        assert_eq!(id, "20261004-0842-ter-0842");
        let view = BoardView {
            station: Station {
                id: "x".to_string(),
                label: "Antibes".to_string(),
            },
            stops: vec![one.clone(), stop("Cannes", "08:20", "2026-10-04")],
            destinations: Vec::new(),
            today: None,
            note: String::new(),
        };
        assert_eq!(stop_by_route_id(&view, &id), Some(one));
        assert_eq!(stop_by_route_id(&view, "20261004-2359-ter-2359"), None);
        assert_eq!(stop_by_route_id(&view, ""), None);
    }
}
