//! Module commands — `access.smart_lock` guest protocol.

use portaki_sdk::host;
use portaki_sdk::prelude::*;
use serde::Serialize;

use crate::config::ModuleConfig;
use crate::stay_code;

#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct StayArgs {
    pub stay_id: Option<String>,
}

#[portaki_sdk::wire(serialize)]
pub struct GuestCredentialResponse {
    #[serde(rename = "type")]
    pub credential_type: &'static str,
    pub code: String,
    pub smartlock_id: String,
    /// La phrase que le livret montre au voyageur, déjà traduite — voir `#[command]` du SDK.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_notice: String,
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub ok: bool,
    pub mode: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub code: String,
    /// Ce que le voyageur lit : la porte s'est ouverte, ou la serrure n'a pas répondu et voici le
    /// code du clavier. Sans elle, le bouton semble ne rien faire.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_notice: String,
}

#[portaki_sdk::wire(serialize)]
struct UnlockConnectorArgs {
    smartlock_id: String,
}

#[portaki_sdk::command(name = "getGuestCredential", guest, example(label = "Code du clavier"))]
pub fn get_guest_credential(ctx: Context, _args: StayArgs) -> Result<GuestCredentialResponse> {
    require_stay_window(&ctx)?;
    let config = ModuleConfig::load(&ctx)?;
    let code = guest_code(&ctx, &config)?;
    Ok(GuestCredentialResponse {
        credential_type: "keypad",
        guest_notice: t!("guest.credential.keypad", code = &code)?,
        code,
        smartlock_id: config.smartlock_id_trimmed().to_string(),
    })
}

#[portaki_sdk::command(name = "unlock", guest, example(label = "Ouvrir la porte"))]
pub fn unlock(ctx: Context, _args: StayArgs) -> Result<UnlockResponse> {
    require_stay_window(&ctx)?;
    let config = ModuleConfig::load(&ctx)?;
    let keypad = config.keypad_code_trimmed().to_string();
    let smartlock_id = config.smartlock_id_trimmed().to_string();

    let mut remote_failed = false;
    if has_nuki_byok(&ctx) && !smartlock_id.is_empty() {
        match try_remote_unlock(&smartlock_id) {
            Ok(()) => {
                return Ok(UnlockResponse {
                    ok: true,
                    mode: "remote",
                    code: String::new(),
                    guest_notice: t!("guest.unlock.opened")?,
                });
            }
            Err(error) => {
                remote_failed = true;
                let mut fields = host::log::Fields::new();
                fields.insert("error", &error.to_string());
                fields.insert("smartlockId", &smartlock_id);
                let _ = host::log::warn("nuki_remote_unlock_failed", &fields);
            }
        }
    }

    // Une serrure qui n'a pas répondu ne recevra pas de nouveau code : le code de secours d'abord.
    let code = if remote_failed && !keypad.is_empty() {
        Ok(keypad)
    } else {
        guest_code(&ctx, &config)
    };
    let Ok(keypad) = code else {
        return Err(PortakiError::Host(
            "unlock unavailable: configure keypad_code or Nuki BYOK + smartlock_id".into(),
        ));
    };

    // Une serrure qui n'a pas répondu se dit ; une serrure qu'on n'a jamais appelée se tait et
    // donne le code. Le voyageur est devant la porte : les deux ont besoin du code.
    let notice = if remote_failed {
        t!("guest.unlock.offline", code = &keypad)?
    } else {
        t!("guest.unlock.keypad", code = &keypad)?
    };
    Ok(UnlockResponse {
        ok: true,
        mode: "credential_fallback",
        code: keypad,
        guest_notice: notice,
    })
}

/// What « Régénérer le code » sends: the stay and its window, as the stay-action surface read
/// them from `input.stay`. A host command has no `ctx.stay`; dates edited by hand only bound a
/// code on the host's own lock.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct RegenerateArgs {
    pub stay_id: Uuid,
    pub check_in: DateTime<Utc>,
    pub check_out: DateTime<Utc>,
}

/// « Régénérer le code » (spec Nuki §1, `stay-action`) : a new code for the stay, from check-in to
/// check-out. The previous one is not revoked — it ends at check-out like every per-stay code.
/// Host only, and only when the module creates per-stay codes (option on, lock, Nuki Web key).
#[portaki_sdk::command(
    name = "regenerateCode",
    example(
        label = "Nouveau code pour le séjour",
        input = r#"{"stayId":"1a2b3c4d-0000-4000-8000-000000000000","checkIn":"2026-08-24T14:00:00Z","checkOut":"2026-08-29T08:00:00Z"}"#
    )
)]
pub fn regenerate_code(ctx: Context, args: RegenerateArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".into()));
    }
    let config = ModuleConfig::load(&ctx)?;
    if !stay_code::wanted(&ctx, &config) {
        return Err(PortakiError::Host("code_per_stay_unavailable".into()));
    }
    let now = host::time::now()?;
    if args.check_out <= now || args.check_in >= args.check_out {
        return Err(PortakiError::Host("stay_ended".into()));
    }
    stay_code::create(
        config.smartlock_id_trimmed(),
        &args.stay_id,
        args.check_in,
        args.check_out,
        now,
    )?;
    Ok(())
}

/// How long before check-in the lock answers. access-guide's reveal policy is not readable
/// from here; its default (J-1 16:00, property time) always falls at least 8 h before check-in,
/// whatever the check-in hour — so 8 h never opens earlier than it.
const OPENS_BEFORE_CHECKIN_SECS: i64 = 8 * 3600;

/// The lock only answers from shortly before check-in until check-out; no stay, no dates → no.
fn require_stay_window(ctx: &Context) -> Result<()> {
    let now = host::time::now()?.timestamp();
    let within = ctx
        .stay
        .as_ref()
        .and_then(|stay| Some((stay.checkin_at?, stay.checkout_at?)))
        .is_some_and(|(checkin, checkout)| {
            now >= checkin.timestamp() - OPENS_BEFORE_CHECKIN_SECS && now <= checkout.timestamp()
        });
    if within {
        Ok(())
    } else {
        Err(PortakiError::Host("outside_stay_window".into()))
    }
}

/// Whether the host stored a Nuki Web key. Named through the SDK id, so a capability rename
/// stops the build here instead of silently answering `false` forever.
pub(crate) fn has_nuki_byok(ctx: &Context) -> bool {
    ctx.has_capability(capability::external::NUKI_BYOK)
}

fn try_remote_unlock(smartlock_id: &str) -> Result<()> {
    let _: serde_json::Value = host::connectors::call(
        "nuki",
        "remote_unlock",
        &UnlockConnectorArgs {
            smartlock_id: smartlock_id.to_string(),
        },
    )?;
    Ok(())
}

/// The code the guest types: the stay's own when the host asked for one and Nuki answers, the
/// shared keypad code otherwise — the fallback of spec §9 #1.
fn guest_code(ctx: &Context, config: &ModuleConfig) -> Result<String> {
    if let Some(code) = stay_code(ctx, config) {
        return Ok(code);
    }
    require_keypad_code(config)
}

fn stay_code(ctx: &Context, config: &ModuleConfig) -> Option<String> {
    if !stay_code::wanted(ctx, config) {
        return None;
    }
    let stay = ctx.stay.as_ref()?;
    let (checkin, checkout) = (stay.checkin_at?, stay.checkout_at?);
    let smartlock_id = config.smartlock_id_trimmed();
    let created = host::time::now()
        .and_then(|now| stay_code::ensure(smartlock_id, &stay.stay_id, checkin, checkout, now));
    match created {
        Ok(code) => Some(code),
        Err(error) => {
            let mut fields = host::log::Fields::new();
            fields.insert("error", &error.to_string());
            fields.insert("smartlockId", &smartlock_id);
            let _ = host::log::warn("nuki_stay_code_failed", &fields);
            None
        }
    }
}

fn require_keypad_code(config: &ModuleConfig) -> Result<String> {
    let code = config.keypad_code_trimmed();
    if code.is_empty() {
        return Err(PortakiError::Host("keypad_code not configured".into()));
    }
    Ok(code.to_string())
}

#[cfg(test)]
mod tests {
    use portaki_sdk::contracts::smart_lock;

    /// UNLOCK / GET_GUEST_CREDENTIAL must stay aligned with the peer protocol.
    #[test]
    fn smart_lock_ops_match_sdk_contract() {
        assert_eq!(super::UNLOCK, smart_lock::UNLOCK);
        assert_eq!(
            super::GET_GUEST_CREDENTIAL,
            smart_lock::GET_GUEST_CREDENTIAL
        );
    }
}
