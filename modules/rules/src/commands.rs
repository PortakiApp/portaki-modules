//! Module commands — save house rules content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RuleItem, RuleStatus, RulesBundle, RulesPayload};
use crate::store;

/// One rule row from the host form (`items.N.*`) for the active locale.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleItemInput {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    /// `neutral` · `important` · `allowed` — partagé entre les langues.
    #[serde(default)]
    pub status: String,
    /// Clé de thème ([`crate::content::THEMES`]) — partagée entre les langues.
    #[serde(default)]
    pub theme: String,
    /// Heures de la règle (`22:00 – 08:00`) — partagées entre les langues.
    #[serde(default)]
    pub hours: String,
    /// Legacy bilingual fields.
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub subtitle_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub subtitle_en: String,
}

/// Une ligne du tableau hôte (`rules.N.*`), dans l'ordre affiché. Un champ absent garde la valeur
/// enregistrée de la règle de même `id` (une ligne jamais ouverte n'envoie que son id).
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleSlot {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    /// La précision — `subtitle` dans le stockage.
    #[serde(default, alias = "subtitle")]
    pub detail: Option<String>,
    /// `forbidden` · `important` · `allowed` · `neutral`.
    #[serde(default)]
    pub status: Option<String>,
    /// Clé de thème ([`crate::content::THEMES`]).
    #[serde(default)]
    pub theme: Option<String>,
    /// Plage `HH:MM-HH:MM` du `TimeRange`.
    #[serde(default)]
    pub hours: Option<String>,
}

/// Arguments for `saveContent` / `updateConfig` (workspace Save chrome).
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveContentArgs {
    /// Le tableau hôte : les règles de la langue de l'hôte, dans l'ordre affiché, par id.
    #[serde(default)]
    pub rules: Option<Vec<RuleSlot>>,
    /// Ancien formulaire : les règles de la langue de la requête, par rang.
    #[serde(default)]
    pub items: Vec<RuleItemInput>,
    /// Legacy JSON string for French payload.
    #[serde(default)]
    pub content_fr: String,
    /// Legacy JSON string for English payload.
    #[serde(default)]
    pub content_en: String,
    /// « Règles sur la carte » — un nombre : le `NumberInput` n'envoie pas d'entier.
    #[serde(default)]
    pub card_limit: Option<f64>,
}

/// Workspace header Save → nested form `{ items: [{ icon, title, subtitle }] }`.
///
/// Same payload as [`save_content`] — host `HostSurfacePanel` always dispatches
/// `updateConfig` for property-workspace-tab modules.
#[portaki_sdk::command(
    name = "updateConfig",
    example(
        label = "Enregistrer le règlement",
        input = r#"{"card_limit":4,"rules":[{"id":"r0","title":"Silence","detail":"Pensez aux voisins.","status":"important","theme":"noise","hours":"22:00-08:00"},{"id":"r1","title":"Pas de fête","status":"forbidden","theme":"parties"}]}"#
    )
)]
pub fn update_config(ctx: Context, args: SaveContentArgs) -> Result<()> {
    save_content(ctx, args)
}

#[portaki_sdk::command(
    name = "saveContent",
    example(
        label = "Trois règles",
        input = r#"{"items":[{"icon":"clock-circle","title":"Calme après 22 h","subtitle":"Merci de penser au voisinage","status":"important","theme":"Voisinage"},{"icon":"users","title":"Pas de fête ni d'événement","status":"important","theme":"Logement"},{"icon":"check-circle","title":"Animaux bienvenus","subtitle":"Prévenez-nous avant votre arrivée","status":"allowed","theme":"Animaux"}]}"#
    )
)]
pub fn save_content(ctx: Context, args: SaveContentArgs) -> Result<()> {
    let lang = RulesBundle::lang_code(&ctx.locale);
    let existing = store::load_content()?;
    let (prev_fr, prev_en) = match existing {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let mut bundle = RulesBundle::from_row(&prev_fr, &prev_en);
    if let Some(limit) = args.card_limit.filter(|n| n.is_finite() && *n > 0.0) {
        bundle.card_limit = Some(limit.round() as u32);
    }

    if let Some(slots) = args.rules.as_deref() {
        let stored = bundle.get(&lang);
        let payload = merge_rules(&stored, slots);
        let seen: Vec<String> = stored.items.iter().map(|item| item.id.clone()).collect();
        bundle.set(&lang, payload.clone());
        bundle.follow_order(&payload, &seen);
        bundle.sync_shared_from(&payload);
    } else if !args.items.is_empty() {
        let has_legacy_dual = args.items.iter().any(|i| {
            !i.title_fr.trim().is_empty()
                || !i.title_en.trim().is_empty()
                || !i.subtitle_fr.trim().is_empty()
                || !i.subtitle_en.trim().is_empty()
        });
        if has_legacy_dual {
            let (fr, en) = build_payloads_from_legacy_items(&args.items);
            bundle.set("fr", fr.clone());
            bundle.set("en", en);
            bundle.sync_shared_from(&fr);
        } else {
            let mut payload = build_payload_for_lang(&args.items);
            // Rangs de l'ancien formulaire : chaque ligne reprend l'id de la règle de même rang.
            let stored = bundle.get(&lang);
            for (index, item) in payload.items.iter_mut().enumerate() {
                if let Some(previous) = stored.items.get(index) {
                    item.id = previous.id.clone();
                }
            }
            bundle.set(&lang, payload.clone());
            bundle.sync_shared_from(&payload);
        }
    } else if !args.content_fr.trim().is_empty() || !args.content_en.trim().is_empty() {
        // Legacy raw JSON path — seed/migrate into bundle.
        let migrated = RulesBundle::from_row(&args.content_fr, &args.content_en);
        for (k, v) in migrated.by_lang {
            bundle.set(&k, v);
        }
    }

    let json = bundle
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content: {e}")))?;
    let _ = store::save_content_row(json, String::new())?;
    Ok(())
}

/// Les lignes reçues, fusionnées aux règles enregistrées de même `id` : un champ absent garde sa
/// valeur, un champ vide l'efface. Une ligne sans titre ni précision s'en va ; au-delà de 30, rien
/// n'est refusé — l'erreur bloque « Publier », jamais l'enregistrement.
fn merge_rules(stored: &RulesPayload, slots: &[RuleSlot]) -> RulesPayload {
    let mut items = Vec::new();
    for slot in slots {
        let id = slot.id.trim();
        let previous = (!id.is_empty()).then(|| stored.find(id)).flatten();
        let text = |sent: &Option<String>, kept: fn(&RuleItem) -> &String| {
            sent.as_deref()
                .map(|value| value.trim().to_string())
                .or_else(|| previous.map(|item| kept(item).clone()))
                .unwrap_or_default()
        };
        let rule = RuleItem {
            id: if id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                id.to_string()
            },
            icon: previous.map(|item| item.icon.clone()).unwrap_or_default(),
            title: text(&slot.title, |item| &item.title),
            subtitle: text(&slot.detail, |item| &item.subtitle),
            status: match slot.status.as_deref() {
                Some(raw) => RuleStatus::from_wire(raw),
                None => previous.map(|item| item.status).unwrap_or_default(),
            },
            theme: text(&slot.theme, |item| &item.theme),
            hours: text(&slot.hours, |item| &item.hours),
        };
        if !rule.is_blank() {
            items.push(rule);
        }
    }
    RulesPayload {
        items,
        ..RulesPayload::default()
    }
}

/// Arguments de `applyTemplate` : aucun.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplyTemplateArgs {}

/// « Partir d'un modèle » (§4) : silence, animaux, tabac, fêtes, visiteurs, dans la langue de
/// l'hôte. Seulement sur un règlement vide dans cette langue : un second clic n'écrase rien.
#[portaki_sdk::command(
    name = "applyTemplate",
    example(label = "Partir d'un modèle", input = "{}")
)]
pub fn apply_template(ctx: Context, _args: ApplyTemplateArgs) -> Result<()> {
    let lang = RulesBundle::lang_code(&ctx.locale);
    let (prev_fr, prev_en) = store::load_content()?
        .map(|row| (row.content_fr, row.content_en))
        .unwrap_or_default();
    let mut bundle = RulesBundle::from_row(&prev_fr, &prev_en);
    if !bundle.get(&lang).is_empty() {
        return Ok(());
    }
    let payload = template();
    bundle.set(&lang, payload.clone());
    bundle.sync_shared_from(&payload);
    let json = bundle
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content: {e}")))?;
    let _ = store::save_content_row(json, String::new())?;
    Ok(())
}

/// Le modèle : le texte vient du dictionnaire de l'hôte, les ids sont fixes (`t-…`) pour que la
/// même règle se retrouve d'une langue à l'autre si l'hôte part du modèle dans deux langues.
pub(crate) fn template() -> RulesPayload {
    let rule = |key: &str, status, theme: &str, hours: &str| RuleItem {
        id: format!("t-{key}"),
        title: t!(&format!("host.template.{key}.title"))
            .unwrap_or_else(|_| format!("i18n:host.template.{key}.title")),
        subtitle: t!(&format!("host.template.{key}.detail"))
            .unwrap_or_else(|_| format!("i18n:host.template.{key}.detail")),
        status,
        theme: theme.into(),
        hours: hours.into(),
        icon: String::new(),
    };
    RulesPayload {
        items: vec![
            rule("silence", RuleStatus::Important, "noise", "22:00-08:00"),
            rule("pets", RuleStatus::Allowed, "pets", ""),
            rule("smoking", RuleStatus::Forbidden, "smoking", ""),
            rule("parties", RuleStatus::Forbidden, "parties", ""),
            rule("visitors", RuleStatus::Neutral, "visitors", ""),
        ],
        ..RulesPayload::default()
    }
}

fn build_payload_for_lang(items: &[RuleItemInput]) -> RulesPayload {
    let mut out = Vec::new();
    for item in items {
        let rule = RuleItem {
            id: String::new(),
            icon: item.icon.trim().to_string(),
            title: item.title.trim().to_string(),
            subtitle: item.subtitle.trim().to_string(),
            status: RuleStatus::from_wire(&item.status),
            theme: item.theme.trim().to_string(),
            hours: item.hours.trim().to_string(),
        };
        // Une ligne vidée s'en va ; une précision sans titre reste, avec « Écrivez la règle. »
        // sous son titre — l'écarter perdait ce que l'hôte avait écrit, sans un mot.
        if !rule.is_blank() {
            out.push(rule);
        }
    }
    RulesPayload {
        items: out,
        ..RulesPayload::default()
    }
}

fn build_payloads_from_legacy_items(items: &[RuleItemInput]) -> (RulesPayload, RulesPayload) {
    let mut fr_items = Vec::new();
    let mut en_items = Vec::new();
    for item in items {
        if item.title_fr.trim().is_empty() && item.title_en.trim().is_empty() {
            continue;
        }
        let icon = item.icon.trim().to_string();
        let status = RuleStatus::from_wire(&item.status);
        let theme = item.theme.trim().to_string();
        let hours = item.hours.trim().to_string();
        fr_items.push(RuleItem {
            id: String::new(),
            icon: icon.clone(),
            title: item.title_fr.trim().to_string(),
            subtitle: item.subtitle_fr.trim().to_string(),
            status,
            theme: theme.clone(),
            hours: hours.clone(),
        });
        en_items.push(RuleItem {
            id: String::new(),
            icon,
            title: item.title_en.trim().to_string(),
            subtitle: item.subtitle_en.trim().to_string(),
            status,
            theme,
            hours,
        });
    }
    (
        RulesPayload {
            items: fr_items,
            ..RulesPayload::default()
        },
        RulesPayload {
            items: en_items,
            ..RulesPayload::default()
        },
    )
}
