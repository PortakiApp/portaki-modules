//! Les opérations : une liste tirée d'un modèle, et les coches du voyageur.

pub mod queries;
pub mod remind;
pub mod tasks;

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::{lists, storage};

/// Arguments for `createChecklist`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct CreateChecklistArgs {
    /// One of [`lists::TEMPLATES`].
    pub template: String,
    /// The id the editor picked for the new list, so it opens selected. Already taken = nothing
    /// to do (a second click on the same card).
    #[serde(default)]
    pub id: Option<Uuid>,
}

#[portaki_sdk::command(
    name = "createChecklist",
    example(label = "Liste de départ", input = r#"{"template":"departure"}"#),
    example(
        label = "Ménage entre deux séjours",
        input = r#"{"template":"cleaning"}"#
    )
)]
pub fn create_checklist(_ctx: Context, args: CreateChecklistArgs) -> Result<()> {
    let template = lists::template(&args.template)
        .ok_or_else(|| PortakiError::Host(format!("unknown_template:{}", args.template)))?;
    if let Some(id) = args.id {
        if crate::domain::all_lists()?
            .iter()
            .any(|list| list.uuid() == id)
        {
            return Ok(());
        }
    }
    crate::domain::add_from_template(template, args.id).map(|_| ())
}

/// Arguments for complete / uncomplete.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct ItemIdArgs {
    pub item_id: Uuid,
}

#[portaki_sdk::command(
    name = "completeItem",
    guest,
    example(
        label = "Cocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn complete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::complete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

#[portaki_sdk::command(
    name = "uncompleteItem",
    guest,
    example(
        label = "Décocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn uncomplete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::uncomplete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

/// Les étapes cochées, telles que la primitive `ChoiceList` les sérialise : une liste de valeurs
/// séparées par des virgules.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SetCompletedArgs {
    #[serde(default)]
    pub item_ids: String,
}

/// Remplace l'ensemble des étapes cochées d'un séjour.
///
/// Un `ChoiceList` multiple est un ensemble, pas un événement : il renvoie tout ce qui est coché à
/// chaque basculement. Remplacer est donc idempotent — rejouer le même appel ne double rien.
///
/// ponytail: dernier écrit gagne. Deux téléphones qui cochent chacun une étape au même instant
/// s'écrasent l'un l'autre ; pour une liste de départ remplie par un voyageur sur son téléphone,
/// l'échange ne vaut pas un journal d'opérations.
#[portaki_sdk::command(
    name = "setCompleted",
    guest,
    example(
        label = "Deux étapes cochées",
        input = r#"{"itemIds":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c,1f8e7d6c-5b4a-4938-8271-6e5d4c3b2a19"}"#
    ),
    example(label = "Tout décoché", input = r#"{"itemIds":""}"#)
)]
pub fn set_completed(ctx: Context, args: SetCompletedArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let guest_items = queries::guest_items()?;
    let mut wanted = Vec::new();
    for raw in args.item_ids.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let id = Uuid::parse_str(trimmed)
            .map_err(|_| PortakiError::Host("invalid_item_id".to_string()))?;
        if !guest_items.iter().any(|item| item.uuid() == id) {
            return Err(PortakiError::Host("not_guest_item".to_string()));
        }
        if !wanted.contains(&id) {
            wanted.push(id);
        }
    }

    let already: Vec<Uuid> = storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect();
    for id in &already {
        if !wanted.contains(id) && guest_items.iter().any(|item| item.uuid() == *id) {
            storage::uncomplete_item(stay_id, *id)?;
        }
    }
    for id in &wanted {
        if !already.contains(id) {
            storage::complete_item(stay_id, *id)?;
        }
    }
    emit_progress(ctx.property_id, stay_id)
}

/// A guest ticks only the lists written for guests, never the host's.
fn require_guest_item(item_id: Uuid) -> Result<()> {
    if queries::guest_items()?
        .iter()
        .any(|item| item.uuid() == item_id)
    {
        Ok(())
    } else {
        Err(PortakiError::Host("not_guest_item".to_string()))
    }
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
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
fn emit_progress(property_id: Uuid, stay_id: Uuid) -> Result<()> {
    let guest_items = queries::guest_items()?;
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
