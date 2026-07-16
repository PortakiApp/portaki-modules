//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{save_config, Localized, ModuleConfig, SpotRow};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpotInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub distance: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub spots: Vec<SpotInput>,
    #[serde(default)]
    pub spots_json: String,
    #[serde(default)]
    pub disclaimer: String,
}

impl UpdateConfigArgs {
    fn resolve_spots(&self) -> Vec<SpotRow> {
        if !self.spots.is_empty() {
            return self
                .spots
                .iter()
                .enumerate()
                .filter_map(|(index, input)| spot_from_input(input, index))
                .collect();
        }
        let raw = self.spots_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<SpotRow>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.id.trim().is_empty())
            .collect()
    }
}

fn spot_from_input(input: &SpotInput, index: usize) -> Option<SpotRow> {
    let name = input.name.trim();
    if name.is_empty() {
        return None;
    }
    let description = input.description.trim();
    Some(SpotRow {
        id: format!("spot-{}", index + 1),
        title: Localized {
            fr: name.to_string(),
            en: name.to_string(),
        },
        url: None,
        category: nonempty_opt(&input.category),
        distance: nonempty_opt(&input.distance),
        tag: nonempty_opt(&input.tag),
        note: None,
        detail: if description.is_empty() {
            None
        } else {
            Some(Localized {
                fr: description.to_string(),
                en: description.to_string(),
            })
        },
    })
}

fn nonempty_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        spots: args.resolve_spots(),
        spots_json: String::new(),
        disclaimer: args.disclaimer,
    })
}
