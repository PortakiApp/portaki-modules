//! Le contrôle des réglages de la collection.

use portaki_sdk::resource::{Collection, Problem};

use crate::{Product, Requests};

/// Demandes ouvertes sans canal : personne ne serait prévenu.
pub fn problems(products: &Collection<Product, Requests>) -> Vec<Problem> {
    let requests = products.meta();
    let silent = requests.enabled() && requests.notify.as_ref().is_some_and(Vec::is_empty);
    silent
        .then(|| Problem {
            row: None,
            field: "notify",
            message: portaki_sdk::bundle_text!("host.notify.required"),
        })
        .into_iter()
        .collect()
}
