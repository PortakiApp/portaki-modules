//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stay-scoped guest shortage report (many per stay).
///
/// `property_id` is injected by typed-repo from invocation context; kept in the
/// schema so SELECT/INSERT/DELETE with `WHERE property_id = …` succeed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 5)]
pub struct ConsumableReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    /// Label snapshot at submit time (catalog may change later).
    pub item_label: String,
    /// Wire: `missing` | `low`.
    pub level: String,
    pub note: Option<String>,
    /// Wire: `open` (à traiter) | `planned` (prévu) | `restocked` (livré) — default `open`.
    #[serde(default = "default_status")]
    pub status: String,
    pub created_at: DateTime<Utc>,
    /// Set when the host marks it restocked.
    #[serde(default)]
    pub restocked_at: Option<DateTime<Utc>>,
    /// La réponse de l'hôte, que le voyageur lit sous sa demande.
    #[serde(default)]
    pub host_reply: Option<String>,
    /// Le nom du voyageur à l'envoi, pour la colonne « Voyageur ». Vide quand il est inconnu.
    #[serde(default)]
    pub guest_name: String,
    /// Les dates du séjour à l'envoi, pour la colonne « Séjour ».
    #[serde(default)]
    pub stay_check_in: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stay_check_out: Option<DateTime<Utc>>,
}

fn default_status() -> String {
    crate::status::DEFAULT.to_string()
}

#[portaki_sdk::entity_indexes(ConsumableReport)]
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const CONSUMABLE_REPORT_INDEXES: &[&str] = &["stay_id", "status"];
