//! Load checklist data for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{Checklist, ChecklistItem};
use crate::lists;
use crate::show_when::is_checklist_available;
use crate::storage;

pub enum GuestLoad {
    NoItems,
    NotYet,
    Ready(GuestChecklistData),
}

/// Ce que le livret a besoin de savoir pour rendre la liste.
///
/// Plus de compte ni de pourcentage : la barre de progression et le « n / N » sont dessinés par le
/// livret à partir de ce qui est coché, et les calculer ici revenait à décider deux fois.
pub struct GuestChecklistData {
    /// Guest lists open right now, each with its items.
    pub lists: Vec<(Checklist, Vec<ChecklistItem>)>,
    pub completed: Vec<Uuid>,
    pub locale: String,
    pub property_locale: String,
    /// Departure instant (UTC) — rendered as the card title in the property timezone.
    pub checkout_at: Option<chrono::DateTime<chrono::Utc>>,
    pub property_timezone: String,
    /// « Étapes visibles » and « Message final » of the first open list.
    pub display: storage::display::Display,
}

/// `departure_only` keeps the `atDeparture` lists (post-stay card).
pub fn load_guest_checklist(ctx: &GuestContext, departure_only: bool) -> Result<GuestLoad> {
    let mut guest_lists = Vec::new();
    for list in storage::list_checklists()? {
        let wanted = list.audience == lists::GUEST
            && (!departure_only || list.trigger == lists::AT_DEPARTURE);
        if !wanted {
            continue;
        }
        let items = storage::items_of(list.id)?;
        if !items.is_empty() {
            guest_lists.push((list, items));
        }
    }
    if guest_lists.is_empty() {
        return Ok(GuestLoad::NoItems);
    }

    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let now = time::now()?;
    guest_lists
        .retain(|(list, _)| is_checklist_available(&list.trigger, now, checkin_at, checkout_at));
    if guest_lists.is_empty() {
        return Ok(GuestLoad::NotYet);
    }

    let stay_id = ctx.guest.as_ref().map(|guest| guest.session_id);
    let completed: Vec<Uuid> = match stay_id {
        Some(id) => storage::list_completions(Some(id))?
            .into_iter()
            .map(|row| row.item_id)
            .collect(),
        None => Vec::new(),
    };
    // ponytail: the card is one list for every open guest list, so it follows the first one's
    // display settings — the spec shows a single « Voyageur · Départ » list at a time. Per-list
    // folding needs a `ChoiceList` per list.
    let display = storage::display::read(guest_lists[0].0.id);
    Ok(GuestLoad::Ready(GuestChecklistData {
        lists: guest_lists,
        completed,
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        checkout_at,
        property_timezone: ctx.property.timezone.clone(),
        display,
    }))
}
