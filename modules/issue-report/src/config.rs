//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Le module n'en avait aucune : les cinq catégories étaient un `enum` Rust fermé, donc les mêmes
//! pastilles pour tous les logements. Un gîte sans voisin n'a pas de catégorie « Bruit » à
//! proposer, et une pastille qu'on ne choisit jamais encombre un formulaire qu'on remplit vite.

use serde::{Deserialize, Serialize};

/// Une case par catégorie : le formulaire hôte envoie des champs plats, et un ensemble n'a pas de
/// ligne à fusionner — c'est le raisonnement déjà appliqué aux jours de collecte de `waste-recycling`.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(label = "host.category.appliance")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_appliance: bool,
    #[field(label = "host.category.cleanliness")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_cleanliness: bool,
    #[field(label = "host.category.noise")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_noise: bool,
    #[field(label = "host.category.access")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_access: bool,
    #[field(label = "host.category.other")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_other: bool,
}

impl ModuleConfig {
    /// Les catégories proposées, dans l'ordre du formulaire.
    ///
    /// Aucune cochée vaut les cinq : un hôte qui n'a pas touché à ce réglage garde le formulaire
    /// qu'il avait, et personne ne se retrouve avec un formulaire sans catégorie à choisir.
    pub fn categories(&self) -> Vec<&'static str> {
        let chosen: Vec<&'static str> = [
            ("appliance", self.category_appliance),
            ("cleanliness", self.category_cleanliness),
            ("noise", self.category_noise),
            ("access", self.category_access),
            ("other", self.category_other),
        ]
        .into_iter()
        .filter(|(_, on)| *on)
        .map(|(wire, _)| wire)
        .collect();
        if chosen.is_empty() {
            crate::category::WIRE_VALUES.to_vec()
        } else {
            chosen
        }
    }

    /// La catégorie est-elle proposée par cet hôte ?
    pub fn offers(&self, wire: &str) -> bool {
        self.categories().contains(&wire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untouched_config_offers_every_category() {
        let config = ModuleConfig::default();
        assert_eq!(config.categories(), crate::category::WIRE_VALUES.to_vec());
        assert!(config.offers("noise"));
    }

    /// Un gîte isolé retire « Bruit » : la pastille disparaît, et la catégorie est refusée.
    #[test]
    fn the_host_may_drop_a_category() {
        let config = ModuleConfig {
            category_appliance: true,
            category_other: true,
            ..ModuleConfig::default()
        };
        assert_eq!(config.categories(), ["appliance", "other"]);
        assert!(!config.offers("noise"));
    }
}
