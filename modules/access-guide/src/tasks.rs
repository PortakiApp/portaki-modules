//! « Code d'entrée manquant · séjour de Marie · arrivée dans 2 j » (spec Accès §1, §9 cas 1–2) et
//! « Changer le code de la boîte à clés · après le départ du 29/08 » (§2.2) : `timelineTasks`,
//! `taskToggle`, `taskComplete`. Calculées, jamais stockées.
//!
//! Le code manquant n'a pas de case : la tâche disparaît quand l'hôte saisit le code dans les
//! réglages — cocher une case ne ferait entrer personne. Le changement de code, lui, se fait hors
//! de Portaki (la boîte elle-même) : rien ne le verrait, alors il a sa case « Code changé », seule
//! gardée, en KV, par tâche.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TimelineTask, TimelineTaskItem, TimelineTasks,
    TimelineTasksArgs,
};
use portaki_sdk::host::{kv, time};
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
    let now = time::now()?;
    let mut tasks = if config.code_per_stay {
        // Les séjours qui ont leur code ; illisible (un bac à sable) : aucune tâche plutôt qu'une
        // alerte sur chaque séjour.
        match portaki_sdk::host::stay_secrets::list(crate::stay_code::KEY) {
            Ok(with_code) => {
                missing_stay_code_tasks(&config, &args, now, &ctx.property.timezone, &with_code)
            }
            Err(_) => Vec::new(),
        }
    } else {
        missing_code_tasks(&config, &args, now, &ctx.property.timezone)
    };
    tasks.extend(rotate_code_tasks(
        &config,
        &args,
        &ctx.property.timezone,
        |id| kv::get(&kv_key(id)).is_ok_and(|done| done.is_some()),
    ));
    Ok(TimelineTasks { tasks })
}

const ROTATE_PREFIX: &str = "rotate-code:";
const ROTATE_ITEM: &str = "changed";
/// Une case cochée se garde deux mois : au-delà, la frise ne remonte plus jusqu'à elle.
const DONE_TTL_SECONDS: u32 = 60 * 24 * 3600;

fn rotate_task_id(stay: Uuid) -> String {
    format!("{ROTATE_PREFIX}{stay}")
}

fn kv_key(task_id: &str) -> String {
    format!("task:{task_id}")
}

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Code changé",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","taskId":"rotate-code:0b6f2c3e-1d4a-4e5b-8c7d-9e0f1a2b3c4d","itemId":"changed","done":true}"#
    )
)]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ROTATE_ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    set_done(&ctx, &args.task_id, args.done)
}

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Code changé",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","taskId":"rotate-code:0b6f2c3e-1d4a-4e5b-8c7d-9e0f1a2b3c4d"}"#
    )
)]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    set_done(&ctx, &args.task_id, true)
}

/// Coche ou décoche « Code changé » ; le code manquant, lui, ne se coche pas.
fn set_done(ctx: &Context, id: &str, done: bool) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let stay = id
        .strip_prefix(ROTATE_PREFIX)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or_else(|| PortakiError::Host("task_not_found".to_string()))?;
    let key = kv_key(&rotate_task_id(stay));
    if done {
        kv::set(&key, b"1", Some(DONE_TTL_SECONDS))
    } else {
        kv::delete(&key)
    }
}

/// Le rappel de rotation (§2.2) : une tâche à chaque départ de la fenêtre — à venir ou tout
/// juste passé, la fenêtre en décide —, tant que l'hôte l'a demandé pour sa boîte à clés.
/// `done` dit, par id de tâche, si « Code changé » est coché.
pub fn rotate_code_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    timezone: &str,
    done: impl Fn(&str) -> bool,
) -> Vec<TimelineTask> {
    if !config.rotate_reminder {
        return Vec::new();
    }
    args.stays
        .iter()
        .filter(|stay| (args.from..=args.to).contains(&stay.check_out))
        .map(|stay| {
            let date = match time::PropertyTz::parse(timezone) {
                Some(tz) => tz.to_local(stay.check_out).format("%d/%m").to_string(),
                None => stay.check_out.format("%d/%m").to_string(),
            };
            let id = rotate_task_id(stay.id);
            let mut item = TimelineTaskItem::new(ROTATE_ITEM, i18n::text("task.rotateCode.item"));
            item.done = done(&id);
            timeline::task(
                id,
                stay.check_out,
                args.property_id,
                i18n::text("task.rotateCode.title"),
                i18n::text_vars("task.rotateCode.context", &[("date", &date)]),
            )
            .stay(stay.id)
            .items(vec![item])
        })
        .collect()
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
    code_tasks(config, args, now, timezone, |_| true)
}

/// « À chaque séjour » : une tâche par arrivée à venir dont le séjour n'a pas son code
/// (`with_code`, les séjours qui l'ont).
pub fn missing_stay_code_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    now: DateTime<Utc>,
    timezone: &str,
    with_code: &[Uuid],
) -> Vec<TimelineTask> {
    if !config.code_per_stay {
        return Vec::new();
    }
    code_tasks(config, args, now, timezone, |stay| {
        !with_code.contains(&stay)
    })
}

fn code_tasks(
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    now: DateTime<Utc>,
    timezone: &str,
    missing: impl Fn(Uuid) -> bool,
) -> Vec<TimelineTask> {
    args.stays
        .iter()
        .filter(|stay| stay.check_in > now && (args.from..=args.to).contains(&stay.check_in))
        .filter(|stay| missing(stay.id))
        .map(|stay| {
            let at = reveal_at(
                config.reveal_policy,
                config.reveal_hours,
                stay.check_in,
                timezone,
            )
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
