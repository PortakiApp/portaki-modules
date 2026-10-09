//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One lost/found report for a stay (many per stay allowed).
///
/// `item_description` may be plain text (guest) or TipTap JSON (host-found).
/// `status` tracks host workflow — see [`crate::status`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct LostFoundReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub kind: String,
    pub item_description: String,
    pub contact_hint: Option<String>,
    pub details: Option<String>,
    /// L'adresse de renvoi, quand l'hôte propose le renvoi postal et que le voyageur l'a écrite.
    #[serde(default)]
    pub return_address: Option<String>,
    /// La catégorie d'objet que le voyageur a touchée — `phone`, `clothing`, … `other`.
    /// Absente des signalements d'avant la grille : une fiche ancienne reste lisible.
    #[serde(default)]
    pub category: Option<String>,
    /// La pièce où l'objet a été laissé, `unknown` quand le voyageur ne sait pas.
    #[serde(default)]
    pub room: Option<String>,
    /// La photo jointe, en référence `portaki-file:` ; `None` quand il n'y en a pas.
    #[serde(default)]
    pub photo: Option<String>,
    /// Ce que le voyageur voudrait qu'on en fasse — `ship`, `pickup`, `donate`.
    #[serde(default)]
    pub return_choice: Option<String>,
    /// One of [`crate::status::WIRE_VALUES`]. Rows from before the six statuses still hold
    /// `to_collect` | `sent` | `returned`: storage reads them through [`crate::status::normalize`].
    #[serde(default)]
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(LostFoundReport)]
#[allow(dead_code)]
pub const LOST_FOUND_REPORT_INDEXES: &[&str] = &["stay_id"];
