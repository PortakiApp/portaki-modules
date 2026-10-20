//! `setCompleted` : l'ensemble des étapes cochées d'un séjour.

use portaki_sdk::prelude::*;
use uuid::Uuid;

use super::progress::{emit_progress, require_stay_id};
use crate::storage;

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
    let guest_items = crate::domain::guest_items()?;
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
