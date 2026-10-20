//! Ce que partagent les coches du voyageur : le séjour, l'étape permise, la progression émise.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::storage;

/// A guest ticks only the lists written for guests, never the host's.
pub(super) fn require_guest_item(item_id: Uuid) -> Result<()> {
    if crate::domain::guest_items()?
        .iter()
        .any(|item| item.uuid() == item_id)
    {
        Ok(())
    } else {
        Err(PortakiError::Host("not_guest_item".to_string()))
    }
}

pub(super) fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

#[portaki_sdk::wire(serialize)]
struct ProgressPayload {
    property_id: Uuid,
    percentage: u8,
}

#[portaki_sdk::wire(serialize)]
struct CompletedPayload {
    stay_id: Uuid,
}

/// Progress of the stay over every guest item.
pub(super) fn emit_progress(property_id: Uuid, stay_id: Uuid) -> Result<()> {
    let guest_items = crate::domain::guest_items()?;
    let completed = storage::list_completions(Some(stay_id))?;
    let done = guest_items
        .iter()
        .filter(|item| completed.iter().any(|row| row.item_id == item.uuid()))
        .count();
    let percentage = ((done * 100).checked_div(guest_items.len()).unwrap_or(0)) as u8;
    events::emit(
        crate::ids::PROGRESS_UPDATED,
        &ProgressPayload {
            property_id,
            percentage,
        },
    )?;
    if percentage == 100 {
        events::emit(crate::ids::COMPLETED, &CompletedPayload { stay_id })?;
    }
    Ok(())
}
