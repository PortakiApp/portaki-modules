//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

/// Les réglages de l'hôte. Les clés sont les noms des champs du formulaire : la plateforme prend
/// `updateConfig` elle-même, et une clé absente se lit comme son [`Default`].
#[portaki_sdk::config]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// La gare du logement, écrite comme l'hôte la nomme : « Gare d'Antibes », « Antibes ».
    ///
    /// Un nom, pas un identifiant : l'hôte sait le nom de sa gare, pas son `stop_area` Navitia.
    /// Le module le résout lui-même et garde la correspondance.
    #[field(required, label = "config.station")]
    pub station: String,
    /// Les gares que le voyageur trouve en tête du filtre (spec §2.2) — 6 au plus. Le voyageur
    /// peut toujours en choisir une autre.
    #[field(label = "config.destinations")]
    pub destinations: Vec<String>,
    /// Le sens montré d'abord : les départs de la gare, ou les arrivées.
    #[field(kind = "select", options = ["from", "to"], label = "config.direction")]
    pub direction: Direction,
    /// Une phrase libre sous le tableau : « Le bus 14 descend à la gare en 10 min. »
    #[field(label = "config.note")]
    pub note: I18nText,
}

/// Le sens par défaut du tableau.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Départs de la gare.
    #[default]
    From,
    /// Arrivées à la gare.
    To,
}

impl Direction {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::From => "from",
            Self::To => "to",
        }
    }
}

/// Six destinations proposées au plus (spec §2.2).
pub const MAX_DESTINATIONS: usize = 6;

impl ModuleConfig {
    /// Les destinations de l'hôte, vides retirées, dans son ordre.
    pub fn proposed_destinations(&self) -> Vec<String> {
        self.destinations
            .iter()
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .collect()
    }
}

impl ModuleConfig {
    /// La gare, débarrassée des espaces, ou `None` quand l'hôte ne l'a pas donnée.
    pub fn station_name(&self) -> Option<&str> {
        let trimmed = self.station.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_station_of_spaces_is_no_station() {
        let blank: ModuleConfig = serde_json::from_value(json!({ "station": "   " })).unwrap();
        assert_eq!(blank.station_name(), None);
        let named: ModuleConfig =
            serde_json::from_value(json!({ "station": "  Gare d'Antibes " })).unwrap();
        assert_eq!(named.station_name(), Some("Gare d'Antibes"));
    }
}
