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
    /// La ligne du tableau. Une ligne jamais ouverte n'envoie que lui : le produit garde alors
    /// tout ce qu'il avait. Sans id (API, ancien formulaire), la ligne se lit à son rang.
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Le nom, dans la langue de l'hôte. Absent : inchangé ; vide sur une ligne à id : effacé.
    #[serde(default)]
    pub label: Option<String>,
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
    /// « Où le trouver », dans la langue de l'hôte. Absent : inchangé ; vide : effacé.
    #[serde(default)]
    pub location: Option<String>,
}

impl ConsumableItemInput {
    fn typed_label(&self) -> Option<&str> {
        self.label.as_deref().map(str::trim)
    }

    /// Rien de nommé : ni nom dans la langue de l'hôte, ni nom par langue (API).
    fn unnamed(&self) -> bool {
        self.typed_label().unwrap_or_default().is_empty()
            && self.label_fr.trim().is_empty()
            && self.label_en.trim().is_empty()
    }
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
                // Sans id, une ligne sans nom est une case laissée vide : elle tombe. Avec id,
                // c'est une ligne du tableau, que `replace_items` lit contre l'enregistré.
                if item.id.is_none() && item.unnamed() {
                    return None;
                }
                Some(ConsumableItemInput {
                    id: item.id,
                    location: item.location.clone(),
                    emoji: item.emoji.trim().to_string(),
                    label: item.typed_label().map(str::to_string),
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
    ///
    /// Absent quand le champ est masqué (demandes fermées) : le délai enregistré reste. Envoyé
    /// vide, il s'efface.
    #[serde(default, alias = "restockDelay")]
    pub restock_delay: Option<I18nText>,
    /// « Le voyageur peut demander » : un booléen, ou `"true"` / `"false"` selon le formulaire.
    #[serde(default, alias = "requestsEnabled")]
    pub requests_enabled: Option<serde_json::Value>,
    /// « Demandes par séjour » : un nombre, le `NumberInput` n'envoie pas d'entier.
    #[serde(default, alias = "maxRequests")]
    pub max_requests: Option<f64>,
    /// « Prévenir par » : un tableau, ou du JSON / une liste à virgules d'un ancien formulaire.
    #[serde(default)]
    pub notify: Option<serde_json::Value>,
}

/// Les canaux cochés, quelle que soit la forme reçue ; `None` quand le formulaire n'en dit rien.
fn channels(value: &serde_json::Value) -> Option<Vec<String>> {
    let list: Vec<String> = match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        serde_json::Value::String(text) => serde_json::from_str::<Vec<String>>(text)
            .unwrap_or_else(|_| text.split(',').map(str::to_string).collect()),
        _ => return None,
    };
    let mut out: Vec<String> = Vec::new();
    for channel in list.iter().map(|c| c.trim()) {
        if [storage::settings::EMAIL, storage::settings::APP].contains(&channel)
            && !out.iter().any(|known| known == channel)
        {
            out.push(channel.to_string());
        }
    }
    Some(out)
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
    if let Some(delay) = &args.restock_delay {
        storage::restock_delay::write(delay)?;
    }
    let mut settings = storage::settings::read();
    if let Some(enabled) = args
        .requests_enabled
        .as_ref()
        .and_then(|value| match value {
            serde_json::Value::Bool(on) => Some(*on),
            serde_json::Value::String(on) => Some(on == "true"),
            _ => None,
        })
    {
        settings.requests_enabled = Some(enabled);
    }
    // Gardé tel quel, hors bornes compris : « Entre 1 et 20. » s'affiche sous le champ et bloque
    // la publication, et la lecture reste bornée pour le voyageur.
    if let Some(max) = args.max_requests.filter(|n| n.is_finite()) {
        settings.max_requests = Some(max.round().max(0.0) as u32);
    }
    // Aucun canal coché est gardé tel quel : « Choisissez au moins un canal. » bloque la
    // publication, jamais l'enregistrement.
    if let Some(notify) = args.notify.as_ref().and_then(channels) {
        settings.notify = Some(notify);
    }
    storage::settings::write(&settings)?;
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
        // La ligne du tableau par son id ; une ligne sans id, à son rang.
        let base = match input.id {
            Some(id) => existing.iter().find(|item| item.id == id),
            None => existing.get(index),
        };
        let mut labels_map = base.map(labels::labels_from_item).unwrap_or_default();
        match input.typed_label() {
            Some("") if input.id.is_some() => {
                labels_map.remove(&lang);
            }
            Some("") | None => {}
            Some(label) => {
                labels_map.insert(lang.clone(), label.to_string());
            }
        }
        if !input.label_fr.trim().is_empty() {
            labels_map.insert("fr".into(), input.label_fr.trim().to_string());
        }
        if !input.label_en.trim().is_empty() {
            labels_map.insert("en".into(), input.label_en.trim().to_string());
        }
        let mut location_map = base.map(labels::location_from_item).unwrap_or_default();
        if let Some(location) = input.location.as_deref().map(str::trim) {
            if location.is_empty() {
                location_map.remove(&lang);
            } else {
                location_map.insert(lang.clone(), location.to_string());
            }
        }
        let (label_fr, label_en) = labels::encode_labels(&labels_map);
        // L'emoji suit la ligne, et un champ vide ne l'efface pas : l'hôte qui renomme un
        // produit dans une autre langue ne doit pas perdre le pictogramme qu'il avait choisi.
        let emoji = match input.emoji.trim() {
            "" => base.map(|item| item.emoji.clone()).unwrap_or_default(),
            emoji => emoji.to_string(),
        };
        let low_threshold = match input.low_threshold {
            n if n > 0 => n,
            _ => base.map(|item| item.low_threshold).unwrap_or_default(),
        };
        next.push(storage::ItemDraft {
            id: base.map(|item| item.id).or(input.id),
            label_fr,
            label_en,
            // Le rang du tableau : déplacer une ligne la déplace dans le livret.
            sort_order: if input.id.is_some() {
                next.len() as i32
            } else {
                input.sort_order
            },
            low_threshold,
            emoji,
            location: if location_map.is_empty() {
                String::new()
            } else {
                labels::encode_labels(&location_map).0
            },
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
    // La liste de départ de la spec (§4), chacun avec son emoji : la grille du voyageur se lit
    // alors d'un coup d'œil.
    [
        ("Café", "Coffee", "☕"),
        ("Thé", "Tea", "🍵"),
        ("Papier toilette", "Toilet paper", "🧻"),
        ("Liquide vaisselle", "Dish soap", "🧽"),
        ("Sel", "Salt", "🧂"),
        ("Huile", "Oil", "🫒"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (fr, en, emoji))| ConsumableItemInput {
        id: None,
        label: None,
        label_fr: fr.into(),
        label_en: en.into(),
        sort_order: index as i32,
        low_threshold: 0,
        emoji: emoji.into(),
        location: None,
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
    // Demandes fermées, ou plafond du séjour atteint : un écran resté ouvert ne passe pas non plus.
    let settings = storage::settings::read();
    if !settings.requests_enabled() {
        return Err(PortakiError::Host("requests_disabled".to_string()));
    }
    if storage::list_by_stay(stay_id)?.len() >= settings.max_requests() as usize {
        return Err(PortakiError::Host("requests_limit".to_string()));
    }
    // Les produits d'abord : `note` consomme `args`, et la liste se lit encore par référence.
    let selected = args.selected_items();
    let note = normalize_optional(args.note);

    if selected.is_empty() {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    let guest = storage::GuestSnapshot {
        name: ctx
            .guest
            .as_ref()
            .and_then(|guest| guest.display_name.as_deref())
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        check_in: ctx.stay.as_ref().and_then(|stay| stay.checkin_at),
        check_out: ctx.stay.as_ref().and_then(|stay| stay.checkout_at),
    };

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
            &guest,
        )?;

        // ponytail: sent at once, day or night — the platform has no account quiet hours to
        // group a 2 am request into a wake-up notification (spec §9 n° 6).
        if !settings.notifies_by_email() {
            continue;
        }
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
    /// La réponse au voyageur, du champ du formulaire : vide l'efface, absente la garde.
    #[serde(default, alias = "host_reply", skip_serializing_if = "Option::is_none")]
    pub host_reply: Option<String>,
}

/// « Répondre » : la réponse de l'hôte part au voyageur (`host-reply-<uuid>`).
#[portaki_sdk::email(
    id = "host-reply",
    audience = EmailAudience::Guest,
    requires_guest_email,
    description_key = "email.host-reply.description"
)]
#[portaki_sdk::command(
    name = "updateStatus",
    example(
        label = "Réassort fait",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"restocked"}"#
    ),
    example(
        label = "Prévu, avec une réponse",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"planned","hostReply":"Je passe ce soir vers 18 h."}"#
    )
)]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let status = status::parse_status(&args.status)?;
    let before = storage::find_report(args.report_id)?
        .and_then(|report| report.host_reply)
        .unwrap_or_default();
    let report = storage::update_status(args.report_id, status, args.host_reply)?;
    // « Répondre » : une réponse nouvelle part au voyageur par e-mail. Celle déjà envoyée ne
    // repart pas quand l'hôte passe ensuite la demande « Livré ».
    if let Some(reply) = report
        .host_reply
        .as_deref()
        .filter(|reply| *reply != before)
    {
        if let Err(error) = email_send::reply_to_guest(&report, reply) {
            email_text::log_send_failure("consumables_guest_reply_email_failed", &error);
        }
    }
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
