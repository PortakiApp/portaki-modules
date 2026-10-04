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
    /// Une phrase libre sous le tableau : « Le bus 14 descend à la gare en 10 min. »
    #[field(label = "config.note")]
    pub note: I18nText,
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
