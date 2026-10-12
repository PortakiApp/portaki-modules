//! « Arrivée tardive · Marie · 23:30 » (spec Pré-arrivée §1, §2.1) : `timelineTasks`. Calculée,
//! jamais stockée : une tâche par arrivée de la fenêtre dont le voyageur a annoncé une heure à
//! partir de « Arrivée tardive à partir de ».

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{self, TimelineTask, TimelineTasks, TimelineTasksArgs};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n;
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
    Ok(TimelineTasks {
        tasks: late_arrival_tasks(&config, &args, &ctx.property.timezone, &ctx.property.name),
    })
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
