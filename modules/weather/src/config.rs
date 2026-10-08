//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Règles communes de la config hôte et spec Météo (§2) : le lieu des prévisions, le nom affiché,
//! et la carte avant l'arrivée. Les unités suivent la langue du voyageur (`WeatherUnits::for_locale`),
//! la fenêtre est celle du séjour : ni l'un ni l'autre n'est un réglage.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::sdui::common::GeoPoint;
use serde::{Deserialize, Serialize};

/// Le nom affiché fait 40 caractères au plus (§2.1).
pub const LABEL_MAX: usize = 40;

/// Au-delà, la position corrigée est sans doute une erreur : un avertissement, jamais un blocage.
pub const OVERRIDE_RADIUS_KM: f64 = 200.0;

/// Owner-configurable module settings. The keys are the names of the host form fields: the
/// platform takes `updateConfig`, and a missing key reads as its [`Default`].
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// Sous-titre du livret ; vide, la commune du logement.
    #[field(label = "host.locationLabel.label")]
    pub location_label: String,
    /// Prévoir ailleurs qu'à l'adresse du logement (montagne, île), en repli.
    #[field(label = "host.locationOverride.label")]
    pub location_override: bool,
    /// L'adresse cherchée dans le sélecteur ; seule la position compte pour les prévisions.
    #[field(label = "host.locationOverride.address")]
    pub location_address: String,
    #[field(label = "host.locationOverride.lat")]
    pub location_lat: Option<f64>,
    #[field(label = "host.locationOverride.lng")]
    pub location_lng: Option<f64>,
    /// La carte du fil avant l'arrivée, à partir de J-5.
    #[field(label = "host.showUpcoming.label")]
    pub show_upcoming: bool,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            location_label: String::new(),
            location_override: false,
            location_address: String::new(),
            location_lat: None,
            location_lng: None,
            show_upcoming: true,
        }
    }
}

impl ModuleConfig {
    /// La position corrigée, quand le repli est choisi et l'épingle posée.
    pub fn override_point(&self) -> Option<GeoPoint> {
        if !self.location_override {
            return None;
        }
        match (self.location_lat, self.location_lng) {
            (Some(lat), Some(lng)) if check::pin(Some(lat), Some(lng)).is_none() => {
                Some(GeoPoint { lat, lng })
            }
            _ => None,
        }
    }

    /// Où prévoir : la position corrigée, sinon celle du logement.
    pub fn point(&self, property: Option<GeoPoint>) -> Option<GeoPoint> {
        self.override_point().or(property)
    }

    /// Le nom affiché, `None` quand l'hôte n'en a pas donné.
    pub fn label(&self) -> Option<&str> {
        Some(self.location_label.trim()).filter(|label| !label.is_empty())
    }

    /// « 40 caractères au maximum. »
    pub fn label_error(&self) -> Option<I18nText> {
        check::max_chars(&self.location_label, LABEL_MAX)
    }

    /// « Placez l'épingle sur la carte. », quand le repli est choisi sans position.
    pub fn pin_error(&self) -> Option<I18nText> {
        if !self.location_override {
            return None;
        }
        check::pin(self.location_lat, self.location_lng)
    }

    /// La position corrigée est-elle loin du logement ? `false` sans l'une ou l'autre.
    pub fn override_is_far(&self, property: Option<GeoPoint>) -> bool {
        match (self.override_point(), property) {
            (Some(pin), Some(home)) => {
                !check::within_km(pin.lat, pin.lng, home.lat, home.lng, OVERRIDE_RADIUS_KM)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    const HOME: GeoPoint = GeoPoint {
        lat: 47.25,
        lng: 0.43,
    };

    #[test]
    fn a_new_install_shows_the_upcoming_card_and_forecasts_at_the_property() {
        let config = ModuleConfig::default();
        assert!(config.show_upcoming);
        assert_eq!(config.point(Some(HOME)), Some(HOME));
        assert_eq!(config.label(), None);
    }

    #[test]
    fn the_override_counts_only_when_chosen_and_placed() {
        let mut config = ModuleConfig {
            location_lat: Some(45.12),
            location_lng: Some(5.88),
            ..ModuleConfig::default()
        };
        assert_eq!(config.point(Some(HOME)), Some(HOME), "repli non choisi");
        config.location_override = true;
        assert_eq!(
            config.point(Some(HOME)),
            Some(GeoPoint {
                lat: 45.12,
                lng: 5.88
            })
        );
        assert!(
            config.override_is_far(Some(HOME)),
            "Chamrousse est à plus de 200 km de Tours"
        );

        config.location_lng = None;
        assert_eq!(config.point(Some(HOME)), Some(HOME));
        assert_eq!(
            config.pin_error().unwrap().get("fr"),
            "Placez l'épingle sur la carte."
        );
    }

    #[test]
    fn the_label_is_trimmed_and_bounded() {
        let config = ModuleConfig {
            location_label: "  Chamrousse 1750 ".into(),
            ..ModuleConfig::default()
        };
        assert_eq!(config.label(), Some("Chamrousse 1750"));
        assert!(config.label_error().is_none());
        let long = ModuleConfig {
            location_label: "x".repeat(41),
            ..ModuleConfig::default()
        };
        assert_eq!(
            long.label_error().unwrap().get("fr"),
            "40 caractères au maximum."
        );
    }

    /// Les anciens réglages (unités, rafraîchissement) restent stockés : ils ne gênent pas la lecture.
    #[test]
    #[serial_test::serial]
    fn an_older_config_still_loads() {
        MockContext::guest()
            .with_config(&json!({ "units": "fahrenheit", "refresh_interval": "3h" }))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }
}
