//! Module commands — host catalog + guest submit + host status.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email_send;
use crate::email_text;
use crate::labels::{self, lang_code};
use crate::level;
use crate::status;
use crate::storage;

/// Single item payload for `replaceItems` / `updateConfig`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableItemInput {
    #[serde(default)]
    pub label: String,
    #[serde(default, alias = "labelFr")]
    pub label_fr: String,
    #[serde(default, alias = "labelEn")]
    pub label_en: String,
    #[serde(default, alias = "sortOrder")]
    pub sort_order: i32,
    #[serde(default, alias = "lowThreshold")]
    pub low_threshold: i32,
    #[serde(default)]
    pub emoji: String,
}

/// Arguments for `replaceItems`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceItemsArgs {
    #[serde(default)]
    pub items: Vec<ConsumableItemInput>,
    #[serde(default, alias = "itemsJson")]
    pub items_json: Option<String>,
}

impl ReplaceItemsArgs {
    fn resolve_items(&self) -> Result<Vec<ConsumableItemInput>> {
        let from_array: Vec<ConsumableItemInput> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let empty = item.label.trim().is_empty()
                    && item.label_fr.trim().is_empty()
                    && item.label_en.trim().is_empty();
                if empty {
                    return None;
                }
                Some(ConsumableItemInput {
                    emoji: item.emoji.trim().to_string(),
                    label: item.label.trim().to_string(),
                    label_fr: item.label_fr.trim().to_string(),
                    label_en: item.label_en.trim().to_string(),
                    sort_order: if item.sort_order == 0 {
                        index as i32
                    } else {
                        item.sort_order
                    },
                    low_threshold: item.low_threshold.max(0),
                })
            })
            .collect();
        if !from_array.is_empty() || self.items_json.is_none() {
            return Ok(from_array);
        }
        let Some(raw) = self.items_json.as_ref() else {
            return Ok(Vec::new());
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(trimmed)
            .map_err(|error| PortakiError::Host(format!("invalid items_json: {error}")))
    }
}

/// Workspace header Save → nested form `{ items: [{ label }] }`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub items: Vec<ConsumableItemInput>,
    /// Le délai de réapprovisionnement annoncé au voyageur : « sous 24 h », « le lendemain
    /// matin » (§2.5). Du texte et non un nombre d'heures — « le lendemain matin » n'en est pas
    /// un, et c'est ce qu'un hôte écrit.
    #[serde(default, alias = "restockDelay")]
    pub restock_delay: I18nText,
}

/// Persists catalog items from the host workspace Save chrome.
#[portaki_sdk::command(
    name = "updateConfig",
    example(
        label = "Papier toilette et café",
        input = r#"{"items":[{"label_fr":"Papier toilette"},{"label_fr":"Café"}]}"#
    )
)]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    // Le délai d'abord : si le catalogue échoue, l'hôte voit son erreur sans avoir perdu sa
    // promesse de réapprovisionnement, qui n'y est pour rien.
    storage::restock_delay::write(&args.restock_delay)?;
    replace_items(
        ctx,
        ReplaceItemsArgs {
            items: args.items,
            items_json: None,
        },
    )
}

#[portaki_sdk::command(
    name = "replaceItems",
    example(
        label = "Catalogue bilingue",
        input = r#"{"items":[{"label_fr":"Papier toilette","label_en":"Toilet paper","low_threshold":2},{"label_fr":"Café","label_en":"Coffee"}]}"#
    )
)]
pub fn replace_items(ctx: Context, args: ReplaceItemsArgs) -> Result<()> {
    let lang = lang_code(&ctx.locale);
    let items = args.resolve_items()?;
    let existing = storage::list_items()?;
    let mut next = Vec::new();
    for (index, input) in items.into_iter().enumerate() {
        let mut labels_map = existing
            .get(index)
            .map(labels::labels_from_item)
            .unwrap_or_default();
        if !input.label.trim().is_empty() {
            labels_map.insert(lang.clone(), input.label.trim().to_string());
        } else {
            if !input.label_fr.trim().is_empty() {
                labels_map.insert("fr".into(), input.label_fr.trim().to_string());
            }
            if !input.label_en.trim().is_empty() {
                labels_map.insert("en".into(), input.label_en.trim().to_string());
            }
        }
        let (label_fr, label_en) = labels::encode_labels(&labels_map);
        let id = existing.get(index).map(|item| item.id);
        let low_threshold = existing
            .get(index)
            .map(|item| item.low_threshold)
            .unwrap_or(input.low_threshold);
        // L'emoji suit la ligne, et un champ vide ne l'efface pas : l'hôte qui renomme un
        // produit dans une autre langue ne doit pas perdre le pictogramme qu'il avait choisi.
        let emoji = if input.emoji.trim().is_empty() {
            existing
                .get(index)
                .map(|item| item.emoji.clone())
                .unwrap_or_default()
        } else {
            input.emoji.trim().to_string()
        };
        next.push(storage::ItemDraft {
            id,
            label_fr,
            label_en,
            sort_order: input.sort_order,
            low_threshold: if input.low_threshold > 0 {
                input.low_threshold
            } else {
                low_threshold
            },
            emoji,
        });
    }
    storage::replace_items_preserving_ids(next)
}

/// Fills an empty catalog with common consumables (FR/EN). No-op if items exist.
#[portaki_sdk::command(name = "seedDefaults", example(label = "Catalogue par défaut"))]
pub fn seed_defaults(ctx: Context, _args: EmptyArgs) -> Result<()> {
    if !storage::list_items()?.is_empty() {
        return Ok(());
    }
    replace_items(
        ctx,
        ReplaceItemsArgs {
            items: default_catalog(),
            items_json: None,
        },
    )
}

fn default_catalog() -> Vec<ConsumableItemInput> {
    // Chacun avec son emoji : la grille du voyageur se lit alors d'un coup d'œil, et un hôte qui
    // clique « Partir de la liste courante » n'a pas huit colis identiques à distinguer.
    [
        ("Papier toilette", "Toilet paper", "🧻"),
        ("Savon", "Hand soap", "🧼"),
        ("Gel douche", "Shower gel", "🚿"),
        ("Shampoing", "Shampoo", "🧴"),
        ("Café", "Coffee", "☕"),
        ("Tablettes lave-vaisselle", "Dishwasher tablets", "🍽️"),
        ("Essuie-tout", "Paper towels", "🧽"),
        ("Lessive", "Laundry detergent", "🧺"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (fr, en, emoji))| ConsumableItemInput {
        label: String::new(),
        label_fr: fr.into(),
        label_en: en.into(),
        sort_order: index as i32,
        low_threshold: 0,
        emoji: emoji.into(),
    })
    .collect()
}

/// Arguments for guest `submit`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitArgs {
    /// Les produits signalés d'un coup (§2.5 : grille de tuiles, choix multiple).
    #[serde(default)]
    pub item_ids: Vec<Uuid>,
    /// L'ancien champ à un seul produit, encore accepté : un formulaire déjà ouvert dans le
    /// téléphone d'un voyageur au moment du déploiement l'envoie toujours.
    #[serde(default)]
    pub item_id: Option<Uuid>,
    pub level: String,
    #[serde(default)]
    pub note: Option<String>,
}

impl SubmitArgs {
    /// Les produits à signaler, quelle que soit la forme reçue, sans doublon et dans l'ordre choisi.
    fn selected_items(&self) -> Vec<Uuid> {
        let mut ids: Vec<Uuid> = Vec::new();
        for id in self.item_ids.iter().chain(self.item_id.iter()) {
            if !ids.contains(id) {
                ids.push(*id);
            }
        }
        ids
    }
}

#[portaki_sdk::command(
    name = "submit",
    guest,
    example(
        label = "Plus de papier toilette",
        input = r#"{"itemIds":["3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47"],"level":"missing","note":"Plus un rouleau dans la salle de bain"}"#
    ),
    example(
        label = "Café presque fini",
        input = r#"{"itemId":"3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47","level":"low"}"#
    )
)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_guest_stay_id(&ctx)?;
    let level = level::parse_level(&args.level)?;
    // Les produits d'abord : `note` consomme `args`, et la liste se lit encore par référence.
    let selected = args.selected_items();
    let note = normalize_optional(args.note);

    if selected.is_empty() {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }

    // Un signalement par produit, et non un pour le lot : l'hôte coche « Papier toilette » sans
    // clore « Café », et chaque e-mail garde le lien vers son propre signalement.
    for item_id in selected {
        let item = storage::find_item(item_id)?
            .ok_or_else(|| PortakiError::Host("item_not_found".to_string()))?;
        let item_label = labels::pick_label(
            &labels::labels_from_item(&item),
            &ctx.locale,
            &ctx.property.locale,
        );
        if item_label.trim().is_empty() {
            return Err(PortakiError::Host("item_label_empty".to_string()));
        }

        let report = storage::create_report(
            stay_id,
            item.id,
            item_label.clone(),
            level.clone(),
            note.clone(),
        )?;

        // The report is saved: a refused email is logged, it does not fail the guest's submit.
        if let Err(error) = email_send::notify_host_submitted(
            ctx.property_id,
            stay_id,
            report.id,
            &item_label,
            &level,
            note.as_deref(),
        ) {
            email_text::log_send_failure("consumables_host_email_failed", &error);
        }
    }
    Ok(())
}

/// Arguments for host `updateStatus`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct UpdateStatusArgs {
    pub report_id: Uuid,
    pub status: String,
}

#[portaki_sdk::command(
    name = "updateStatus",
    example(
        label = "Réassort fait",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"restocked"}"#
    )
)]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let status = status::parse_status(&args.status)?;
    let _ = storage::update_status(args.report_id, status)?;
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn require_guest_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
