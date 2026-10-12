//! Les tâches de `timelineTasks` (spec Pré-arrivée §1), calculées, jamais stockées :
//! - « Arrivée tardive · Marie · 23:30 » (§2.1) : une arrivée de la fenêtre annoncée à partir de
//!   « Arrivée tardive à partir de » ;
//! - « Pré-arrivée non remplie · Marie · arrivée demain » : une arrivée sans réponse dont la
//!   limite (`deadline`) est passée ou tombe dans les 24 h.

use chrono::{DateTime, Datelike, Duration, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{self, TimelineTask, TimelineTasks, TimelineTasksArgs};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n;
use crate::show_when::is_form_available;
use crate::slots::{checkin_hour, minutes_of};
use crate::storage;

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-08-17T00:00:00Z","to":"2026-08-24T00:00:00Z","stays":[{"id":"0b6f2c3e-1d4a-4e5b-8c7d-9e0f1a2b3c4d","checkIn":"2026-08-21T14:00:00Z","checkOut":"2026-08-24T08:00:00Z","guestName":"Marie","status":"UPCOMING"}]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let config = ModuleConfig::load(&ctx)?;
    let timezone = &ctx.property.timezone;
    let mut tasks = late_arrival_tasks(&config, &args, timezone, &ctx.property.name);
    tasks.extend(unfilled_form_tasks(&config, &args, time::now()?, timezone));
    Ok(TimelineTasks { tasks })
}

/// Une tâche par arrivée de la fenêtre encore à venir, au formulaire proposé mais sans réponse, et
/// dont la limite est passée ou tombe dans les 24 h. Elle disparaît à la réponse ou à l'arrivée.
pub fn unfilled_form_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    now: DateTime<Utc>,
    timezone: &str,
) -> Vec<TimelineTask> {
    args.stays
        .iter()
        .filter(|stay| (args.from..=args.to).contains(&stay.check_in) && now < stay.check_in)
        .filter(|stay| now >= config.deadline.at(stay.check_in, timezone) - Duration::hours(24))
        .filter(|stay| is_form_available(config.show_when, now, Some(stay.check_in)))
        // Une lecture en échec ne dit pas « non rempli » : pas de tâche plutôt qu'une fausse.
        .filter(|stay| matches!(storage::find_by_stay(stay.id), Ok(None)))
        .map(|stay| {
            timeline::task(
                format!("unfilled:{}", stay.id),
                stay.check_in,
                args.property_id,
                i18n::text("task.unfilled.title"),
                arrival_context(stay.guest_name.trim(), stay.check_in, now, timezone),
            )
            .stay(stay.id)
        })
        .collect()
}

/// « Marie · arrivée demain », « arrivée aujourd'hui », « arrivée le 24/08 » : les jours comptés
/// à l'heure du logement (UTC sans fuseau connu, comme [`crate::config::Deadline::at`]).
fn arrival_context(
    guest: &str,
    check_in: DateTime<Utc>,
    now: DateTime<Utc>,
    timezone: &str,
) -> I18nText {
    let tz = PropertyTz::parse(timezone);
    let day = |at: DateTime<Utc>| tz.map_or(at.date_naive(), |tz| tz.to_local(at).date_naive());
    let arrival = day(check_in);
    let when = match (arrival - day(now)).num_days() {
        0 => "today",
        1 => "tomorrow",
        _ => "date",
    };
    let (dd, mm) = (
        format!("{:02}", arrival.day()),
        format!("{:02}", arrival.month()),
    );
    let mut vars = vec![("day", dd.as_str()), ("month", mm.as_str())];
    let key = if guest.is_empty() {
        format!("task.unfilled.context.{when}.anonymous")
    } else {
        vars.push(("guest", guest));
        format!("task.unfilled.context.{when}")
    };
    i18n::text_vars(&key, &vars)
}

/// Une tâche par arrivée de la fenêtre annoncée à `late_from` ou après. Rien quand l'hôte a vidé
/// le champ.
pub fn late_arrival_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    timezone: &str,
    property: &str,
) -> Vec<TimelineTask> {
    let Some(late_from) = config.late_from_minutes() else {
        return Vec::new();
    };
    args.stays
        .iter()
        .filter(|stay| (args.from..=args.to).contains(&stay.check_in))
        // ponytail: une lecture par séjour de la fenêtre ; une requête `stay_id IN …` si la
        // fenêtre grossit.
        .filter_map(|stay| {
            let row = storage::find_by_stay(stay.id).ok().flatten()?;
            let time = row.arrival_time?.trim().to_string();
            let hour = checkin_hour(Some(stay.check_in), timezone)?;
            is_late(&time, late_from, hour).then_some((stay, time))
        })
        .map(|(stay, time)| {
            let guest = stay.guest_name.trim();
            let title = if guest.is_empty() {
                i18n::text_vars("task.lateArrival.title.anonymous", &[("time", &time)])
            } else {
                i18n::text_vars(
                    "task.lateArrival.title",
                    &[("guest", guest), ("time", &time)],
                )
            };
            timeline::task(
                format!("late-arrival:{}", stay.id),
                stay.check_in,
                args.property_id,
                title,
                I18nText::new(property, property),
            )
            .stay(stay.id)
        })
        .collect()
}

/// `time` à `late_from` ou après, comptés depuis l'heure d'entrée : les créneaux passent minuit,
/// et « 00:30 » est plus tard que « 22:00 ». Une heure d'avant l'entrée (plus d'une demi-journée
/// « après ») n'est pas tardive, comme dans [`crate::slots::slot_index`].
fn is_late(time: &str, late_from: u32, checkin_hour: u32) -> bool {
    let Some(at) = minutes_of(time) else {
        return false;
    };
    let offset = |minutes: u32| (minutes + 24 * 60 - checkin_hour * 60) % (24 * 60);
    offset(at) < 12 * 60 && offset(at) >= offset(late_from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_is_counted_from_check_in() {
        let ten_pm = 22 * 60;
        assert!(is_late("23:30", ten_pm, 16));
        assert!(is_late("22:00", ten_pm, 16));
        assert!(is_late("00:30", ten_pm, 16));
        assert!(!is_late("18:00", ten_pm, 16));
        // Avant l'entrée : pas tardive.
        assert!(!is_late("11:00", ten_pm, 16));
        assert!(!is_late("midi", ten_pm, 16));
    }
}
