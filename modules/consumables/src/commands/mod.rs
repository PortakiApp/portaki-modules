//! Les opérations : le catalogue de départ, la demande du voyageur, la réponse de l'hôte.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use uuid::Uuid;

pub mod amenities;
pub mod queries;
pub mod tasks;

use crate::{email_send, email_text, level, status, storage};
use crate::{Product, Products};

#[portaki_sdk::command(name = "seedDefaults", example(label = "Catalogue par défaut"))]
pub fn seed_defaults(ctx: Context, _args: EmptyArgs) -> Result<()> {
    let mut products = Products::load(&ctx)?;
    if !products.is_empty() {
        return Ok(());
    }
    // La liste de départ de la spec (§4), chacun avec son emoji : la grille du voyageur se lit
    // alors d'un coup d'œil.
    let starter = [
        ("Café", "Coffee", "☕"),
        ("Thé", "Tea", "🍵"),
        ("Papier toilette", "Toilet paper", "🧻"),
        ("Liquide vaisselle", "Dish soap", "🧽"),
        ("Sel", "Salt", "🧂"),
        ("Huile", "Oil", "🫒"),
    ];
    products
        .rows_mut()
        .extend(starter.into_iter().map(|(fr, en, emoji)| Product {
            id: Uuid::new_v4().to_string(),
            label: I18nText::new(fr, en),
            emoji: emoji.into(),
            ..Product::default()
        }));
    Products::save(&products)
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
    let products = Products::load(&ctx)?;
    let settings = products.meta();
    if !settings.enabled() {
        return Err(PortakiError::Host("requests_disabled".to_string()));
    }
    if storage::list_by_stay(stay_id)?.len() >= settings.max() as usize {
        return Err(PortakiError::Host("requests_limit".to_string()));
    }
    // Les produits d'abord : `note` consomme `args`, et la liste se lit encore par référence.
    let selected = args.selected_items();
    let asked = storage::Asked {
        level,
        note: normalize_optional(args.note),
    };

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
        let item = products
            .rows()
            .iter()
            .find(|product| product.uuid() == item_id)
            .ok_or_else(|| PortakiError::Host("item_not_found".to_string()))?;
        let item_label = item.label.for_ctx(&ctx).trim().to_string();
        if item_label.trim().is_empty() {
            return Err(PortakiError::Host("item_label_empty".to_string()));
        }

        let report = storage::create_report(stay_id, item_id, item_label, &asked, &guest)?;

        // ponytail: sent at once, day or night — the platform has no account quiet hours to
        // group a 2 am request into a wake-up notification (spec §9 n° 6).
        if !settings.notifies_by_email() {
            continue;
        }
        // The report is saved: a refused email is logged, it does not fail the guest's submit.
        if let Err(error) = email_send::notify_host_submitted(ctx.property_id, &report) {
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
