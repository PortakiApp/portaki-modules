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

/// Des créneaux réguliers (spec Pré-arrivée §2.1) : de l'heure d'entrée jusqu'à `until`, tous
/// les `step` minutes. `until` avant l'entrée passe minuit (« 16:00 → 01:00 »). Toujours au moins
/// un créneau, celui de l'entrée.
pub fn stepped(hour: u32, until_minutes: u32, step: u32) -> Vec<Slot> {
    let start = hour * 60;
    let span = (until_minutes + 24 * 60 - start) % (24 * 60);
    let step = step.max(15);
    (0..=span / step)
        .map(|n| {
            let at = (start + n * step) % (24 * 60);
            let value = format!("{:02}:{:02}", at / 60, at % 60);
            Slot {
                label: value.clone(),
                value,
            }
        })
        .collect()
}

/// `HH:MM` en minutes depuis minuit.
pub fn minutes_of(raw: &str) -> Option<u32> {
    let (hour, minute) = raw.trim().split_once(':')?;
    let (hour, minute): (u32, u32) = (hour.parse().ok()?, minute.parse().ok()?);
    (hour < 24 && minute < 60).then_some(hour * 60 + minute)
}

/// Le créneau qui contient une réponse déjà donnée, quels que soient les créneaux : le dernier
/// qui commence avant elle. Une heure *antérieure* à l'entrée — plus d'une demi-journée « après »
/// en tournant l'horloge — revient au premier : c'est le plus tôt qui existe.
pub fn slot_index(answer: &str, slots: &[Slot]) -> Option<usize> {
    let answered = minutes_of(answer)?;
    let first = minutes_of(&slots.first()?.value)?;
    let offset = |at: u32| (at + 24 * 60 - first) % (24 * 60);
    let answered = offset(answered);
    if answered >= 12 * 60 {
        return Some(0);
    }
    slots
        .iter()
        .rposition(|slot| minutes_of(&slot.value).is_some_and(|at| offset(at) <= answered))
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
        let ranges = slots(16);
        assert_eq!(slot_index("16:00", &ranges), Some(0));
        assert_eq!(slot_index("16:45", &ranges), Some(0));
        // Une heure libre d'avant les créneaux tombe dans le sien.
        assert_eq!(slot_index("17:30", &ranges), Some(1));
        assert_eq!(slot_index("18:00", &ranges), Some(1));
        assert_eq!(slot_index("19:00", &ranges), Some(2));
        assert_eq!(slot_index("23:15", &ranges), Some(2));
        // Avant l'heure d'entrée : le premier créneau, pas le dernier.
        assert_eq!(slot_index("11:00", &ranges), Some(0));
        assert_eq!(slot_index("", &ranges), None);
        assert_eq!(slot_index("midi", &ranges), None);
        assert_eq!(slot_index("99:00", &ranges), None);
    }

    #[test]
    fn stepped_slots_run_from_check_in_to_the_end() {
        let values: Vec<String> = stepped(16, 23 * 60, 60)
            .into_iter()
            .map(|s| s.value)
            .collect();
        assert_eq!(values.len(), 8);
        assert_eq!(values.first().map(String::as_str), Some("16:00"));
        assert_eq!(values.last().map(String::as_str), Some("23:00"));
        let half: Vec<String> = stepped(22, 60, 30).into_iter().map(|s| s.value).collect();
        assert_eq!(
            half,
            ["22:00", "22:30", "23:00", "23:30", "00:00", "00:30", "01:00"]
        );
        assert_eq!(slot_index("22:45", &stepped(22, 60, 30)), Some(1));
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
