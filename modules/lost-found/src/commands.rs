//! Module commands — guest submit, host submitFound / updateStatus.

use portaki_sdk::files::FileRef;
use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::description;
use crate::email_send;
use crate::email_text;
use crate::kind;
use crate::status;
use crate::storage;

/// Arguments for guest `submit`.
///
/// `Default` pour que les tests n'aient à nommer que les champs du cas qu'ils décrivent : la
/// grille d'objets, la pièce, la photo et le choix de restitution sont facultatifs, et les
/// épeler en `None` partout rendrait chaque cas illisible.
#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct SubmitArgs {
    pub kind: String,
    pub item_description: String,
    #[serde(default)]
    pub contact_hint: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    /// L'adresse de renvoi — présente seulement quand le formulaire l'a demandée.
    #[serde(default)]
    pub return_address: Option<String>,
    /// La catégorie touchée dans la grille — `phone`, `clothing`, … `other`.
    #[serde(default)]
    pub category: Option<String>,
    /// La pièce, ou `unknown`.
    #[serde(default)]
    pub room: Option<String>,
    /// La photo jointe : la valeur d'`ImageUpload`, `portaki-file:<uuid>`.
    #[serde(default)]
    pub photo: Option<String>,
    /// Ce que le voyageur voudrait qu'on en fasse, parmi ce que l'hôte propose.
    #[serde(default)]
    pub return_choice: Option<String>,
}

#[portaki_sdk::command(
    name = "submit",
    guest,
    example(
        label = "Chargeur oublié",
        input = r#"{"kind":"lost","itemDescription":"Chargeur de téléphone blanc","details":"Sans doute branché près du lit de la chambre 2."}"#
    ),
    example(
        label = "Objet trouvé sur place",
        input = r#"{"kind":"found","itemDescription":"Boucle d'oreille dorée","contactHint":"Posée sur la table de l'entrée"}"#
    )
)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_guest_stay_id(&ctx)?;
    let kind = kind::parse_kind(&args.kind)?;
    let item_description = require_description(&args.item_description)?;
    let contact_hint = normalize_optional(args.contact_hint);
    let details = normalize_optional(args.details);
    let config = crate::config::ModuleConfig::load(&ctx)?;

    let room = normalize_optional(args.room.clone());
    let return_choice = normalize_optional(args.return_choice.clone())
        .filter(|choice| config.return_options().contains(&choice.as_str()));
    let photo =
        normalize_optional(args.photo.clone()).filter(|value| FileRef::parse(value).is_some());

    let report = storage::create(storage::ReportDraft {
        stay_id,
        kind: kind.clone(),
        item_description: item_description.clone(),
        contact_hint: contact_hint.clone(),
        details: details.clone(),
        // Gardée seulement si l'hôte propose le renvoi : un formulaire laissé ouvert dans un
        // téléphone avant que l'hôte change d'avis ne doit pas faire entrer une adresse.
        return_address: config
            .offers_shipping()
            .then(|| normalize_optional(args.return_address.clone()))
            .flatten(),
        category: normalize_optional(args.category.clone()),
        // La pièce, la photo et le souhait sont résolus au-dessus : l'e-mail les reprend, et
        // les recalculer donnerait deux vérités pour un seul signalement.
        room: room.clone(),
        photo: photo.clone(),
        return_choice: return_choice.clone(),
        status: status::DEFAULT.to_string(),
    })?;

    // The report is saved: a refused email is logged, it does not fail the guest's submit.
    if let Err(error) = email_send::notify_host_submitted(ctx.property_id, &report) {
        email_text::log_send_failure("lost_found_host_email_failed", &error);
    }
    Ok(())
}

/// Arguments for host `submitFound` — one report per stay (shared description/status).
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitFoundArgs {
    /// Target stays (property-scoped). Empty when only [`Self::stay_id`] is set.
    #[serde(default)]
    pub stay_ids: Vec<Uuid>,
    /// Convenience single stay (merged into `stay_ids`).
    #[serde(default)]
    pub stay_id: Option<Uuid>,
    /// TipTap JSON or plain text — stored as-is; email gets plain extract.
    pub description: String,
    /// Ignored: an item the host declares is always « Trouvé ».
    #[serde(default)]
    pub status: Option<String>,
}

/// Each stay's guest hears of the found item (`host-found-<report id>`).
#[portaki_sdk::email(
    id = "host-found",
    audience = EmailAudience::Guest,
    requires_guest_email,
    description_key = "email.host-found.description"
)]
#[portaki_sdk::command(
    name = "submitFound",
    example(
        label = "Lunettes retrouvées",
        input = r#"{"stayId":"5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54","description":"Lunettes de soleil retrouvées sur la terrasse"}"#
    )
)]
pub fn submit_found(ctx: Context, args: SubmitFoundArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let stay_ids = resolve_stay_ids(&args)?;
    let description = require_description(&args.description)?;
    // The host has the item in hand: « Trouvé ». Status edits use `updateStatus`.
    let _ = args.status;
    let status = status::FOUND.to_string();
    let plain = description::to_plain_text(&description);
    if plain.is_empty() {
        return Err(PortakiError::Host("description_required".to_string()));
    }

    for stay_id in stay_ids {
        let report = storage::create(storage::ReportDraft {
            stay_id,
            kind: "found".to_string(),
            item_description: description.clone(),
            status: status.clone(),
            ..storage::ReportDraft::default()
        })?;

        // The report is saved: a refused email is logged, the other stays still get theirs.
        if let Err(error) =
            email_send::notify_guest_host_found(ctx.property_id, stay_id, report.id, &plain)
        {
            email_text::log_send_failure("lost_found_guest_email_failed", &error);
        }
    }

    Ok(())
}

/// Scheduled J+2 tick — module owns gate + guest email content.
#[portaki_sdk::email(
    id = "checkout-j2",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckOut,
    offset_days = 2,
    requires_guest_email,
    description_key = "email.checkout-j2.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendCheckoutFollowUp",
    example(label = "Relance deux jours après le départ")
)]
pub fn send_checkout_follow_up(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_checkout_follow_up(&ctx)
}

/// Arguments for host `updateStatus` — change workflow status after create.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct UpdateStatusArgs {
    pub report_id: Uuid,
    pub status: String,
}

#[portaki_sdk::command(
    name = "updateStatus",
    example(
        label = "Objet renvoyé",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"shipped"}"#
    )
)]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    set_status(&ctx, args.report_id, status::parse_status(&args.status)?)
}

/// Moves a report to `to` and writes the workspace journal line (`updateStatus`, task).
pub(crate) fn set_status(ctx: &Context, report_id: Uuid, to: &str) -> Result<()> {
    let report = storage::update_status(report_id, to)?;
    // Activity log of the workspace (journal).
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &serde_json::json!({
            "eventId": "lost-found.status-updated",
            "propertyId": ctx.property_id,
            "payload": { "reportId": report.id, "stayId": report.stay_id, "status": report.status },
            "display": { "chips": [{
                "label": "Objet",
                "value": description::to_plain_text(&report.item_description),
            }] },
        }),
    )
}

fn resolve_stay_ids(args: &SubmitFoundArgs) -> Result<Vec<Uuid>> {
    let mut ids = args.stay_ids.clone();
    if let Some(stay_id) = args.stay_id {
        if !ids.contains(&stay_id) {
            ids.push(stay_id);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err(PortakiError::Host("stay_ids_required".to_string()));
    }
    Ok(ids)
}

fn require_description(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || description::to_plain_text(trimmed).is_empty() {
        return Err(PortakiError::Host("description_required".to_string()));
    }
    Ok(trimmed.to_string())
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
