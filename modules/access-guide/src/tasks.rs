//! « Code manquant · séjour de Marie · arrivée dans 2 j » (spec Accès §1, §9 cas 1–2) :
//! `timelineTasks`. Calculée, jamais stockée, sans case à cocher : la tâche disparaît quand
//! l'hôte saisit le code dans les réglages — cocher une case ne ferait entrer personne.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::timeline::{self, TimelineTask, TimelineTasks, TimelineTasksArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n;
use crate::reveal::reveal_at;

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-08-17T00:00:00Z","to":"2026-08-24T00:00:00Z","stays":[{"id":"0b6f2c3e-1d4a-4e5b-8c7d-9e0f1a2b3c4d","checkIn":"2026-08-21T14:00:00Z","checkOut":"2026-08-24T08:00:00Z","guestName":"Marie","status":"UPCOMING"}]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let config = ModuleConfig::read(&ctx)?;
    Ok(TimelineTasks {
        tasks: missing_code_tasks(&config, &args, time::now()?, &ctx.property.timezone),
    })
}

/// Une tâche par arrivée à venir de la fenêtre, tant que la méthode demande un code et que
/// l'hôte ne l'a pas saisi. Posée au moment où le voyageur aurait dû le voir, ramené au début de
/// la fenêtre et jamais après l'arrivée.
pub fn missing_code_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    now: DateTime<Utc>,
    timezone: &str,
) -> Vec<TimelineTask> {
    if !config.entry_code_missing() {
        return Vec::new();
    }
    args.stays
        .iter()
        .filter(|stay| stay.check_in > now && (args.from..=args.to).contains(&stay.check_in))
        .map(|stay| {
            let at = reveal_at(config.reveal_policy, stay.check_in, timezone)
                .unwrap_or(now)
                .max(args.from)
                .min(stay.check_in);
            // « dans 2 j » : les jours pleins, au moins un.
            let days = (stay.check_in - now).num_days().max(1).to_string();
            let guest = stay.guest_name.trim();
            let context = if guest.is_empty() {
                i18n::text_vars("task.missingCode.context.anonymous", &[("days", &days)])
            } else {
                i18n::text_vars(
                    "task.missingCode.context",
                    &[("guest", guest), ("days", &days)],
                )
            };
            timeline::task(
                format!("missing-code:{}", stay.id),
                at,
                args.property_id,
                i18n::text("task.missingCode.title"),
                context,
            )
            .stay(stay.id)
        })
        .collect()
}
