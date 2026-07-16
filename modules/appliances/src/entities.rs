//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Per-property appliance guide content.
///
/// v2 stores a single-language JSON payload in `content_fr` (`content_en` unused / empty).
/// Legacy bilingual JSON is migrated on read via [`crate::content::AppliancesPayload::parse`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct AppliancesContent {
    pub id: Uuid,
    pub content_fr: String,
    pub content_en: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
