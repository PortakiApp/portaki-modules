//! Les jours de collecte, et lequel vient (§2.7).
//!
//! Le module portait une phrase — « Mardi et vendredi matin » — qui se lit très bien et ne calcule
//! rien. Le §2.7 veut un bandeau « Prochaine collecte {jour} », ce qui demande de savoir quels
//! jours, et quel jour il est.
//!
//! Des jours structurés viennent donc **s'ajouter** à la phrase, sans la remplacer. Un hôte qui n'en
//! coche aucun garde exactement le bandeau qu'il avait ; un hôte qui les coche gagne le jour
//! calculé. Rien ici ne tente de lire « Mardi et vendredi matin » : deviner les jours d'une phrase,
//! c'est risquer de sortir les poubelles le mauvais soir.
//!
//! Le départ compte aussi. Le §2.7 veut deux cas dits : la collecte tombe le jour du départ, et le
//! départ est la veille d'une collecte — celui-là est le piège, puisque le voyageur ne sera plus là
//! pour sortir le bac et doit donc le faire avant de fermer la porte.
//!
//! Les jours de la semaine sont recopiés ici plutôt qu'empruntés au module des horaires : un module
//! n'importe pas un autre module (§4), même pour sept mots.

use chrono::{DateTime, Datelike, Days, NaiveDate, Utc, Weekday};
use portaki_sdk::host::time::PropertyTz;

/// Le jour de la semaine désigné par `mon` … `sun`, quelle que soit la casse.
pub fn parse_day(raw: &str) -> Option<Weekday> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "mon" => Some(Weekday::Mon),
        "tue" => Some(Weekday::Tue),
        "wed" => Some(Weekday::Wed),
        "thu" => Some(Weekday::Thu),
        "fri" => Some(Weekday::Fri),
        "sat" => Some(Weekday::Sat),
        "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

/// Ce que le départ du voyageur change à la collecte qui vient (§2.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Departure {
    /// La collecte tombe le jour du départ.
    SameDay,
    /// Le voyageur part la veille : le bac doit sortir avant qu'il ferme la porte, sinon personne
    /// ne le sortira.
    Eve,
}

/// La collecte qui vient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextCollection {
    pub day: Weekday,
    /// La date, et pas seulement le jour : c'est elle qu'on compare au départ.
    pub date: NaiveDate,
    /// C'est aujourd'hui — le bandeau ne dit pas la même chose (§2.7).
    pub today: bool,
    /// Ce que le départ en dit, quand la plateforme connaît le séjour.
    pub departure: Option<Departure>,
}

/// La prochaine collecte à partir de `now`, lue à l'heure du logement.
///
/// Aujourd'hui compte comme la prochaine, et n'est pas repoussé à la semaine suivante : un voyageur
/// qui lit le livret le matin d'une collecte a encore le temps de sortir le bac, et lui annoncer
/// mardi prochain serait faux le jour même.
pub fn next_collection(
    days: &[String],
    now: DateTime<Utc>,
    tz: Option<&PropertyTz>,
    checkout: Option<DateTime<Utc>>,
) -> Option<NextCollection> {
    let wanted: Vec<Weekday> = days.iter().filter_map(|day| parse_day(day)).collect();
    if wanted.is_empty() {
        return None;
    }

    let today = local_date(now, tz);
    // Sept jours à partir d'aujourd'hui : il y en a forcément un, puisque la liste n'est pas vide.
    let (date, ahead) = (0..7u64)
        .filter_map(|ahead| Some((today.checked_add_days(Days::new(ahead))?, ahead)))
        .find(|(date, _)| wanted.contains(&date.weekday()))?;

    let departure = checkout
        .map(|checkout| local_date(checkout, tz))
        .and_then(|leaving| {
            if leaving == date {
                Some(Departure::SameDay)
            } else if leaving.succ_opt() == Some(date) {
                Some(Departure::Eve)
            } else {
                None
            }
        });

    Some(NextCollection {
        day: date.weekday(),
        date,
        today: ahead == 0,
        departure,
    })
}

/// La date au fuseau du logement — celle que le voyageur lit sur son téléphone, pas celle du
/// serveur : à minuit passé, les deux ne nomment pas le même jour.
fn local_date(at: DateTime<Utc>, tz: Option<&PropertyTz>) -> NaiveDate {
    match tz {
        Some(tz) => tz.to_local(at).date_naive(),
        None => at.date_naive(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Mercredi 30 septembre 2026.
    fn wednesday(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap()
    }

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, day, 11, 0, 0).unwrap()
    }

    fn days(list: &[&str]) -> Vec<String> {
        list.iter().map(|d| d.to_string()).collect()
    }

    fn next(list: &[&str], now: DateTime<Utc>) -> Option<NextCollection> {
        next_collection(&days(list), now, None, None)
    }

    #[test]
    fn a_day_is_read_whatever_its_case() {
        assert_eq!(parse_day("mon"), Some(Weekday::Mon));
        assert_eq!(parse_day(" SUN "), Some(Weekday::Sun));
        // Une phrase reste une phrase : on ne devine pas les jours d'un texte libre.
        assert_eq!(parse_day("mardi"), None);
        assert_eq!(parse_day("Mardi et vendredi matin"), None);
        assert_eq!(parse_day(""), None);
    }

    #[test]
    fn no_day_no_computation() {
        assert_eq!(next(&[], wednesday(9)), None);
        assert_eq!(next(&["demain"], wednesday(9)), None);
    }

    #[test]
    fn today_counts_as_the_next_one() {
        // Un voyageur qui lit le livret le matin de la collecte a encore le temps de sortir le bac.
        let found = next(&["wed"], wednesday(7)).expect("une collecte");
        assert_eq!(found.day, Weekday::Wed);
        assert!(found.today);
        assert_eq!(found.date, NaiveDate::from_ymd_opt(2026, 9, 30).unwrap());
    }

    #[test]
    fn the_nearest_day_ahead_wins() {
        let found = next(&["tue", "fri"], wednesday(9)).expect("collecte");
        assert_eq!(found.day, Weekday::Fri);
        assert!(!found.today);
        assert_eq!(found.date, NaiveDate::from_ymd_opt(2026, 10, 2).unwrap());
    }

    #[test]
    fn a_day_already_past_this_week_comes_back_next_week() {
        let found = next(&["tue"], wednesday(9)).expect("collecte");
        assert_eq!(found.day, Weekday::Tue);
        assert!(!found.today);
        // Mardi prochain, pas celui d'hier.
        assert_eq!(found.date, NaiveDate::from_ymd_opt(2026, 10, 6).unwrap());
    }

    #[test]
    fn the_order_the_host_ticked_them_does_not_matter() {
        let one = next(&["fri", "tue"], wednesday(9)).expect("collecte");
        let other = next(&["tue", "fri"], wednesday(9)).expect("collecte");
        assert_eq!(one, other);
    }

    /// Le fuseau décide du jour, et à minuit passé les deux ne sont pas le même.
    #[test]
    fn the_day_is_the_one_the_property_is_living() {
        let tz = PropertyTz::parse("Europe/Paris").expect("un fuseau");
        // Mercredi 22 h UTC, et il est déjà jeudi minuit dans le logement.
        let found = next_collection(&days(&["thu"]), wednesday(22), Some(&tz), None).expect("c");
        assert_eq!(found.day, Weekday::Thu);
        assert!(found.today);
    }

    /// « Collecte le jour du départ → le bandeau le dit » (§2.7).
    #[test]
    fn a_collection_on_the_departure_day_is_said() {
        // Jeudi 1er octobre : collecte et départ le même jour.
        let found =
            next_collection(&days(&["thu"]), wednesday(9), None, Some(at(1))).expect("collecte");
        assert_eq!(found.departure, Some(Departure::SameDay));
    }

    /// « Départ la veille d'une collecte → consigne de sortie mise en avant » (§2.7).
    #[test]
    fn leaving_the_day_before_is_the_case_that_matters() {
        // Collecte vendredi 2, départ jeudi 1er : plus personne pour sortir le bac vendredi matin.
        let found =
            next_collection(&days(&["fri"]), wednesday(9), None, Some(at(1))).expect("collecte");
        assert_eq!(found.departure, Some(Departure::Eve));
    }

    /// Un départ plus lointain ne change rien : le bandeau ordinaire suffit.
    #[test]
    fn a_departure_further_out_says_nothing_special() {
        let found =
            next_collection(&days(&["fri"]), wednesday(9), None, Some(at(5))).expect("collecte");
        assert_eq!(found.departure, None);
        // Et un séjour dont la plateforme ne sait rien non plus.
        assert_eq!(next(&["fri"], wednesday(9)).unwrap().departure, None);
    }
}
