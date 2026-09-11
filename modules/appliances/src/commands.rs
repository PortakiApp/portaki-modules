//! Module commands — save / delete / reorder appliances.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::{Appliance, ApplianceStatus, MAX_APPLIANCES, MAX_FEATURED};
use crate::store;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveApplianceArgs {
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: bool,
    pub order: Option<i32>,
    #[serde(default)]
    pub location: String,
    #[serde(default, rename = "manualUrl")]
    pub manual_url: String,
    #[serde(default, rename = "safetyNote")]
    pub safety_note: String,
    #[serde(default)]
    pub status: ApplianceStatus,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteApplianceArgs {
    pub id: String,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderAppliancesArgs {
    #[serde(rename = "orderedIds")]
    pub ordered_ids: Vec<String>,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSafetyNoticeArgs {
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: String,
}

/// Host SDUI form payload — nested `devices.N.*` + `safetyNotice`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceDevicesArgs {
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: String,
    #[serde(default)]
    pub devices: Vec<ReplaceDeviceSlot>,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplaceDeviceSlot {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: serde_json::Value,
    #[serde(default)]
    pub location: String,
    #[serde(default, rename = "manualUrl", alias = "manual_url")]
    pub manual_url: String,
    #[serde(default)]
    pub status: String,
}

#[portaki_sdk::command(name = "saveAppliance")]
pub fn save_appliance(ctx: Context, args: SaveApplianceArgs) -> Result<Appliance> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(PortakiError::Host("appliance name is required".into()));
    }

    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    let is_create = args
        .id
        .as_ref()
        .map(|id| id.trim().is_empty() || !payload.devices.iter().any(|d| d.id == *id))
        .unwrap_or(true);

    if is_create && payload.devices.len() >= MAX_APPLIANCES {
        return Err(PortakiError::Host(format!(
            "max {MAX_APPLIANCES} appliances allowed"
        )));
    }

    let id = args
        .id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let order = args.order.unwrap_or_else(|| {
        payload
            .devices
            .iter()
            .map(|d| d.order)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    });

    let next = Appliance {
        id: id.clone(),
        name,
        emoji: args.emoji.trim().to_string(),
        description: normalize_description(&args.description),
        featured: args.featured,
        order,
        location: args.location.trim().to_string(),
        manual_url: args.manual_url.trim().to_string(),
        safety_note: args.safety_note.trim().to_string(),
        status: args.status,
    };

    if next.featured {
        let featured_others = payload
            .devices
            .iter()
            .filter(|d| d.id != id && d.featured && d.status != ApplianceStatus::Hidden)
            .count();
        if featured_others >= MAX_FEATURED {
            return Err(PortakiError::Host(format!(
                "max {MAX_FEATURED} featured appliances allowed"
            )));
        }
    }

    if let Some(existing) = payload.devices.iter_mut().find(|d| d.id == id) {
        *existing = next.clone();
    } else {
        payload.devices.push(next.clone());
    }

    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(next)
}

#[portaki_sdk::command(name = "deleteAppliance")]
pub fn delete_appliance(ctx: Context, args: DeleteApplianceArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let id = args.id.trim();
    if id.is_empty() {
        return Err(PortakiError::Host("appliance id is required".into()));
    }
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    let before = payload.devices.len();
    payload.devices.retain(|d| d.id != id);
    if payload.devices.len() == before {
        return Err(PortakiError::Host(format!("appliance not found: {id}")));
    }
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

#[portaki_sdk::command(name = "reorderAppliances")]
pub fn reorder_appliances(ctx: Context, args: ReorderAppliancesArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    if args.ordered_ids.is_empty() {
        return Ok(());
    }
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    for (index, id) in args.ordered_ids.iter().enumerate() {
        if let Some(device) = payload.devices.iter_mut().find(|d| d.id == *id) {
            device.order = index as i32;
        }
    }
    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

#[portaki_sdk::command(name = "saveSafetyNotice")]
pub fn save_safety_notice(ctx: Context, args: SaveSafetyNoticeArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    payload.safety_notice = normalize_description(&args.safety_notice);
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

/// Replace the full device list from the host SDUI form (empty name = drop slot).
#[portaki_sdk::command(name = "replaceDevices")]
pub fn replace_devices(ctx: Context, args: ReplaceDevicesArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let mut next_devices: Vec<Appliance> = Vec::new();

    for (index, slot) in args.devices.iter().enumerate() {
        let name = slot.name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        if next_devices.len() >= MAX_APPLIANCES {
            return Err(PortakiError::Host(format!(
                "max {MAX_APPLIANCES} appliances allowed"
            )));
        }

        let id = {
            let trimmed = slot.id.trim();
            if trimmed.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                trimmed.to_string()
            }
        };

        let featured = parse_boolish(&slot.featured);
        let status = if slot.status.trim().eq_ignore_ascii_case("hidden") {
            ApplianceStatus::Hidden
        } else {
            ApplianceStatus::Active
        };

        next_devices.push(Appliance {
            id,
            name,
            emoji: slot.emoji.trim().to_string(),
            description: normalize_description(&slot.description),
            featured,
            order: index as i32,
            location: slot.location.trim().to_string(),
            manual_url: slot.manual_url.trim().to_string(),
            safety_note: String::new(),
            status,
        });
    }

    let featured_count = next_devices
        .iter()
        .filter(|d| d.featured && d.status != ApplianceStatus::Hidden)
        .count();
    if featured_count > MAX_FEATURED {
        return Err(PortakiError::Host(format!(
            "max {MAX_FEATURED} featured appliances allowed"
        )));
    }

    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    payload.safety_notice = normalize_description(&args.safety_notice);
    payload.devices = next_devices;
    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

fn parse_boolish(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::String(s) => matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        ),
        serde_json::Value::Number(n) => n.as_i64() == Some(1),
        _ => false,
    }
}

fn normalize_description(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if v.get("type").and_then(|t| t.as_str()) == Some("doc") {
            return trimmed.to_string();
        }
    }
    // Plain text from TextArea fallback → TipTap paragraph.
    serde_json::json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [{ "type": "text", "text": trimmed }]
        }]
    })
    .to_string()
}
