//! OpenWeather connector declaration for the weather module manifest.
//! HTTP paths are owned by the module; runtime executes generic egress from this metadata.

#[portaki_sdk::custom_connector(
    id = "open-weather",
    display_name_key = "connector.openWeather.name",
    base_url = "https://api.openweathermap.org",
    credential_provider_id = "open-weather"
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleOpenWeather;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleOpenWeather {
    #[portaki_sdk::connector_op(
        connector = "open-weather",
        method = "GET",
        path = "/data/2.5/weather"
    )]
    pub fn current() {}

    #[portaki_sdk::connector_op(
        connector = "open-weather",
        method = "GET",
        path = "/data/2.5/forecast"
    )]
    pub fn forecast() {}
}

/// OpenWeather Statistical Weather Data : les moyennes d'un mois, pour la page publique.
///
/// Un autre hôte que les prévisions (`history.openweathermap.org`), donc un autre connecteur, avec
/// la même clé OpenWeather. L'API n'est ouverte qu'aux offres Expert (et étudiants) : sans elle,
/// l'appel échoue et la page montre seulement la température du jour (`crate::climate`).
///
/// Tant que le catalogue des connecteurs de la plateforme (`contracts/connectors/open-weather.json`)
/// ne permet pas ce chemin sur cet hôte, le runtime refuse l'appel (`OPERATION_NOT_CATALOGED`) :
/// même dégradation, rien ne casse.
#[portaki_sdk::custom_connector(
    id = "open-weather-statistics",
    display_name_key = "connector.openWeatherStatistics.name",
    base_url = "https://history.openweathermap.org",
    credential_provider_id = "open-weather"
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleOpenWeatherStatistics;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleOpenWeatherStatistics {
    /// `month` de 1 à 12 ; `result.temp.mean` en kelvins.
    #[portaki_sdk::connector_op(
        connector = "open-weather-statistics",
        method = "GET",
        path = "/data/2.5/aggregated/month",
        fields = "lat, lon, month",
        sends = "property_coordinates"
    )]
    pub fn month() {}
}
