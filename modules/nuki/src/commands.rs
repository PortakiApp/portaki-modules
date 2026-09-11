//! Module commands — host config and `access.smart_lock` guest protocol.

use portaki_sdk::host;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ModuleConfig};

#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct StayArgs {
    pub stay_id: Option<String>,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub smartlock_id: String,
    #[serde(default)]
    pub keypad_code: String,
    #[serde(default)]
    pub device_name: String,
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

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        smartlock_id: args.smartlock_id.trim().to_string(),
        keypad_code: args.keypad_code.trim().to_string(),
        device_name: args.device_name.trim().to_string(),
    })
}

#[portaki_sdk::command(name = "getGuestCredential")]
pub fn get_guest_credential(_ctx: Context, _args: StayArgs) -> Result<GuestCredentialResponse> {
    let config = load_config()?;
    let code = require_keypad_code(&config)?;
    Ok(GuestCredentialResponse {
        credential_type: "keypad",
        code,
        smartlock_id: config.smartlock_id.trim().to_string(),
    })
}

#[portaki_sdk::command(name = "unlock")]
pub fn unlock(ctx: Context, _args: StayArgs) -> Result<UnlockResponse> {
    let config = load_config()?;
    let keypad = config.keypad_code_trimmed().to_string();
    let smartlock_id = config.smartlock_id.trim().to_string();

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

fn has_nuki_byok(ctx: &Context) -> bool {
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
