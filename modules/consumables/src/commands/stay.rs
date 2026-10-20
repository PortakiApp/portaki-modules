//! Le séjour d'une opération : celui du voyageur, ou celui que l'hôte désigne.

use portaki_sdk::prelude::*;
use uuid::Uuid;

pub(super) fn require_guest_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

pub(super) fn resolve_list_stay_id(ctx: &Context, stay_id: Option<Uuid>) -> Result<Uuid> {
    if let Some(guest) = ctx.guest.as_ref() {
        return Ok(guest.session_id);
    }
    stay_id.ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
