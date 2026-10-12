//! « Serrure hors ligne » et « Batterie faible » dans À venir (spec Nuki §9) : `timelineTasks`.
//! Calculées, jamais stockées, sans case à cocher : la tâche disparaît quand la serrure répond de
//! nouveau ou que les piles sont changées. Le nom du logement, le dashboard l'ajoute au titre.
//!
//! Lu sur Nuki Web (`GET /smartlock/{smartlockId}`, swagger 4.20.0) : `serverState` (0 = ok ;
//! 1 non enregistrée, 2–3 autorisation invalide, 4 hors ligne — toutes injoignables par Nuki Web,
//! donc toutes « hors ligne » ici), `state.batteryCritical`, `state.batteryCharge` (en %, pack
//! Nuki seulement). Le catalogue rend l'objet entier ; on n'en garde que ces trois champs.

use portaki_sdk::contracts::timeline::{self, TimelineTask, TimelineTasks, TimelineTasksArgs};
use portaki_sdk::host;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n;

/// Sous ce seuil, la batterie est faible (spec Nuki §9).
const LOW_BATTERY_PERCENT: i64 = 20;

/// Le dashboard relit À venir à chaque focus et après chaque case cochée : Nuki Web n'est
/// interrogé qu'une fois toutes les cinq minutes par serrure.
const CACHE_TTL_SECS: u32 = 300;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LockArgs {
    smartlock_id: String,
}

/// Ce que la tâche lit de la serrure — le reste de la réponse Nuki est ignoré.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Health {
    #[serde(default)]
    server_state: Option<i64>,
    #[serde(default)]
    state: Option<State>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    #[serde(default)]
    battery_critical: Option<bool>,
    #[serde(default)]
    battery_charge: Option<i64>,
}

impl Health {
    /// Un champ absent n'est pas une panne : pas de fausse alerte.
    fn offline(&self) -> bool {
        self.server_state.is_some_and(|state| state != 0)
    }

    /// `Some(charge)` quand la batterie est faible — `None` en charge inconnue mais critique.
    /// Une charge de 0 sans « critique » se lit « non mesurée » (piles AA), pas « vide ».
    fn low_battery(&self) -> Option<Option<i64>> {
        let state = self.state.as_ref()?;
        let charge = state
            .battery_charge
            .filter(|charge| (1..LOW_BATTERY_PERCENT).contains(charge));
        (state.battery_critical == Some(true) || charge.is_some()).then_some(charge)
    }
}

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-08-17T00:00:00Z","to":"2026-08-24T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let config = ModuleConfig::load(&ctx)?;
    let smartlock_id = config.smartlock_id_trimmed();
    let now = host::time::now()?;
    // Un état, pas une date : il ne se pose que dans la fenêtre qui contient maintenant.
    if !crate::commands::has_nuki_byok(&ctx)
        || smartlock_id.is_empty()
        || !(args.from..=args.to).contains(&now)
    {
        return Ok(TimelineTasks::default());
    }
    let Some(health) = health(smartlock_id) else {
        return Ok(TimelineTasks::default());
    };
    Ok(TimelineTasks {
        tasks: tasks(&health, &config, &args, now),
    })
}

fn tasks(
    health: &Health,
    config: &ModuleConfig,
    args: &TimelineTasksArgs,
    now: DateTime<Utc>,
) -> Vec<TimelineTask> {
    let mut tasks = Vec::new();
    if health.offline() {
        let context = if config.keypad_code_trimmed().is_empty() {
            "task.offline.context.noFallback"
        } else {
            "task.offline.context"
        };
        tasks.push(timeline::task(
            "lock-offline",
            now,
            args.property_id,
            i18n::text("task.offline.title"),
            i18n::text(context),
        ));
    }
    if let Some(charge) = health.low_battery() {
        let context = match charge {
            Some(charge) => {
                i18n::text_vars("task.battery.context", &[("charge", &charge.to_string())])
            }
            None => i18n::text("task.battery.context.critical"),
        };
        tasks.push(timeline::task(
            "battery-low",
            now,
            args.property_id,
            i18n::text("task.battery.title"),
            context,
        ));
    }
    tasks
}

/// L'état de la serrure, du cache ou de Nuki Web. Nuki qui ne répond pas : pas de tâche, un log.
fn health(smartlock_id: &str) -> Option<Health> {
    let key = format!("lock-health:{smartlock_id}");
    if let Some(cached) = host::kv::get(&key)
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        return Some(cached);
    }
    let fetched: Result<Health> = host::connectors::call(
        "nuki",
        "get_smartlock",
        &LockArgs {
            smartlock_id: smartlock_id.to_string(),
        },
    );
    match fetched {
        Ok(health) => {
            if let Ok(bytes) = serde_json::to_vec(&health) {
                let _ = host::kv::set(&key, &bytes, Some(CACHE_TTL_SECS));
            }
            Some(health)
        }
        Err(error) => {
            let mut fields = host::log::Fields::new();
            fields.insert("error", &error.to_string());
            fields.insert("smartlockId", &smartlock_id);
            let _ = host::log::warn("nuki_lock_health_failed", &fields);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn health(json: &str) -> Health {
        serde_json::from_str(json).expect("health")
    }

    #[test]
    fn any_non_zero_server_state_is_offline_and_a_missing_one_is_not() {
        assert!(!health(r#"{"serverState":0}"#).offline());
        assert!(health(r#"{"serverState":4}"#).offline());
        assert!(health(r#"{"serverState":3}"#).offline());
        assert!(!health("{}").offline());
    }

    #[test]
    fn the_battery_is_low_when_critical_or_under_20_percent() {
        let low = |json: &str| health(json).low_battery();
        assert_eq!(low(r#"{"state":{"batteryCharge":15}}"#), Some(Some(15)));
        assert_eq!(low(r#"{"state":{"batteryCritical":true}}"#), Some(None));
        assert_eq!(
            low(r#"{"state":{"batteryCritical":true,"batteryCharge":8}}"#),
            Some(Some(8))
        );
        assert_eq!(low(r#"{"state":{"batteryCharge":20}}"#), None);
        assert_eq!(low(r#"{"state":{"batteryCharge":0}}"#), None);
        assert_eq!(low(r#"{"state":{"batteryCritical":false}}"#), None);
        assert_eq!(low("{}"), None);
    }
}
