//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Le module n'en avait aucune : les cinq catégories étaient un `enum` Rust fermé, donc les mêmes
//! pastilles pour tous les logements. Un gîte sans voisin n'a pas de catégorie « Bruit » à
//! proposer, et une pastille qu'on ne choisit jamais encombre un formulaire qu'on remplit vite.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

/// Une case par catégorie : le formulaire hôte envoie des champs plats, et un ensemble n'a pas de
/// ligne à fusionner — c'est le raisonnement déjà appliqué aux jours de collecte de `waste-recycling`.
///
/// Les cases sont des `Option` : une case jamais touchée (`None`) n'est pas une case décochée
/// (`Some(false)`). Sans cette différence, « tout décoché » se confondait avec « jamais réglé », et
/// exiger une case aurait bloqué la publication de chaque logement qui n'a pas ouvert le tiroir.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    // §2.1 Formulaire.
    #[field(label = "host.category.appliance")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_appliance: Option<bool>,
    #[field(label = "host.category.cleanliness")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_cleanliness: Option<bool>,
    #[field(label = "host.category.noise")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_noise: Option<bool>,
    #[field(label = "host.category.access")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_access: Option<bool>,
    #[field(label = "host.category.wifi")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_wifi: Option<bool>,
    #[field(label = "host.category.outdoor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_outdoor: Option<bool>,
    #[field(label = "host.category.other")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_other: Option<bool>,
    /// « Ajouter une photo » proposé. Absent : oui, comme avant ce réglage.
    #[field(label = "host.photo.label")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_allowed: Option<bool>,
    /// L'option « Tout de suite · votre hôte est appelé ». Absent : oui.
    #[field(label = "host.urgent.label")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urgent_phone: Option<bool>,
    /// Le numéro appelé ; vide : celui du profil de l'hôte.
    #[field(label = "host.urgentNumber.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub urgent_number: String,
    /// Les périodes où le formulaire est proposé. Jamais réglées : pendant le séjour (§2.1).
    #[field(label = "host.phase.before")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_before: Option<bool>,
    #[field(label = "host.phase.during")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_during: Option<bool>,
    #[field(label = "host.phase.after")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_after: Option<bool>,
    // §2.2 Réception.
    /// La confirmation après l'envoi ; vide : « Merci, votre hôte a reçu votre message. ».
    #[field(label = "host.autoReply.label")]
    pub auto_reply: I18nText,
    /// `within_day`, `within_hour` ou `none` ([`RESPONSE_TIMES`]).
    #[field(
        kind = "select",
        options = ["within_day", "within_hour", "none"],
        label = "host.responseTime.label"
    )]
    pub response_time: String,
}

/// Le délai annoncé, « dans la journée » d'abord : c'est le défaut (§2.2).
pub const RESPONSE_TIMES: [&str; 3] = ["within_day", "within_hour", "none"];
/// Les périodes du séjour (§2.1).
pub const PHASES: [&str; 3] = ["before", "during", "after"];
const AUTO_REPLY_MAX: usize = 200;

impl ModuleConfig {
    /// Les cases de catégorie telles que le tiroir les montre, dans l'ordre de
    /// [`crate::category::WIRE_VALUES`]. Jamais réglées : toutes cochées (§2.1, défaut « toutes »).
    pub fn category_ticks(&self) -> [bool; 7] {
        ticks(
            [
                self.category_appliance,
                self.category_cleanliness,
                self.category_noise,
                self.category_access,
                self.category_wifi,
                self.category_outdoor,
                self.category_other,
            ],
            [true; 7],
        )
    }

    /// Les catégories proposées, dans l'ordre du formulaire.
    ///
    /// Toutes décochées, la publication est bloquée ([`Self::problems`]) ; d'ici là le livret
    /// propose les sept plutôt qu'un formulaire sans catégorie à choisir.
    pub fn categories(&self) -> Vec<&'static str> {
        let chosen: Vec<&'static str> = crate::category::WIRE_VALUES
            .iter()
            .copied()
            .zip(self.category_ticks())
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

    pub fn photo_allowed(&self) -> bool {
        self.photo_allowed.unwrap_or(true)
    }

    pub fn urgent_phone(&self) -> bool {
        self.urgent_phone.unwrap_or(true)
    }

    /// Les cases de période telles que le tiroir les montre. Jamais réglées : pendant le séjour.
    pub fn phase_ticks(&self) -> [bool; 3] {
        ticks(
            [self.phase_before, self.phase_during, self.phase_after],
            [false, true, false],
        )
    }

    /// Les périodes cochées. Aucune : la publication est bloquée ([`Self::problems`]), et le
    /// livret s'en tient à « pendant le séjour » — jamais à toutes les périodes.
    pub fn phases(&self) -> Vec<&'static str> {
        let chosen: Vec<&'static str> = PHASES
            .into_iter()
            .zip(self.phase_ticks())
            .filter(|(_, on)| *on)
            .map(|(phase, _)| phase)
            .collect();
        if chosen.is_empty() {
            vec!["during"]
        } else {
            chosen
        }
    }

    /// Le formulaire est-il proposé à `now` ? Avant l'arrivée, pendant le séjour ou après le
    /// départ, selon les périodes cochées. Sans dates de séjour, on ne sait pas : il l'est.
    pub fn open_at(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        checkin: Option<chrono::DateTime<chrono::Utc>>,
        checkout: Option<chrono::DateTime<chrono::Utc>>,
    ) -> bool {
        let phase = match (checkin, checkout) {
            (Some(checkin), _) if now < checkin => "before",
            (_, Some(checkout)) if now >= checkout => "after",
            _ => "during",
        };
        self.phases().contains(&phase)
    }

    /// Le délai annoncé, dans la liste ; « dans la journée » sans choix.
    pub fn response_time(&self) -> &'static str {
        RESPONSE_TIMES
            .iter()
            .find(|key| **key == self.response_time.trim())
            .unwrap_or(&RESPONSE_TIMES[0])
    }

    /// Ce qui ne va pas, champ par champ — sous le champ, et dans `publishReadiness`.
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        let mut problems = Vec::new();
        if !self.category_ticks().contains(&true) {
            problems.push((
                "categories",
                crate::i18n::text("host.category.required", &[]),
            ));
        }
        if !self.phase_ticks().contains(&true) {
            problems.push(("phases", crate::i18n::text("host.phase.required", &[])));
        }
        if self.urgent_phone() {
            if let Some(error) = check::phone(&compact(&self.urgent_number)) {
                problems.push(("urgent_number", error));
            }
        }
        if let Some(error) = self
            .auto_reply
            .by_language()
            .find_map(|(_, text)| check::max_chars(text, AUTO_REPLY_MAX))
        {
            problems.push(("auto_reply", error));
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| *name == field)
            .map(|(_, error)| error)
    }
}

/// Une case touchée (`Some`) fait de l'ensemble un choix explicite, les autres comptent pour
/// décochées ; aucune touchée : `default`.
fn ticks<const N: usize>(values: [Option<bool>; N], default: [bool; N]) -> [bool; N] {
    if values.iter().all(Option::is_none) {
        default
    } else {
        values.map(|value| value.unwrap_or(false))
    }
}

/// Un numéro sans espaces, points ni tirets.
pub fn compact(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '.' | '-' | '(' | ')'))
        .collect()
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
            category_appliance: Some(true),
            category_other: Some(true),
            ..ModuleConfig::default()
        };
        assert_eq!(config.categories(), ["appliance", "other"]);
        assert!(!config.offers("noise"));
    }

    /// Les périodes cochées ferment le formulaire en dehors ; jamais réglées : pendant le séjour.
    #[test]
    fn the_form_opens_in_the_chosen_phases() {
        let at = |raw: &str| {
            chrono::DateTime::parse_from_rfc3339(raw)
                .unwrap()
                .with_timezone(&chrono::Utc)
        };
        let (checkin, checkout) = (
            Some(at("2026-07-10T15:00:00Z")),
            Some(at("2026-07-17T10:00:00Z")),
        );
        let during_only = ModuleConfig {
            phase_during: Some(true),
            ..ModuleConfig::default()
        };
        assert!(!during_only.open_at(at("2026-07-09T12:00:00Z"), checkin, checkout));
        assert!(during_only.open_at(at("2026-07-12T12:00:00Z"), checkin, checkout));
        assert!(!during_only.open_at(at("2026-07-18T12:00:00Z"), checkin, checkout));
        let untouched = ModuleConfig::default();
        assert!(!untouched.open_at(at("2026-07-09T12:00:00Z"), checkin, checkout));
        assert!(untouched.open_at(at("2026-07-12T12:00:00Z"), checkin, checkout));
        assert!(!untouched.open_at(at("2026-07-18T12:00:00Z"), checkin, checkout));
    }

    /// Tout décoché bloque la publication, et le livret s'en tient au séjour — jamais à toutes
    /// les périodes.
    #[test]
    fn unticking_every_phase_is_refused_and_falls_back_to_the_stay() {
        let config: ModuleConfig = serde_json::from_value(serde_json::json!({
            "phase_before": false, "phase_during": false, "phase_after": false
        }))
        .unwrap();
        assert_eq!(config.phases(), ["during"]);
        assert_eq!(
            config.error_of("phases").unwrap().get("fr"),
            "Choisissez au moins une période."
        );
        assert!(config.error_of("categories").is_none());
    }

    /// Tout décoché bloque la publication ; le livret garde les sept catégories d'ici là.
    #[test]
    fn unticking_every_category_is_refused() {
        let config: ModuleConfig = serde_json::from_value(serde_json::json!({
            "category_appliance": false, "category_other": false
        }))
        .unwrap();
        assert_eq!(config.categories(), crate::category::WIRE_VALUES.to_vec());
        assert_eq!(
            config.error_of("categories").unwrap().get("fr"),
            "Choisissez au moins une catégorie."
        );
    }

    /// Le numéro d'urgence n'est vérifié que si l'option est ouverte.
    #[test]
    fn the_urgent_number_is_checked_only_when_offered() {
        let config = |value| serde_json::from_value::<ModuleConfig>(value).unwrap();
        assert!(config(serde_json::json!({ "urgent_number": "06 12" }))
            .error_of("urgent_number")
            .is_some());
        assert!(
            config(serde_json::json!({ "urgent_number": "06 12", "urgent_phone": false }))
                .problems()
                .is_empty()
        );
        assert!(ModuleConfig::default().problems().is_empty());
    }
}
