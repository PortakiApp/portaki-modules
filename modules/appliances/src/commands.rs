//! Module commands — save appliance guide.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{ApplianceDevice, AppliancesPayload};
use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInput {
    #[serde(default)]
    pub name_fr: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub subtitle_fr: String,
    #[serde(default)]
    pub subtitle_en: String,
    #[serde(default)]
    pub steps_fr: String,
    #[serde(default)]
    pub steps_en: String,
    #[serde(default)]
    pub tip_fr: String,
    #[serde(default)]
    pub tip_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveContentArgs {
    #[serde(default)]
    pub devices: Vec<DeviceInput>,
    #[serde(default)]
    pub safety_notice_fr: String,
    #[serde(default)]
    pub safety_notice_en: String,
    /// Legacy JSON string for French payload.
    #[serde(default)]
    pub content_fr: String,
    /// Legacy JSON string for English payload.
    #[serde(default)]
    pub content_en: String,
}

#[portaki_sdk::command(name = "saveContent")]
pub fn save_content(_ctx: Context, args: SaveContentArgs) -> Result<()> {
    let (fr, en) = if !args.devices.is_empty()
        || !args.safety_notice_fr.trim().is_empty()
        || !args.safety_notice_en.trim().is_empty()
    {
        build_payloads_from_form(&args)
    } else {
        (
            AppliancesPayload::parse(&args.content_fr),
            AppliancesPayload::parse(&args.content_en),
        )
    };
    let fr = fr
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content_fr: {e}")))?;
    let en = en
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content_en: {e}")))?;
    let _ = store::save_content_row(fr, en)?;
    Ok(())
}

fn build_payloads_from_form(args: &SaveContentArgs) -> (AppliancesPayload, AppliancesPayload) {
    let mut fr_devices = Vec::new();
    let mut en_devices = Vec::new();
    for (index, input) in args.devices.iter().enumerate() {
        if input.name_fr.trim().is_empty() && input.name_en.trim().is_empty() {
            continue;
        }
        let id = format!("device-{}", index + 1);
        fr_devices.push(ApplianceDevice {
            id: id.clone(),
            icon: String::new(),
            title: input.name_fr.trim().to_string(),
            subtitle: input.subtitle_fr.trim().to_string(),
            steps: lines_to_steps(&input.steps_fr),
            tip: input.tip_fr.trim().to_string(),
            manual_url: String::new(),
        });
        en_devices.push(ApplianceDevice {
            id,
            icon: String::new(),
            title: input.name_en.trim().to_string(),
            subtitle: input.subtitle_en.trim().to_string(),
            steps: lines_to_steps(&input.steps_en),
            tip: input.tip_en.trim().to_string(),
            manual_url: String::new(),
        });
    }
    (
        AppliancesPayload {
            safety_notice: args.safety_notice_fr.trim().to_string(),
            devices: fr_devices,
        },
        AppliancesPayload {
            safety_notice: args.safety_notice_en.trim().to_string(),
            devices: en_devices,
        },
    )
}

fn lines_to_steps(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}
