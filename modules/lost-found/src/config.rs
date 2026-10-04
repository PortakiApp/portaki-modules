//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Le module n'en avait aucune : le délai de signalement et les options de restitution étaient
//! décrits dans une phrase de l'interface hôte sans exister nulle part. Deux cas de la maquette
//! étaient donc inatteignables — *délai dépassé* et *renvoi aux frais du voyageur*.

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

/// Le délai par défaut, quand l'hôte n'a rien choisi.
pub const DEFAULT_WINDOW_DAYS: u32 = 7;

/// Les bornes du délai : en dessous d'un jour il n'y a pas de fenêtre, au-delà de soixante le
/// logement a changé de mains plusieurs fois.
pub const MIN_WINDOW_DAYS: u32 = 1;
pub const MAX_WINDOW_DAYS: u32 = 60;

/// Qui paie le renvoi. Seule la valeur qui s'écarte du défaut a besoin d'un nom.
pub const SHIPPING_HOST: &str = "host";

// Pas d'`Eq` : le délai est un flottant, et deux flottants ne se comparent pas par égalité
// totale.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModuleConfig {
    /// Combien de jours après le départ le voyageur peut encore signaler un oubli.
    ///
    /// `0` veut dire « rien choisi » et non « aucun délai » : un flottant dérivé vaut zéro par
    /// défaut, et fermer la porte le jour même serait un choix que l'hôte n'a pas fait.
    ///
    /// Un flottant, parce que le `NumberInput` du formulaire hôte envoie un nombre, pas un
    /// entier : `45.0` au retour d'un enregistrement — ou « 7,5 » tapé en jours — faisait refuser
    /// le champ par serde, donc toute la configuration, et le module rendait son état d'erreur sur
    /// chacune de ses surfaces dès le premier enregistrement d'un hôte. [`Self::window_days`]
    /// arrondit au jour.
    #[field(label = "host.window.label")]
    pub window_days: f64,
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
    /// Le délai sous lequel l'hôte répond à une déclaration : « sous 48 h » (§2.18).
    ///
    /// Du texte et non un nombre d'heures : « le lendemain matin » n'en est pas un, et c'est ce
    /// qu'un hôte écrit. Vide, le formulaire ne promet rien à sa place.
    #[field(label = "host.responseDelay.label")]
    pub response_delay: I18nText,
}

/// Un délai utilisable : fini et strictement positif.
fn positive(value: f64) -> Option<f64> {
    (value.is_finite() && value > 0.0).then_some(value)
}

impl ModuleConfig {
    /// Le délai effectif, en jours entiers, borné. Ce qui n'est pas un délai — zéro, « rien
    /// choisi », mais aussi un négatif ou un NaN — vaut le défaut, et non la borne basse : l'hôte
    /// n'a pas choisi de fermer la fenêtre dès le lendemain.
    pub fn window_days(&self) -> u32 {
        let Some(days) = positive(self.window_days) else {
            return DEFAULT_WINDOW_DAYS;
        };
        (days.round() as u32).clamp(MIN_WINDOW_DAYS, MAX_WINDOW_DAYS)
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
    use serde_json::json;

    /// Le formulaire hôte envoie un nombre, pas un entier : `7.0` doit se relire.
    ///
    /// Avec un `u32`, serde refusait le champ — donc toute la configuration, `ModuleConfig::load`
    /// échouant d'un coup — et le module rendait son état d'erreur sur chacune de ses surfaces dès
    /// le premier enregistrement d'un hôte.
    #[test]
    fn a_window_sent_as_a_float_still_reads() {
        let parsed = |value| serde_json::from_value::<ModuleConfig>(value).expect("config");
        assert_eq!(parsed(json!({ "window_days": 7.0 })).window_days(), 7);
        // Un délai tapé à la virgule s'arrondit au jour.
        assert_eq!(parsed(json!({ "window_days": 7.5 })).window_days(), 8);
    }

    /// Ce qui n'est pas un délai vaut « rien choisi », et non un délai d'un jour : un hôte qui
    /// laisse le champ vide garde la fenêtre que le module annonçait.
    #[test]
    fn an_absurd_window_falls_back_to_the_default() {
        let bounded = |days| {
            ModuleConfig {
                window_days: days,
                ..ModuleConfig::default()
            }
            .window_days()
        };
        assert_eq!(bounded(0.0), DEFAULT_WINDOW_DAYS);
        assert_eq!(bounded(-5.0), DEFAULT_WINDOW_DAYS);
        assert_eq!(bounded(f64::NAN), DEFAULT_WINDOW_DAYS);
        assert_eq!(bounded(f64::INFINITY), DEFAULT_WINDOW_DAYS);
        // Une fraction de jour reste un choix : la borne basse est d'un jour.
        assert_eq!(bounded(0.2), MIN_WINDOW_DAYS);
    }

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
        assert_eq!(bounded(1.0), 1);
        assert_eq!(bounded(30.0), 30);
        assert_eq!(bounded(900.0), MAX_WINDOW_DAYS);
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
