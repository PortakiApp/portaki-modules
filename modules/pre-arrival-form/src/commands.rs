//! Module commands — submit pre-arrival form.

use portaki_sdk::host::events;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::ModuleConfig;
use crate::email_send;
use crate::show_when::is_editable_until_checkin;
use crate::storage;

#[portaki_sdk::wire(serialize)]
struct CompletedPayload {
    arrival_time_estimated: Option<String>,
    guest_occasion: Option<String>,
    guest_allergies: Option<String>,
    guest_count: Option<String>,
    special_needs: Option<String>,
    id_document: Option<String>,
    message_to_host: Option<String>,
}

/// Arguments for `submit`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitArgs {
    pub arrival_time_estimated: Option<String>,
    pub guest_occasion: Option<String>,
    pub guest_allergies: Option<String>,
    pub guest_count: Option<String>,
    pub special_needs: Option<String>,
    pub id_document: Option<String>,
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
#[portaki_sdk::command(name = "sendFormAvailable")]
pub fn send_form_available(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_form_available(&ctx)
}

#[portaki_sdk::command(name = "submit", guest)]
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
    let message = normalize(args.message_to_host);

    let _ = storage::upsert(
        stay_id,
        arrival_time.clone(),
        occasion.clone(),
        allergies.clone(),
        guest_count.clone(),
        special_needs.clone(),
        id_document.clone(),
        message.clone(),
    )?;

    events::emit(
        crate::ids::COMPLETED,
        &CompletedPayload {
            arrival_time_estimated: arrival_time,
            guest_occasion: occasion,
            guest_allergies: allergies,
            guest_count,
            special_needs,
            id_document,
            message_to_host: message,
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
