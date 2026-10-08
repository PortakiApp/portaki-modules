//! Les créneaux d'arrivée, bornés à l'heure d'entrée du logement (§2.20).
//!
//! Un sélecteur d'heure libre laissait répondre « 11 h 15 » à un logement qui ouvre à 16 h : le
//! voyageur annonçait une heure où personne ne peut entrer, et l'hôte lisait une promesse
//! intenable. La spec demande des segments, et leurs bornes sortent de l'heure d'arrivée du
//! logement — pas d'une liste écrite dans le module.

use chrono::{DateTime, Timelike, Utc};
use portaki_sdk::host::time::PropertyTz;
use portaki_sdk::t;

/// Un créneau proposé au voyageur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// Ce que le formulaire envoie : le début du créneau, en `HH:MM`.
    ///
    /// Une heure, pas une plage : la plateforme la recopie sur le séjour
    /// (`StayContext::arrival_time_estimated`, un `NaiveTime`), et `access-guide` la lit de là.
    /// C'est un plancher — « pas avant » — et les libellés le disent.
    pub value: String,
    pub label: String,
}

/// Les trois créneaux d'un logement qui ouvre à `hour`.
///
/// `[h, h+1)`, `[h+1, h+3)`, puis « après h+3 » : les bornes de la planche quand l'entrée est à
/// 16 h, et elles suivent l'hôte qui ouvre plus tôt ou plus tard. Modulo 24 pour un logement qui
/// ouvre le soir — une entrée à 22 h donne « 23 h – 1 h », pas « 25 h ».
pub fn slots(hour: u32) -> Vec<Slot> {
    let at = |offset: u32| (hour + offset) % 24;
    vec![
        Slot {
            value: time_value(at(0)),
            label: range_label(at(0), at(1)),
        },
        Slot {
            value: time_value(at(1)),
            label: range_label(at(1), at(3)),
        },
        Slot {
            value: time_value(at(3)),
            label: late_label(at(3)),
        },
    ]
}

/// L'heure d'entrée du logement, dans son fuseau.
///
/// Dans son fuseau, pas dans celui du téléphone : un voyageur qui remplit le formulaire depuis
/// Tokyo doit lire les créneaux de la gare d'arrivée, pas les siens. `None` sans séjour ni date
/// d'arrivée — il n'y a alors aucune borne à proposer.
pub fn checkin_hour(checkin_at: Option<DateTime<Utc>>, timezone: &str) -> Option<u32> {
    let checkin = checkin_at?;
    Some(match PropertyTz::parse(timezone) {
        Some(tz) => tz.to_local(checkin).hour(),
        None => checkin.hour(),
    })
}

/// Le créneau qui contient une réponse déjà donnée, pour la retrouver cochée.
///
/// Une réponse d'avant les créneaux est une heure libre (« 17:30 ») : elle tombe dans son
/// créneau plutôt que de disparaître. Et une heure *antérieure* à l'entrée — ce que l'ancien
/// sélecteur laissait saisir — revient au premier créneau : c'est le plus tôt qui existe.
pub fn slot_of(answer: &str, hour: u32) -> Option<usize> {
    let answered = parse_hour(answer)?;
    let offset = (answered + 24 - hour) % 24;
    Some(match offset {
        0 => 0,
        1 | 2 => 1,
        // Au-delà d'une demi-journée, l'heure est avant l'entrée, pas treize heures après.
        13..=23 => 0,
        _ => 2,
    })
}

fn parse_hour(raw: &str) -> Option<u32> {
    let (hour, _) = raw.trim().split_once(':')?;
    hour.parse::<u32>().ok().filter(|hour| *hour < 24)
}

fn time_value(hour: u32) -> String {
    format!("{hour:02}:00")
}

fn range_label(from: u32, to: u32) -> String {
    t!("form.arrival.slot.range", from = from, to = to)
        .unwrap_or_else(|_| "i18n:form.arrival.slot.range".into())
}

fn late_label(from: u32) -> String {
    t!("form.arrival.slot.late", from = from)
        .unwrap_or_else(|_| "i18n:form.arrival.slot.late".into())
}

/// « dès 17 h » — l'heure annoncée est un plancher, pas un rendez-vous.
pub fn floor_label(answer: &str) -> String {
    t!("form.arrival.from", time = answer).unwrap_or_else(|_| "i18n:form.arrival.from".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slots_follow_the_check_in_hour() {
        let values: Vec<String> = slots(16).into_iter().map(|slot| slot.value).collect();
        assert_eq!(values, ["16:00", "17:00", "19:00"]);
        // Un logement qui ouvre à 14 h ne propose pas les créneaux d'un autre.
        let early: Vec<String> = slots(14).into_iter().map(|slot| slot.value).collect();
        assert_eq!(early, ["14:00", "15:00", "17:00"]);
        // Et une entrée du soir ne donne pas « 25 h ».
        let night: Vec<String> = slots(22).into_iter().map(|slot| slot.value).collect();
        assert_eq!(night, ["22:00", "23:00", "01:00"]);
    }

    #[test]
    fn an_answer_finds_its_slot() {
        assert_eq!(slot_of("16:00", 16), Some(0));
        assert_eq!(slot_of("16:45", 16), Some(0));
        // Une heure libre d'avant les créneaux tombe dans le sien.
        assert_eq!(slot_of("17:30", 16), Some(1));
        assert_eq!(slot_of("18:00", 16), Some(1));
        assert_eq!(slot_of("19:00", 16), Some(2));
        assert_eq!(slot_of("23:15", 16), Some(2));
        // Avant l'heure d'entrée : le premier créneau, pas le dernier.
        assert_eq!(slot_of("11:00", 16), Some(0));
        assert_eq!(slot_of("", 16), None);
        assert_eq!(slot_of("midi", 16), None);
        assert_eq!(slot_of("99:00", 16), None);
    }

    #[test]
    fn without_a_check_in_there_is_no_hour_to_bound_on() {
        assert_eq!(checkin_hour(None, "Europe/Paris"), None);
        let checkin = "2026-10-04T14:00:00Z"
            .parse::<DateTime<Utc>>()
            .expect("date");
        // 14 h UTC, 16 h à Paris : c'est l'heure du logement qui borne, pas UTC.
        assert_eq!(checkin_hour(Some(checkin), "Europe/Paris"), Some(16));
        assert_eq!(checkin_hour(Some(checkin), ""), Some(14));
    }
}
