//! Module commands — `access.smart_lock` guest protocol.

use portaki_sdk::host;
use portaki_sdk::prelude::*;
use serde::Serialize;

use crate::config::ModuleConfig;

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
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub ok: bool,
    pub mode: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub code: String,
}

#[portaki_sdk::wire(serialize)]
struct UnlockConnectorArgs {
    smartlock_id: String,
}

#[portaki_sdk::command(name = "getGuestCredential", guest, example(label = "Code du clavier"))]
pub fn get_guest_credential(ctx: Context, _args: StayArgs) -> Result<GuestCredentialResponse> {
    require_stay_window(&ctx)?;
    let config = ModuleConfig::load(&ctx)?;
    let code = require_keypad_code(&config)?;
    Ok(GuestCredentialResponse {
        credential_type: "keypad",
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

    if has_nuki_byok(&ctx) && !smartlock_id.is_empty() {
        match try_remote_unlock(&smartlock_id) {
            Ok(()) => {
                return Ok(UnlockResponse {
                    ok: true,
                    mode: "remote",
                    code: String::new(),
                });
            }
            Err(error) => {
                let mut fields = host::log::Fields::new();
                fields.insert("error", &error.to_string());
                fields.insert("smartlockId", &smartlock_id);
                let _ = host::log::warn("nuki_remote_unlock_failed", &fields);
            }
        }
    }

    if keypad.is_empty() {
        return Err(PortakiError::Host(
            "unlock unavailable: configure keypad_code or Nuki BYOK + smartlock_id".into(),
        ));
    }

    Ok(UnlockResponse {
        ok: true,
        mode: "credential_fallback",
        code: keypad,
    })
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

pub(crate) fn has_nuki_byok(ctx: &Context) -> bool {
    ctx.capabilities
        .iter()
        .any(|grant| grant.id == crate::NUKI_BYOK || grant.id == "external.nuki.byok")
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

fn require_keypad_code(config: &ModuleConfig) -> Result<String> {
    let code = config.keypad_code_trimmed();
    if code.is_empty() {
        return Err(PortakiError::Host("keypad_code not configured".into()));
    }
    Ok(code.to_string())
}
