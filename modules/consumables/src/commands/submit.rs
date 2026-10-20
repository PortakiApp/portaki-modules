//! Ce que le module ajoute aux demandes générées par `#[record]` : un envoi demande plusieurs
//! produits d'un coup, une demande n'est une tâche que tant qu'elle est à traiter, et les e-mails.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::record::item_from;
use portaki_sdk::resource::RecordHooks;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{email, Entry, Products, Request, Restocks, Status};

/// Ce que le formulaire du voyageur envoie.
#[portaki_sdk::wire]
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

impl RecordHooks for Restocks {
    type Item = Request;
    type Status = Status;

    /// Une demande par produit, et non une pour le lot : l'hôte coche « Papier toilette » sans
    /// clore « Café ». Demandes fermées, ou plafond du séjour atteint (compté une fois, avant le
    /// lot) : un écran resté ouvert ne passe pas non plus.
    fn deposits(ctx: &Context, args: &Value) -> Result<Vec<Request>> {
        let args: SubmitArgs = serde_json::from_value(args.clone())?;
        let products = Products::load(ctx)?;
        let settings = products.meta();
        if !settings.enabled() {
            return Err(PortakiError::Host("requests_disabled".into()));
        }
        let stay = ctx.guest.as_ref().map(|g| g.session_id).unwrap_or_default();
        if Restocks::for_stay(ctx, stay)?.len() >= settings.max() as usize {
            return Err(PortakiError::Host("requests_limit".into()));
        }
        let selected = args.selected_items();
        if selected.is_empty() {
            return Err(PortakiError::Host("item_not_found".into()));
        }
        let note = args.note.as_deref().map(str::trim).unwrap_or_default();
        selected
            .into_iter()
            .map(|id| {
                let product = products
                    .rows()
                    .iter()
                    .find(|p| p.uuid() == id)
                    .ok_or_else(|| PortakiError::Host("item_not_found".into()))?;
                let label = product.label.for_ctx(ctx).trim();
                if label.is_empty() {
                    return Err(PortakiError::Host("item_label_empty".into()));
                }
                let row = json!({ "item_id": id, "item_label": label, "level": args.level.trim(), "note": note });
                item_from(&row, &ctx.lang())
            })
            .collect()
    }

    /// « Prévu » n'est plus une tâche : l'hôte l'a vue. Demandes fermées : pas de tâche (§3).
    fn needs_action(ctx: &Context, entry: &Entry) -> bool {
        entry.status == Status::Open && Products::load(ctx).is_ok_and(|p| p.meta().enabled())
    }

    /// « Café demandé », puis le voyageur.
    fn task_context(entry: &Entry) -> I18nText {
        I18nText::from(entry.guest_name.as_str())
    }

    // ponytail: sent at once, day or night — the platform has no account quiet hours to group a
    // 2 am request into a wake-up notification (spec §9 n° 6).
    fn on_created(ctx: &Context, entry: &Entry) -> Result<()> {
        if !Products::load(ctx)?.meta().notifies_by_email() {
            return Ok(());
        }
        email::to_host::to_host(ctx.property_id, entry)
    }

    fn on_note(_: &Context, entry: &Entry, text: &str) -> Result<()> {
        email::to_guest::to_guest(entry, text)
    }
}
