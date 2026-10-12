//! Module queries — pre-arrival form status.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::ModuleConfig;
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

#[portaki_sdk::query(name = "getStatus", example(label = "Statut du formulaire"))]
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

/// Ce que `problems` refuse bloque la publication, sous le champ concerné. L'heure d'arrivée est
/// toujours demandée (§9 cas 4) : plus de « posez au moins une question » à recommander.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let items = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            label: text(match field.as_str() {
                "slots_until" => "host.slots.until",
                "late_from" => "host.lateFrom",
                _ => "host.custom.title",
            }),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();
    Ok(PublishReadiness { items })
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
