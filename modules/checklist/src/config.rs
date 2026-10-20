//! La règle de publication qu'aucun champ ne dit.

use portaki_sdk::resource::{Collection, Problem};

use crate::List;

/// Une étape écrite au moins, sur l'ensemble des listes.
pub fn problems(lists: &Collection<List, portaki_sdk::resource::NoMeta>) -> Vec<Problem> {
    let written = lists
        .rows()
        .iter()
        .any(|list| !list.written_steps().is_empty());
    (!written)
        .then(|| Problem {
            row: None,
            field: "steps",
            message: portaki_sdk::bundle_text!("publish.checklist.hint"),
        })
        .into_iter()
        .collect()
}
