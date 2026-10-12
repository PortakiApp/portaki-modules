//! Un code clavier par séjour (spec Nuki §2.2) — Nuki Web en est la mémoire.
//!
//! Pas de KV : un code écrit depuis le dashboard irait dans la couche brouillon, que le livret ne
//! lit pas. L'autorisation porte le nom `Portaki <8 premiers caractères du séjour>` ; on la cherche
//! sur la serrure, on la crée au premier affichage si elle n'y est pas. Elle vaut de l'arrivée au
//! départ, toujours, puis le premier affichage d'un séjour suivant la supprime : un clavier Nuki a
//! un nombre de codes limité.
//!
//! Le catalogue ouvre la suppression d'une autorisation, sans voir son nom : ne toucher qu'aux
//! codes `Portaki <8 hex>` expirés est la garde de ce module, et la seule.

use portaki_sdk::host;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Nuki's authorization type for a keypad code.
const KEYPAD_CODE: i64 = 13;

/// Every day of the week (Nuki's bitmask, Monday = 64 … Sunday = 1).
const ALL_WEEK_DAYS: u8 = 127;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LockArgs {
    smartlock_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteArgs {
    smartlock_id: String,
    /// Nuki's `id` of the authorization (not its lock-local `authId`).
    auth_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NukiAuth {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: String,
    #[serde(rename = "type", default)]
    auth_type: i64,
    #[serde(default)]
    code: Option<u64>,
    #[serde(default)]
    allowed_until_date: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAuth {
    smartlock_id: String,
    name: String,
    #[serde(rename = "type")]
    auth_type: i64,
    code: u32,
    allowed_from_date: String,
    allowed_until_date: String,
    allowed_week_days: u8,
    allowed_from_time: u32,
    allowed_until_time: u32,
    remote_allowed: bool,
}

/// What the host asked for and the module can do: a code per stay, a lock, the Nuki Web key.
/// Otherwise the shared keypad code is the guest's.
pub(crate) fn wanted(ctx: &Context, config: &ModuleConfig) -> bool {
    config.code_per_stay
        && !config.smartlock_id_trimmed().is_empty()
        && crate::commands::has_nuki_byok(ctx)
}

/// The authorization's name on the lock — Nuki caps keypad names at 20 characters.
pub(crate) fn auth_name(stay_id: &Uuid) -> String {
    format!("Portaki {}", &stay_id.simple().to_string()[..8])
}

fn list(smartlock_id: &str) -> Result<Vec<NukiAuth>> {
    host::connectors::call(
        "nuki",
        "list_auths",
        &LockArgs {
            smartlock_id: smartlock_id.to_string(),
        },
    )
}

/// The stay's code on the lock, if Nuki holds one.
pub(crate) fn find(smartlock_id: &str, stay_id: &Uuid) -> Result<Option<String>> {
    find_in(list(smartlock_id)?, stay_id)
}

fn find_in(auths: Vec<NukiAuth>, stay_id: &Uuid) -> Result<Option<String>> {
    let name = auth_name(stay_id);
    match auths
        .into_iter()
        .find(|auth| auth.auth_type == KEYPAD_CODE && auth.name == name)
    {
        None => Ok(None),
        Some(NukiAuth {
            code: Some(code), ..
        }) => Ok(Some(code.to_string())),
        // Une autorisation à ce nom mais sans code lisible : en créer une seconde donnerait deux
        // codes au même séjour. Le code de secours vaut mieux.
        Some(_) => Err(PortakiError::Host("nuki_auth_without_code".into())),
    }
}

/// The stay's code: found on the lock, or created there for check-in → check-out.
pub(crate) fn ensure(
    smartlock_id: &str,
    stay_id: &Uuid,
    checkin: DateTime<Utc>,
    checkout: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<String> {
    let auths = list(smartlock_id)?;
    // Avant de créer : c'est la place qu'un code expiré tient sur le clavier.
    purge_expired(smartlock_id, &auths, now);
    if let Some(code) = find_in(auths, stay_id)? {
        return Ok(code);
    }
    let code = generate(stay_id, now);
    let _: serde_json::Value = host::connectors::call(
        "nuki",
        "create_auth",
        &CreateAuth {
            smartlock_id: smartlock_id.to_string(),
            name: auth_name(stay_id),
            auth_type: KEYPAD_CODE,
            code,
            allowed_from_date: nuki_date(checkin),
            allowed_until_date: nuki_date(checkout),
            allowed_week_days: ALL_WEEK_DAYS,
            allowed_from_time: 0,
            allowed_until_time: 0,
            remote_allowed: false,
        },
    )?;
    Ok(code.to_string())
}

/// A keypad code this module created (`Portaki ` + 8 hex) whose validity ended. No end date, an
/// unreadable one, or any other name: the host's, never touched.
fn is_expired_portaki_code(auth: &NukiAuth, now: DateTime<Utc>) -> bool {
    let ours = auth
        .name
        .strip_prefix("Portaki ")
        .is_some_and(|suffix| suffix.len() == 8 && suffix.bytes().all(|b| b.is_ascii_hexdigit()));
    let ended = auth
        .allowed_until_date
        .as_deref()
        .and_then(|until| DateTime::parse_from_rfc3339(until).ok())
        .is_some_and(|until| until < now);
    auth.auth_type == KEYPAD_CODE && ours && ended
}

/// Deletes the lock's expired Portaki codes; a refused delete is logged and retried next time.
/// Nuki deletes asynchronously: a code listed again right after may answer 404, logged the same.
fn purge_expired(smartlock_id: &str, auths: &[NukiAuth], now: DateTime<Utc>) {
    for auth in auths
        .iter()
        .filter(|auth| is_expired_portaki_code(auth, now))
    {
        let Some(auth_id) = auth.id.clone() else {
            continue;
        };
        let deleted: Result<serde_json::Value> = host::connectors::call(
            "nuki",
            "delete_auth",
            &DeleteArgs {
                smartlock_id: smartlock_id.to_string(),
                auth_id,
            },
        );
        if let Err(error) = deleted {
            let mut fields = host::log::Fields::new();
            fields.insert("error", &error.to_string());
            fields.insert("smartlockId", &smartlock_id);
            let _ = host::log::warn("nuki_expired_code_delete_failed", &fields);
        }
    }
}

fn nuki_date(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Six digits from 1 to 9 (the keypad has no 0 to lead with), not starting with 12 (Nuki
/// refuses it).
///
/// ponytail: not a CSPRNG — the module's Wasm has no entropy source (the SDK's `getrandom` is a
/// fixed-seed xorshift). Seeded from the stay id (a random UUID only its guest holds) and the
/// instant; switch to host-provided random bytes when the SDK offers them.
pub(crate) fn generate(stay_id: &Uuid, now: DateTime<Utc>) -> u32 {
    let (high, low) = stay_id.as_u64_pair();
    let nanos = now.timestamp_nanos_opt().unwrap_or_default() as u64;
    let mut state = high ^ low.rotate_left(17) ^ nanos;
    loop {
        let code = (0..6).fold(0u32, |code, _| {
            code * 10 + 1 + (splitmix64(&mut state) % 9) as u32
        });
        if code / 10_000 != 12 {
            return code;
        }
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_follow_the_keypad_rule_and_differ_by_stay() {
        let now = DateTime::parse_from_rfc3339("2026-08-23T16:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..500 {
            let code = generate(&Uuid::new_v4(), now).to_string();
            assert_eq!(code.len(), 6, "{code}");
            assert!(!code.contains('0'), "{code}");
            assert!(!code.starts_with("12"), "{code}");
            seen.insert(code);
        }
        assert!(seen.len() > 490, "codes repeat: {}", seen.len());
    }

    #[test]
    fn the_name_fits_a_nuki_keypad_name() {
        let id = Uuid::parse_str("1a2b3c4d-0000-4000-8000-000000000000").unwrap();
        assert_eq!(auth_name(&id), "Portaki 1a2b3c4d");
        assert!(auth_name(&id).len() <= 20);
    }
}
