//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{save_config, AccessStep, Localized, ModuleConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepInput {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub detail_fr: String,
    #[serde(default)]
    pub detail_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub steps: Vec<StepInput>,
    #[serde(default)]
    pub steps_json: String,
    #[serde(default)]
    pub parking_map_url: String,
    #[serde(default)]
    pub arrival_video_url: String,
    #[serde(default)]
    pub global_note: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub gate_code: String,
    #[serde(default)]
    pub keybox_code: String,
    #[serde(default)]
    pub parking_info: String,
}

impl UpdateConfigArgs {
    fn resolve_steps(&self) -> Vec<AccessStep> {
        if !self.steps.is_empty() {
            return self
                .steps
                .iter()
                .enumerate()
                .filter_map(|(index, input)| step_from_input(input, index))
                .collect();
        }
        let raw = self.steps_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<AccessStep>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.id.trim().is_empty())
            .collect()
    }
}

fn step_from_input(input: &StepInput, index: usize) -> Option<AccessStep> {
    if input.title_fr.trim().is_empty() && input.title_en.trim().is_empty() {
        return None;
    }
    let kind = input.kind.trim();
    let detail_fr = input.detail_fr.trim();
    let detail_en = input.detail_en.trim();
    Some(AccessStep {
        id: format!("step-{}", index + 1),
        kind: if kind.is_empty() {
            None
        } else {
            Some(kind.to_string())
        },
        title: Localized {
            fr: input.title_fr.trim().to_string(),
            en: input.title_en.trim().to_string(),
        },
        detail: if detail_fr.is_empty() && detail_en.is_empty() {
            None
        } else {
            Some(Localized {
                fr: detail_fr.to_string(),
                en: detail_en.to_string(),
            })
        },
    })
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        steps: args.resolve_steps(),
        steps_json: String::new(),
        parking_map_url: args.parking_map_url,
        arrival_video_url: args.arrival_video_url,
        global_note: args.global_note,
        address: args.address,
        gate_code: args.gate_code,
        keybox_code: args.keybox_code,
        parking_info: args.parking_info,
    })
}
