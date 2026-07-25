//! Module commands — config + submit pre-arrival form.

use portaki_sdk::host::events;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::{load_config, save_config, ShowWhen};
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

/// Arguments for `updateConfig` (flat form fields from host SDUI Save).
///
/// Question flags are `Option<bool>` so a missing key keeps the KV value.
/// A present `false` must stick — never use `default = true` (empty `{}`
/// payloads used to reset every toggle ON and look like a fake Save).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub show_when: String,
    #[serde(default)]
    pub ask_arrival_time: Option<bool>,
    #[serde(default)]
    pub ask_occasion: Option<bool>,
    #[serde(default)]
    pub ask_allergies: Option<bool>,
    #[serde(default)]
    pub ask_guest_count: Option<bool>,
    #[serde(default)]
    pub ask_special_needs: Option<bool>,
    #[serde(default)]
    pub ask_id_document: Option<bool>,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let mut config = load_config().unwrap_or_default();

    if !args.show_when.trim().is_empty() {
        config.show_when = ShowWhen::parse(&args.show_when);
    }

    apply_flag(
        &mut config.questions.ask_arrival_time,
        args.ask_arrival_time,
    );
    apply_flag(&mut config.questions.ask_occasion, args.ask_occasion);
    apply_flag(&mut config.questions.ask_allergies, args.ask_allergies);
    apply_flag(&mut config.questions.ask_guest_count, args.ask_guest_count);
    apply_flag(
        &mut config.questions.ask_special_needs,
        args.ask_special_needs,
    );
    apply_flag(&mut config.questions.ask_id_document, args.ask_id_document);

    save_config(&config)
}

fn apply_flag(target: &mut bool, value: Option<bool>) {
    if let Some(next) = value {
        *target = next;
    }
}

/// Arguments for `submit`.
#[portaki_sdk::wire]
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
#[portaki_sdk::command(name = "sendFormAvailable")]
pub fn send_form_available(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_form_available(&ctx)
}

#[portaki_sdk::command(name = "submit")]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now().unwrap_or_else(|_| chrono::Utc::now());
    if !is_editable_until_checkin(now, checkin_at) {
        return Err(PortakiError::Host("form_locked_after_checkin".to_string()));
    }

    let config = load_config().unwrap_or_default();
    let q = &config.questions;

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
