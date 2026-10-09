//! Module queries — stay reports, host recent list, email context.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::storage;

/// Row returned by list queries.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct LostFoundReportRow {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub kind: String,
    pub item_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// L'adresse de renvoi écrite par le voyageur — l'hôte en a besoin pour expédier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_address: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<crate::entities::LostFoundReport> for LostFoundReportRow {
    fn from(row: crate::entities::LostFoundReport) -> Self {
        Self {
            id: row.id,
            stay_id: row.stay_id,
            kind: row.kind,
            item_description: row.item_description,
            contact_hint: row.contact_hint,
            details: row.details,
            return_address: row.return_address,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

/// Optional host override — guest sessions ignore and use the guest stay id.
#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct ListForStayArgs {
    #[serde(default)]
    pub stay_id: Option<Uuid>,
}

#[portaki_sdk::query(
    name = "listForStay",
    example(
        label = "Signalements d'un séjour",
        input = r#"{"stayId":"5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54"}"#
    )
)]
pub fn list_for_stay(ctx: Context, args: ListForStayArgs) -> Result<Vec<LostFoundReportRow>> {
    let stay_id = resolve_list_stay_id(&ctx, args.stay_id)?;
    Ok(storage::list_by_stay(stay_id)?
        .into_iter()
        .map(LostFoundReportRow::from)
        .collect())
}

#[portaki_sdk::query(name = "listRecent", example(label = "Derniers signalements"))]
pub fn list_recent(_ctx: Context) -> Result<Vec<LostFoundReportRow>> {
    Ok(storage::list_recent()?
        .into_iter()
        .map(LostFoundReportRow::from)
        .collect())
}

fn resolve_list_stay_id(ctx: &Context, stay_id: Option<Uuid>) -> Result<Uuid> {
    if let Some(guest) = ctx.guest.as_ref() {
        return Ok(guest.session_id);
    }
    stay_id.ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

/// Ce qui bloque la publication : les mêmes messages que sous les champs du tiroir
/// ([`crate::config::ModuleConfig::problems`]), désignant chacun son champ (`config.<clé>`).
///
/// « Au moins une option de restitution » n'y est pas : aucune cochée vaut renvoi et
/// récupération (`return_options`), le voyageur a toujours un choix.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = crate::config::ModuleConfig::load(&ctx)?;
    let items = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            label: crate::i18n::text(field_label(field), &[]),
            hint: error,
        })
        .collect();
    Ok(PublishReadiness { items })
}

fn field_label(field: &str) -> &'static str {
    match field {
        "window_days" => "host.window.label",
        "keep_days" => "host.keep.label",
        "pickup_note" => "host.pickupNote.label",
        _ => "host.donateOrg.label",
    }
}
