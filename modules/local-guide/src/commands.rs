//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, Localized, ModuleConfig, SpotRow};

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

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let spots = resolve_spots(&args, &existing.spots, &lang);
    let mut disclaimer = existing.disclaimer;
    disclaimer.set(&lang, args.disclaimer.trim().to_string());
    save_config(&ModuleConfig {
        spots,
        spots_json: String::new(),
        disclaimer,
    })
}

fn resolve_spots(args: &UpdateConfigArgs, existing: &[SpotRow], lang: &str) -> Vec<SpotRow> {
    if !args.spots.is_empty() {
        return args
            .spots
            .iter()
            .enumerate()
            .filter_map(|(index, input)| merge_spot(input, existing.get(index), index, lang))
            .collect();
    }
    let raw = args.spots_json.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<SpotRow>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter(|s| !s.id.trim().is_empty())
        .collect()
}

fn merge_spot(
    input: &SpotInput,
    previous: Option<&SpotRow>,
    index: usize,
    lang: &str,
) -> Option<SpotRow> {
    let name = input.name.trim();
    if name.is_empty() {
        return None;
    }
    let mut title = previous.map(|p| p.title.clone()).unwrap_or_default();
    title.set(lang, name.to_string());

    let mut detail = previous.and_then(|p| p.detail.clone()).unwrap_or_default();
    detail.set(lang, input.description.trim().to_string());
    let detail = if detail.is_empty() {
        None
    } else {
        Some(detail)
    };

    Some(SpotRow {
        id: previous
            .map(|p| p.id.clone())
            .unwrap_or_else(|| format!("spot-{}", index + 1)),
        title,
        url: previous.and_then(|p| p.url.clone()),
        category: nonempty_opt(&input.category),
        distance: nonempty_opt(&input.distance),
        tag: nonempty_opt(&input.tag),
        note: previous.and_then(|p| p.note.clone()),
        detail,
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
