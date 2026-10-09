//! Ce qui ne va pas dans les réglages de l'hôte (spec Consommables §2), champ par champ.
//!
//! Pas de `#[portaki_sdk::config]` ici : les produits sont des lignes d'entité et les réglages des
//! demandes vivent en KV. Les contrôles portent donc sur ce qui est stocké, et la même liste nourrit
//! le message sous le champ et `publishReadiness`, pour que les deux disent la même chose.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;

use crate::entities::ConsumableItem;
use crate::host::MAX_ITEMS;
use crate::i18n::text;
use crate::labels::labels_from_item;
use crate::storage::settings::{Settings, MAX_MAX_REQUESTS, MIN_MAX_REQUESTS};

/// Le nom d'un produit et le délai annoncé : 40 caractères au plus.
const NAME_MAX: usize = 40;
const RESTOCK_DELAY_MAX: usize = 40;

/// Ce que l'hôte a enregistré, tel que les contrôles le lisent.
pub struct Stored<'a> {
    pub items: &'a [ConsumableItem],
    pub restock_delay: Option<&'a I18nText>,
    pub settings: &'a Settings,
}

impl Stored<'_> {
    /// `(champ du formulaire, message)` : `items`, `items.<i>.label`, `restock_delay`,
    /// `max_requests`.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let mut problems = Vec::new();
        if self.items.len() > MAX_ITEMS {
            problems.push(("items".into(), text("host.products.tooMany", &[])));
        }
        for (index, item) in self.items.iter().enumerate() {
            let labels = labels_from_item(item);
            let error = if labels.is_empty() {
                Some(text("host.product.name.required", &[]))
            } else {
                labels
                    .values()
                    .find_map(|label| check::max_chars(label, NAME_MAX))
            };
            if let Some(error) = error {
                problems.push((format!("items.{index}.label"), error));
            }
        }
        // Demandes fermées : le plafond et le délai sont masqués, une erreur n'aurait pas de champ.
        if !self.settings.requests_enabled() {
            return problems;
        }
        if let Some(error) = self.restock_delay.and_then(|delay| {
            delay
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, RESTOCK_DELAY_MAX))
        }) {
            problems.push(("restock_delay".into(), error));
        }
        if let Some(error) = self.settings.max_requests.and_then(|max| {
            check::between(
                f64::from(max),
                f64::from(MIN_MAX_REQUESTS),
                f64::from(MAX_MAX_REQUESTS),
            )
        }) {
            problems.push(("max_requests".into(), error));
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }
}

/// Le libellé du champ en défaut, tel que le formulaire le montre.
pub fn field_label(field: &str) -> &'static str {
    match field {
        "items" => "host.main.catalogTitle",
        "restock_delay" => "host.main.restockDelay",
        "max_requests" => "host.requests.max",
        _ => "host.product.name",
    }
}
