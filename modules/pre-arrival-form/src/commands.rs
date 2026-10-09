//! Module commands — submit pre-arrival form.

use portaki_sdk::host::events;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::{Deadline, ModuleConfig};
use crate::email_send;
use crate::show_when::is_editable_until_checkin;
use crate::storage;

/// What the platform copies onto the stay — and nothing more: the ID document, special needs and
/// party size stay in the module's own table, out of the outbox.
#[portaki_sdk::wire(serialize)]
struct CompletedPayload {
    arrival_time_estimated: Option<String>,
    guest_occasion: Option<String>,
    guest_allergies: Option<String>,
    message_to_host: Option<String>,
    /// Le moyen d'arrivée, pour que le séjour le garde et que le livret mette Trains en avant la
    /// veille (§2.20). La plateforme n'en retient que les valeurs qu'elle sait lire.
    guest_transport: Option<String>,
}

/// Arguments for `submit`.
/// `Default` pour que les tests ne nomment que les champs du cas qu'ils décrivent : les
/// questions sont facultatives, et les épeler en `None` partout rendrait chaque cas illisible.
#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct SubmitArgs {
    pub arrival_time_estimated: Option<String>,
    pub guest_occasion: Option<String>,
    pub guest_allergies: Option<String>,
    pub guest_count: Option<String>,
    pub special_needs: Option<String>,
    pub id_document: Option<String>,
    /// Le moyen de transport choisi — `car`, `train`, `plane`, `other`.
    #[serde(default)]
    pub transport: Option<String>,
    pub message_to_host: Option<String>,
}

/// Scheduling tick / stay-created — module owns availability gate + email content.
#[portaki_sdk::email(
    id = "form-available",
    audience = EmailAudience::Guest,
    dispatch_on_stay_created,
    catch_up_on_property_publish,
    catch_up_on_config_update,
    requires_guest_email,
    description_key = "email.form-available.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendFormAvailable",
    example(label = "E-mail « formulaire disponible »")
)]
pub fn send_form_available(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_form_available(&ctx)
}

/// Relance « 24 h avant la limite » (§2.3), au passage quotidien de la plateforme : limite au
/// soir de J-3 → passage de J-4. Chaque limite a son e-mail ; seule celle choisie part.
#[portaki_sdk::email(
    id = "reminder-j-3",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckIn,
    offset_days = -4,
    requires_guest_email,
    description_key = "email.reminder-j-3.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(name = "sendReminderJ3", example(label = "Relance, limite à J-3"))]
pub fn send_reminder_j3(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_reminder(&ctx, Deadline::J3, "reminder-j-3")
}

/// Limite la veille à 18 h → passage de J-2.
#[portaki_sdk::email(
    id = "reminder-j-1-18h",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckIn,
    offset_days = -2,
    requires_guest_email,
    description_key = "email.reminder-j-1-18h.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendReminderJ1",
    example(label = "Relance, limite la veille à 18 h")
)]
pub fn send_reminder_j1(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_reminder(&ctx, Deadline::J1At18, "reminder-j-1-18h")
}

/// Limite le jour même à midi → passage de J-1.
#[portaki_sdk::email(
    id = "reminder-j0-12h",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckIn,
    offset_days = -1,
    requires_guest_email,
    description_key = "email.reminder-j0-12h.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendReminderJ0",
    example(label = "Relance, limite le jour même à midi")
)]
pub fn send_reminder_j0(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_reminder(&ctx, Deadline::J0At12, "reminder-j0-12h")
}

#[portaki_sdk::command(
    name = "submit",
    guest,
    example(
        label = "Arrivée en fin d'après-midi",
        input = r#"{"arrivalTimeEstimated":"17:30","guestOccasion":"Anniversaire","guestCount":"2","messageToHost":"Nous arriverons en voiture."}"#
    ),
    example(
        label = "Allergie signalée",
        input = r#"{"arrivalTimeEstimated":"21:00","guestAllergies":"Arachides","guestCount":"4"}"#
    )
)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now()?;
    if !is_editable_until_checkin(now, checkin_at) {
        return Err(PortakiError::Host("form_locked_after_checkin".to_string()));
    }

    let q = ModuleConfig::load(&ctx)?;

    let arrival_time = if q.ask_arrival_time {
        normalize(args.arrival_time_estimated)
    } else {
        None
    };
    let occasion = if q.ask_occasion {
        normalize(args.guest_occasion)
    } else {
        None
    };
    let allergies = if q.ask_allergies {
        normalize(args.guest_allergies)
    } else {
        None
    };
    let guest_count = if q.ask_guest_count {
        normalize(args.guest_count)
    } else {
        None
    };
    let special_needs = if q.ask_special_needs {
        normalize(args.special_needs)
    } else {
        None
    };
    let id_document = if q.ask_id_document {
        normalize(args.id_document)
    } else {
        None
    };
    // Et seulement un moyen de la liste : une valeur hors liste ne dit rien à l'hôte, et un
    // formulaire resté ouvert dans un téléphone peut en porter une.
    let transport = if q.ask_transport {
        normalize(args.transport).filter(|value| crate::TRANSPORTS.contains(&value.as_str()))
    } else {
        None
    };
    let message = normalize(args.message_to_host);

    let _ = storage::upsert(
        stay_id,
        storage::ResponseDraft {
            arrival_time: arrival_time.clone(),
            occasion: occasion.clone(),
            allergies: allergies.clone(),
            guest_count,
            special_needs,
            id_document,
            transport: transport.clone(),
            guest_message: message.clone(),
        },
    )?;

    events::emit(
        crate::ids::COMPLETED,
        &CompletedPayload {
            arrival_time_estimated: arrival_time,
            guest_occasion: occasion,
            guest_allergies: allergies,
            message_to_host: message,
            guest_transport: transport,
        },
    )?;
    Ok(())
}

fn normalize(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The keys `pre-arrival.completed` carries are the ones the platform reads.
    #[test]
    fn completed_event_carries_only_what_the_platform_reads() {
        let some = || Some("x".to_string());
        let payload = serde_json::to_value(CompletedPayload {
            arrival_time_estimated: some(),
            guest_occasion: some(),
            guest_allergies: some(),
            message_to_host: some(),
            guest_transport: some(),
        })
        .expect("json");
        let mut keys: Vec<&str> = payload
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "arrivalTimeEstimated",
                "guestAllergies",
                "guestOccasion",
                "guestTransport",
                "messageToHost"
            ]
        );
    }
}
