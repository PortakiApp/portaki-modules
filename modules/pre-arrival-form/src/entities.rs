//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One pre-arrival form response per stay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct PreArrivalResponse {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub arrival_time: Option<String>,
    pub occasion: Option<String>,
    pub allergies: Option<String>,
    pub guest_count: Option<String>,
    pub special_needs: Option<String>,
    pub id_document: Option<String>,
    pub guest_message: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(PreArrivalResponse)]
#[allow(dead_code)]
pub const PRE_ARRIVAL_RESPONSE_INDEXES: &[&str] = &["stay_id"];
