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
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    // §2.1 Formulaire.
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
    #[field(label = "host.category.wifi")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub category_wifi: bool,
    #[field(label = "host.category.outdoor")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub category_outdoor: bool,
    #[field(label = "host.category.other")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub category_other: bool,
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
    /// Les périodes où le formulaire est proposé. Aucune cochée : toutes, comme avant ce réglage.
    #[field(label = "host.phase.before")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub phase_before: bool,
    #[field(label = "host.phase.during")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub phase_during: bool,
    #[field(label = "host.phase.after")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub phase_after: bool,
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
            ("wifi", self.category_wifi),
            ("outdoor", self.category_outdoor),
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

    pub fn photo_allowed(&self) -> bool {
        self.photo_allowed.unwrap_or(true)
    }

    pub fn urgent_phone(&self) -> bool {
        self.urgent_phone.unwrap_or(true)
    }

    /// Les périodes cochées ; aucune : toutes. La spec propose « pendant le séjour » par défaut,
    /// mais une case décochée n'est pas stockée : « aucune » se confond avec « jamais réglé », et
    /// fermer le formulaire hors séjour retirerait aux hôtes existants ce qu'ils avaient.
    pub fn phases(&self) -> Vec<&'static str> {
        let chosen: Vec<&'static str> = PHASES
            .into_iter()
            .zip([self.phase_before, self.phase_during, self.phase_after])
            .filter(|(_, on)| *on)
            .map(|(phase, _)| phase)
            .collect();
        if chosen.is_empty() {
            PHASES.to_vec()
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
            category_appliance: true,
            category_other: true,
            ..ModuleConfig::default()
        };
        assert_eq!(config.categories(), ["appliance", "other"]);
        assert!(!config.offers("noise"));
    }

    /// Les périodes cochées ferment le formulaire en dehors ; sans choix, il est toujours ouvert.
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
            phase_during: true,
            ..ModuleConfig::default()
        };
        assert!(!during_only.open_at(at("2026-07-09T12:00:00Z"), checkin, checkout));
        assert!(during_only.open_at(at("2026-07-12T12:00:00Z"), checkin, checkout));
        assert!(!during_only.open_at(at("2026-07-18T12:00:00Z"), checkin, checkout));
        assert!(ModuleConfig::default().open_at(at("2026-07-18T12:00:00Z"), checkin, checkout));
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
