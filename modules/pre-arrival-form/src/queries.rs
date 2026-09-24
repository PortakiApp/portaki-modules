//! Module queries — pre-arrival form status.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::load_config;
use crate::i18n::text;
use crate::storage;

/// Status DTO returned by `getStatus`.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct PreArrivalStatus {
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrival_time_estimated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_occasion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_allergies: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_needs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_to_host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl PreArrivalStatus {
    fn incomplete() -> Self {
        Self {
            completed: false,
            arrival_time_estimated: None,
            guest_occasion: None,
            guest_allergies: None,
            guest_count: None,
            special_needs: None,
            id_document: None,
            message_to_host: None,
            completed_at: None,
        }
    }
}

#[portaki_sdk::query(name = "getStatus")]
pub fn get_status(ctx: Context) -> Result<PreArrivalStatus> {
    let stay_id = require_stay_id(&ctx)?;
    let Some(row) = storage::find_by_stay(stay_id)? else {
        return Ok(PreArrivalStatus::incomplete());
    };
    Ok(PreArrivalStatus {
        completed: true,
        arrival_time_estimated: row.arrival_time,
        guest_occasion: row.occasion,
        guest_allergies: row.allergies,
        guest_count: row.guest_count,
        special_needs: row.special_needs,
        id_document: row.id_document,
        message_to_host: row.guest_message,
        completed_at: Some(row.completed_at),
    })
}

/// Recommends asking the guest at least one question; never blocks.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let q = load_config()?.questions;
    let ok = q.ask_arrival_time
        || q.ask_occasion
        || q.ask_allergies
        || q.ask_guest_count
        || q.ask_special_needs
        || q.ask_id_document;
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "questions".into(),
            level: PublishLevel::Recommended,
            ok,
            label: text("publish.questions.label"),
            hint: text("publish.questions.hint"),
        }],
    })
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
