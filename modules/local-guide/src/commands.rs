//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::affiliate::{normalize_curated_url, CuratedUrlError, MAX_CURATED_LINKS};
use crate::config::{
    load_config, save_config, ActivitiesConfig, ActivityRow, Localized, ModuleConfig, SpotRow,
};

/// Une URL proposée par l'hôte n'est ni GetYourGuide ni un lien court `gyg.me`.
pub const ERR_ACTIVITY_URL_NOT_GYG: &str = "activities_url_not_getyourguide";

/// Plus de [`MAX_CURATED_LINKS`] liens soumis.
pub const ERR_ACTIVITIES_TOO_MANY: &str = "activities_too_many";

#[portaki_sdk::params]
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

/// Une ligne du tableau « Activités & billets » du formulaire hôte.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityInput {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub label: String,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub spots: Vec<SpotInput>,
    #[serde(default)]
    pub spots_json: String,
    #[serde(default)]
    pub disclaimer: String,
    /// `None` = champ non soumis, on garde l'état enregistré.
    #[serde(default)]
    pub activities_enabled: Option<bool>,
    #[serde(default)]
    pub activities_destination: String,
    #[serde(default)]
    pub activities_intro: String,
    #[serde(default)]
    pub activities: Vec<ActivityInput>,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let spots = resolve_spots(&args, &existing.spots, &lang);
    let activities = resolve_activities(&args, &existing.activities, &lang)?;
    let mut disclaimer = existing.disclaimer;
    disclaimer.set(&lang, args.disclaimer.trim().to_string());
    save_config(&ModuleConfig {
        spots,
        spots_json: String::new(),
        disclaimer,
        activities,
    })
}

/// Valide et normalise la section activités soumise par l'hôte.
///
/// Un domaine étranger est refusé ici, et non silencieusement ignoré : l'hôte a collé une
/// URL en pensant l'afficher, la faire disparaître sans rien dire serait pire que l'erreur.
fn resolve_activities(
    args: &UpdateConfigArgs,
    existing: &ActivitiesConfig,
    lang: &str,
) -> Result<ActivitiesConfig> {
    let mut intro = existing.intro.clone();
    intro.set(lang, args.activities_intro.trim().to_string());

    let links = if args.activities.is_empty() {
        // Champ non soumis (appelant plus ancien que la section) — on ne vide rien.
        existing.links.clone()
    } else {
        resolve_activity_links(&args.activities, &existing.links, lang)?
    };

    Ok(ActivitiesConfig {
        enabled: args.activities_enabled.unwrap_or(existing.enabled),
        destination: args.activities_destination.trim().to_string(),
        intro,
        links,
    })
}

fn resolve_activity_links(
    inputs: &[ActivityInput],
    existing: &[ActivityRow],
    lang: &str,
) -> Result<Vec<ActivityRow>> {
    let mut rows = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        let url = match normalize_curated_url(&input.url) {
            Ok(url) => url,
            // Ligne laissée vide : c'est un créneau libre, pas une faute.
            Err(CuratedUrlError::Empty) => continue,
            Err(CuratedUrlError::NotGetYourGuide) => {
                return Err(PortakiError::Host(ERR_ACTIVITY_URL_NOT_GYG.to_string()))
            }
        };

        // L'index est celui du créneau affiché : il désigne la même ligne que celle
        // rendue depuis la configuration, donc les autres langues du libellé suivent.
        let mut label = existing
            .get(index)
            .map(|row| row.label.clone())
            .unwrap_or_default();
        label.set(lang, input.label.trim().to_string());

        rows.push(ActivityRow { url, label });
    }

    if rows.len() > MAX_CURATED_LINKS {
        return Err(PortakiError::Host(ERR_ACTIVITIES_TOO_MANY.to_string()));
    }
    Ok(rows)
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
