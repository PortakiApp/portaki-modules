//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Le module n'en avait aucune : le délai de signalement et les options de restitution étaient
//! décrits dans une phrase de l'interface hôte sans exister nulle part. Deux cas de la maquette
//! étaient donc inatteignables — *délai dépassé* et *renvoi aux frais du voyageur*.

use serde::{Deserialize, Serialize};

/// Le délai par défaut, quand l'hôte n'a rien choisi.
pub const DEFAULT_WINDOW_DAYS: u32 = 7;

/// Les bornes du délai : en dessous d'un jour il n'y a pas de fenêtre, au-delà de soixante le
/// logement a changé de mains plusieurs fois.
pub const MIN_WINDOW_DAYS: u32 = 1;
pub const MAX_WINDOW_DAYS: u32 = 60;

/// Qui paie le renvoi. Seule la valeur qui s'écarte du défaut a besoin d'un nom.
pub const SHIPPING_HOST: &str = "host";

#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    /// Combien de jours après le départ le voyageur peut encore signaler un oubli.
    ///
    /// `0` veut dire « rien choisi » et non « aucun délai » : un `u32` dérivé vaut zéro par
    /// défaut, et fermer la porte le jour même serait un choix que l'hôte n'a pas fait.
    #[field(label = "host.window.label")]
    pub window_days: u32,
    /// Renvoi postal proposé.
    #[field(label = "host.return.ship")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub return_ship: bool,
    /// Récupération sur place proposée.
    #[field(label = "host.return.pickup")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub return_pickup: bool,
    /// Don de l'objet proposé.
    #[field(label = "host.return.donate")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub return_donate: bool,
    /// `guest` ou `host` — qui paie le renvoi, quand le renvoi est proposé.
    #[field(
        kind = "select",
        options = ["guest", "host"],
        label = "host.shipping.label"
    )]
    pub shipping_paid_by: String,
}

impl ModuleConfig {
    /// Le délai effectif, borné. Zéro — « rien choisi » — vaut le défaut.
    pub fn window_days(&self) -> u32 {
        if self.window_days == 0 {
            return DEFAULT_WINDOW_DAYS;
        }
        self.window_days.clamp(MIN_WINDOW_DAYS, MAX_WINDOW_DAYS)
    }

    /// Les options de restitution que l'hôte propose, dans l'ordre où le voyageur les lit.
    ///
    /// Aucune cochée vaut les deux premières : un hôte qui n'a pas touché à ce réglage propose le
    /// renvoi et la récupération, ce que le module faisait déjà en toutes lettres.
    pub fn return_options(&self) -> Vec<&'static str> {
        let chosen: Vec<&'static str> = [
            ("ship", self.return_ship),
            ("pickup", self.return_pickup),
            ("donate", self.return_donate),
        ]
        .into_iter()
        .filter(|(_, on)| *on)
        .map(|(key, _)| key)
        .collect();
        if chosen.is_empty() {
            vec!["ship", "pickup"]
        } else {
            chosen
        }
    }

    /// Le renvoi postal est proposé — c'est ce qui décide de demander une adresse.
    pub fn offers_shipping(&self) -> bool {
        self.return_options().contains(&"ship")
    }

    /// Le voyageur paie le renvoi. Vrai par défaut : c'est le cas courant, et l'hôte qui prend les
    /// frais à sa charge fait un geste qu'il choisit.
    pub fn shipping_paid_by_guest(&self) -> bool {
        self.shipping_paid_by.trim() != SHIPPING_HOST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untouched_config_keeps_what_the_module_used_to_say() {
        let config = ModuleConfig::default();
        assert_eq!(config.window_days(), DEFAULT_WINDOW_DAYS);
        assert_eq!(config.return_options(), ["ship", "pickup"]);
        assert!(config.offers_shipping());
        assert!(config.shipping_paid_by_guest());
    }

    #[test]
    fn the_window_stays_between_one_and_sixty_days() {
        let bounded = |days| {
            ModuleConfig {
                window_days: days,
                ..ModuleConfig::default()
            }
            .window_days()
        };
        assert_eq!(bounded(1), 1);
        assert_eq!(bounded(30), 30);
        assert_eq!(bounded(900), MAX_WINDOW_DAYS);
    }

    /// Un hôte qui ne propose que le don ne propose pas le renvoi : pas d'adresse à demander.
    #[test]
    fn only_donating_asks_for_no_address() {
        let config = ModuleConfig {
            return_donate: true,
            ..ModuleConfig::default()
        };
        assert_eq!(config.return_options(), ["donate"]);
        assert!(!config.offers_shipping());
    }

    #[test]
    fn the_host_may_take_the_postage() {
        let config = ModuleConfig {
            shipping_paid_by: SHIPPING_HOST.into(),
            ..ModuleConfig::default()
        };
        assert!(!config.shipping_paid_by_guest());
    }
}
