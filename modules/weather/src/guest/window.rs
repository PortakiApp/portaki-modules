//! La fenêtre de prévisions : celle du séjour, jamais un nombre de jours en dur (§0.7, §2.14).
//!
//! Cinq jours fixes ne disaient rien du séjour : une semaine à la mer s'arrêtait le mercredi, et
//! un voyageur qui prépare sa valise trois jours avant lisait surtout des journées qui ne le
//! concernent pas. « Rien avant, rien après. »

use chrono::{DateTime, Duration, NaiveDate, Utc};
use portaki_sdk::context::StayContext;
use portaki_sdk::host::time::PropertyTz;

/// Au-delà, aucun fournisseur ne sait rien de fiable, et §2.14 le dit déjà au voyageur : « au-delà
/// de 5 jours → mention de fiabilité ». Demander quinze jours coûterait un appel plus lourd pour
/// des chiffres inventés.
const MAX_DAYS: u8 = 10;

/// Le premier et le dernier jour à montrer, dans le fuseau du logement.
///
/// De la veille de l'arrivée au lendemain du départ (§0.7), sans jamais remonter avant
/// aujourd'hui : pendant le séjour, « jamais de jour passé » (§1.10), et une prévision d'hier
/// n'est plus une prévision.
pub fn window(
    stay: Option<&StayContext>,
    now: DateTime<Utc>,
    timezone: &str,
) -> Option<(NaiveDate, NaiveDate)> {
    let local = |instant: DateTime<Utc>| match PropertyTz::parse(timezone) {
        Some(tz) => tz.to_local(instant).date_naive(),
        None => instant.date_naive(),
    };
    let stay = stay?;
    let today = local(now);
    let from = local(stay.checkin_at?) - Duration::days(1);
    let to = local(stay.checkout_at?) + Duration::days(1);
    let from = from.max(today);
    (from <= to).then_some((from, to))
}

/// Le nombre de jours à demander au fournisseur pour couvrir cette fenêtre.
///
/// Sans séjour — un aperçu, une surface hors séjour — les cinq jours d'avant restent : c'est ce
/// qu'on peut dire de mieux sans savoir quand quelqu'un vient.
pub fn days_to_fetch(
    window: Option<(NaiveDate, NaiveDate)>,
    now: DateTime<Utc>,
    timezone: &str,
) -> u8 {
    let Some((_, to)) = window else {
        return 5;
    };
    let today = match PropertyTz::parse(timezone) {
        Some(tz) => tz.to_local(now).date_naive(),
        None => now.date_naive(),
    };
    let span = (to - today).num_days() + 1;
    span.clamp(1, MAX_DAYS as i64) as u8
}

/// Les jours de la fenêtre, dans l'ordre reçu.
///
/// Un jour dont la date ne se lit pas est gardé : il vient du fournisseur, et l'écarter sur un
/// format qu'on n'a pas su lire viderait la carte.
pub fn keep_window<T>(
    days: Vec<T>,
    window: Option<(NaiveDate, NaiveDate)>,
    date_of: impl Fn(&T) -> &str,
) -> Vec<T> {
    let Some((from, to)) = window else {
        return days;
    };
    days.into_iter()
        .filter(
            |day| match NaiveDate::parse_from_str(date_of(day).trim(), "%Y-%m-%d") {
                Ok(date) => date >= from && date <= to,
                Err(_) => true,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn stay(checkin: &str, checkout: &str) -> StayContext {
        StayContext {
            stay_id: Uuid::nil(),
            checkin_at: Some(at(checkin)),
            checkout_at: Some(at(checkout)),
            ..StayContext::default()
        }
    }

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("date")
            .with_timezone(&Utc)
    }

    fn day(date: &str) -> String {
        date.to_string()
    }

    /// Avant l'arrivée : de la veille au lendemain du départ, et rien d'autre.
    #[test]
    fn before_the_stay_the_window_opens_the_day_before_arrival() {
        let w = window(
            Some(&stay("2026-08-10T15:00:00Z", "2026-08-13T10:00:00Z")),
            at("2026-08-07T08:00:00Z"),
            "Europe/Paris",
        )
        .expect("fenêtre");
        assert_eq!(w.0.to_string(), "2026-08-09");
        assert_eq!(w.1.to_string(), "2026-08-14");

        let kept = keep_window(
            vec![
                day("2026-08-07"),
                day("2026-08-09"),
                day("2026-08-12"),
                day("2026-08-14"),
                day("2026-08-15"),
            ],
            Some(w),
            |d| d.as_str(),
        );
        assert_eq!(kept, ["2026-08-09", "2026-08-12", "2026-08-14"]);
    }

    /// Pendant le séjour : jamais un jour passé, même s'il est dans la fenêtre du §0.7.
    #[test]
    fn during_the_stay_yesterday_is_gone() {
        let w = window(
            Some(&stay("2026-08-10T15:00:00Z", "2026-08-20T10:00:00Z")),
            at("2026-08-15T08:00:00Z"),
            "Europe/Paris",
        )
        .expect("fenêtre");
        assert_eq!(w.0.to_string(), "2026-08-15");
        assert_eq!(w.1.to_string(), "2026-08-21");
    }

    /// Le nombre de jours demandé couvre la fenêtre, et s'arrête où les prévisions valent encore.
    #[test]
    fn the_fetch_covers_the_window_and_no_more() {
        let short = window(
            Some(&stay("2026-08-10T15:00:00Z", "2026-08-12T10:00:00Z")),
            at("2026-08-10T08:00:00Z"),
            "Europe/Paris",
        );
        assert_eq!(
            days_to_fetch(short, at("2026-08-10T08:00:00Z"), "Europe/Paris"),
            4
        );

        let long = window(
            Some(&stay("2026-08-10T15:00:00Z", "2026-09-10T10:00:00Z")),
            at("2026-08-10T08:00:00Z"),
            "Europe/Paris",
        );
        assert_eq!(
            days_to_fetch(long, at("2026-08-10T08:00:00Z"), "Europe/Paris"),
            MAX_DAYS
        );
    }

    /// Sans séjour, les cinq jours d'avant : c'est ce qu'on peut dire de mieux sans savoir
    /// quand quelqu'un vient.
    #[test]
    fn without_a_stay_nothing_is_dropped() {
        assert_eq!(
            window(None, at("2026-08-10T08:00:00Z"), "Europe/Paris"),
            None
        );
        assert_eq!(
            days_to_fetch(None, at("2026-08-10T08:00:00Z"), "Europe/Paris"),
            5
        );
        let days = vec![day("2026-01-01"), day("2027-01-01")];
        assert_eq!(keep_window(days.clone(), None, |d| d.as_str()), days);
    }
}
