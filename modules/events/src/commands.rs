//! Module commands — configuration persistence and nearby cache refresh.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, EventRow, Localized, ModuleConfig};
use crate::nearby::invalidate_nearby_cache;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventInput {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub starts_at: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub lat: String,
    #[serde(default)]
    pub lng: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub events: Vec<EventInput>,
    #[serde(default)]
    pub disclaimer: String,
    /// `"true"` / `"false"` from host Select (empty keeps previous).
    #[serde(default)]
    pub nearby_enabled: String,
    /// Radius km as string from host Select (empty keeps previous).
    #[serde(default)]
    pub radius_km: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let events = resolve_events(&args.events, &existing.events, &lang);
    let mut disclaimer = existing.disclaimer;
    disclaimer.set(&lang, args.disclaimer.trim().to_string());

    let nearby_enabled = parse_nearby_enabled(&args.nearby_enabled, existing.nearby_enabled);
    let radius_km = parse_radius_km(&args.radius_km, existing.radius_km);
    let radius_changed = radius_km != existing.radius_km;
    let nearby_toggled = nearby_enabled != existing.nearby_enabled;

    save_config(&ModuleConfig {
        events,
        disclaimer,
        nearby_enabled,
        radius_km,
    })?;

    if radius_changed || nearby_toggled {
        let _ = invalidate_nearby_cache();
    }
    Ok(())
}

fn parse_nearby_enabled(raw: &str, fallback: bool) -> bool {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" => fallback,
        "true" | "1" | "on" | "yes" => true,
        "false" | "0" | "off" | "no" => false,
        _ => fallback,
    }
}

fn parse_radius_km(raw: &str, fallback: u32) -> u32 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed.parse::<u32>().unwrap_or(fallback)
}

#[portaki_sdk::command(name = "refreshNearby")]
pub fn refresh_nearby(_ctx: Context) -> Result<()> {
    invalidate_nearby_cache()
}

fn resolve_events(args: &[EventInput], existing: &[EventRow], lang: &str) -> Vec<EventRow> {
    args.iter()
        .enumerate()
        .filter_map(|(index, input)| merge_event(input, existing.get(index), index, lang))
        .collect()
}

fn merge_event(
    input: &EventInput,
    previous: Option<&EventRow>,
    index: usize,
    lang: &str,
) -> Option<EventRow> {
    let title_raw = input.title.trim();
    if title_raw.is_empty() {
        return None;
    }

    let mut title = previous.map(|p| p.title.clone()).unwrap_or_default();
    title.set(lang, title_raw.to_string());

    let mut place = previous.map(|p| p.place.clone()).unwrap_or_default();
    place.set(lang, input.place.trim().to_string());

    Some(EventRow {
        id: previous
            .map(|p| p.id.clone())
            .unwrap_or_else(|| format!("evt-{}", index + 1)),
        title,
        place,
        starts_at: input.starts_at.trim().to_string(),
        ends_at: previous.and_then(|p| p.ends_at.clone()),
        url: nonempty_opt(&input.url),
        lat: parse_coord(&input.lat),
        lng: parse_coord(&input.lng),
        note: previous.and_then(|p| p.note.clone()),
    })
}

fn parse_coord(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

fn nonempty_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
