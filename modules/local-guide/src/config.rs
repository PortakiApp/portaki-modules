//! Host configuration stored in KV (`config` key).

use std::collections::BTreeMap;

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const CONFIG_KEY: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub spots: Vec<SpotRow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub spots_json: String,
    /// Per-language disclaimer (plain string legacy → `fr` on load).
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub disclaimer: Localized,
    /// Section « Activités & billets » (liens d'affiliation GetYourGuide).
    #[serde(default)]
    pub activities: ActivitiesConfig,
    /// Section « Billets & activités (Tiqets) » (produits à proximité, API Tiqets).
    #[serde(default)]
    pub tiqets: TiqetsConfig,
}

/// Rayon proposé par défaut : la ville et sa proche périphérie.
pub const TIQETS_DEFAULT_RADIUS_KM: u32 = 10;

/// Rayons proposés à l'hôte. Tiqets accepte 1 à 100 km ; au-delà de 40, la liste se remplit
/// d'attractions qu'aucun voyageur ne ferait dans la journée.
pub const TIQETS_RADIUS_CHOICES_KM: [u32; 4] = [5, 10, 20, 40];

/// Configuration de la section Tiqets.
///
/// **Éteinte par défaut**, pour la même raison que la section GetYourGuide : ces liens
/// rapportent une commission, et une configuration écrite avant la section n'a pas la clé.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TiqetsConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Rayon de recherche autour du logement, en km.
    #[serde(default = "default_tiqets_radius_km")]
    pub radius_km: u32,
    /// Note minimale des avis, 1 à 5 ; 0 = aucun filtre.
    #[serde(default)]
    pub min_rating: u8,
}

impl Default for TiqetsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            radius_km: TIQETS_DEFAULT_RADIUS_KM,
            min_rating: 0,
        }
    }
}

fn default_tiqets_radius_km() -> u32 {
    TIQETS_DEFAULT_RADIUS_KM
}

impl TiqetsConfig {
    /// Un rayon que Tiqets accepte : écrit hors du formulaire, il ne doit pas devenir un `400`.
    pub fn normalized_radius_km(&self) -> u32 {
        self.radius_km.clamp(1, 100)
    }

    /// `None` = pas de filtre, sinon 1 à 5.
    pub fn normalized_min_rating(&self) -> Option<u8> {
        match self.min_rating {
            0 => None,
            rating => Some(rating.min(5)),
        }
    }
}

/// Configuration de la section « Activités & billets ».
///
/// La section est **éteinte par défaut**. Ces liens rapportent une commission à Portaki :
/// personne ne doit se mettre à en afficher sans l'avoir décidé, et une configuration
/// écrite avant que la section existe n'a pas la clé — elle reste donc éteinte.
///
/// Une fois allumée, tout le reste est optionnel : elle se débrouille avec la ville de
/// l'adresse du logement, sans que l'hôte ait rien d'autre à saisir.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivitiesConfig {
    /// Affiche la section dans le livret. Éteinte tant que l'hôte ne l'allume pas.
    #[serde(default)]
    pub enabled: bool,
    /// Ville visée, quand celle de l'adresse ne convient pas (« Antibes »).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub destination: String,
    /// Phrase d'introduction affichée au-dessus des liens.
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub intro: Localized,
    /// Liens choisis par l'hôte, dans son ordre.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ActivityRow>,
}

/// Un lien choisi par l'hôte : `{ url, label? }`.
///
/// `label` accepte une chaîne simple autant qu'une carte par langue — c'est le même
/// `Localized` que partout ailleurs dans le module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ActivityRow {
    pub url: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub label: Localized,
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_spots().is_empty() && self.disclaimer.is_empty()
    }

    pub fn parse_spots(&self) -> Vec<SpotRow> {
        self.spots
            .iter()
            .filter(|s| !s.id.trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn migrate_legacy(&mut self) {
        if !self.spots.is_empty() {
            return;
        }
        let raw = self.spots_json.trim();
        if raw.is_empty() {
            return;
        }
        if let Ok(data) = serde_json::from_str::<Vec<SpotRow>>(raw) {
            self.spots = data
                .into_iter()
                .filter(|s| !s.id.trim().is_empty())
                .collect();
            self.spots_json.clear();
        }
    }
}

/// `Eq` en moins des autres structures du module : `lat` et `lng` sont des `f64`, qui
/// n'ont pas d'égalité totale. `PartialEq` suffit partout où on compare des spots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotRow {
    pub id: String,
    pub title: Localized,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub distance: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub note: Option<Localized>,
    #[serde(default)]
    pub detail: Option<Localized>,
    /// Adresse postale telle que le sélecteur de carte l'a géocodée.
    #[serde(default)]
    pub address: Option<String>,
    /// Latitude WGS-84, absente tant que l'hôte n'a pas posé le lieu sur la carte.
    #[serde(default)]
    pub lat: Option<f64>,
    /// Longitude WGS-84.
    #[serde(default)]
    pub lng: Option<f64>,
}

impl SpotRow {
    /// Position affichable du spot, ou `None`.
    pub fn coords(&self) -> Option<(f64, f64)> {
        valid_coords(self.lat?, self.lng?)
    }
}

/// Filtre les coordonnées qu'on ne peut pas afficher.
///
/// Deux rejets. Hors des bornes WGS-84, d'abord : une valeur pareille ne vient pas d'une
/// carte, et la projeter déplacerait toute la vue. Le point `0, 0` ensuite — au large du
/// golfe de Guinée, où aucun hôte n'a de bonne adresse, mais où atterrit n'importe quel
/// formulaire ayant soumis des champs vides. Le refuser vaut mieux que de centrer la carte
/// du livret sur l'Atlantique.
pub fn valid_coords(lat: f64, lng: f64) -> Option<(f64, f64)> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return None;
    }
    if lat.abs() < f64::EPSILON && lng.abs() < f64::EPSILON {
        return None;
    }
    Some((lat, lng))
}

/// N-language string map. Legacy `{fr,en}` deserializes as-is; extra langs via flatten.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Localized {
    #[serde(default)]
    pub fr: String,
    #[serde(default)]
    pub en: String,
    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

impl Localized {
    pub fn lang_code(locale: &str) -> String {
        let trimmed = locale.trim();
        if trimmed.is_empty() {
            return "fr".to_string();
        }
        let lower = trimmed.to_ascii_lowercase();
        let base = lower.split(['-', '_']).next().unwrap_or("fr").trim();
        if base.is_empty() {
            "fr".to_string()
        } else {
            base.to_string()
        }
    }

    pub fn singleton(lang: &str, value: impl Into<String>) -> Self {
        let mut loc = Self::default();
        loc.set(lang, value.into());
        loc
    }

    pub fn get(&self, lang: &str) -> &str {
        let code = Self::lang_code(lang);
        match code.as_str() {
            "fr" => self.fr.as_str(),
            "en" => self.en.as_str(),
            other => self.other.get(other).map(String::as_str).unwrap_or(""),
        }
    }

    pub fn set(&mut self, lang: &str, value: String) {
        let code = Self::lang_code(lang);
        match code.as_str() {
            "fr" => self.fr = value,
            "en" => self.en = value,
            other => {
                if value.trim().is_empty() {
                    self.other.remove(other);
                } else {
                    self.other.insert(other.to_string(), value);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fr.trim().is_empty()
            && self.en.trim().is_empty()
            && self.other.values().all(|v| v.trim().is_empty())
    }

    /// Guest resolve: request locale → property default → `fr` → first non-empty.
    pub fn pick(&self, locale: &str) -> String {
        self.pick_with_fallback(locale, "fr")
    }

    pub fn pick_with_fallback(&self, guest_locale: &str, property_locale: &str) -> String {
        let candidates = [
            Self::lang_code(guest_locale),
            Self::lang_code(property_locale),
            "fr".to_string(),
        ];
        let mut tried = std::collections::BTreeSet::new();
        for lang in &candidates {
            if !tried.insert(lang.clone()) {
                continue;
            }
            let value = self.get(lang);
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
        for (lang, value) in [("fr", self.fr.as_str()), ("en", self.en.as_str())] {
            let _ = lang;
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
        for value in self.other.values() {
            if !value.trim().is_empty() {
                return value.clone();
            }
        }
        String::new()
    }

    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::String(s) => Self::singleton("fr", s.trim()),
            Value::Object(map) => {
                let mut loc = Self::default();
                for (key, val) in map {
                    if let Some(s) = val.as_str() {
                        loc.set(key, s.trim().to_string());
                    }
                }
                loc
            }
            _ => Self::default(),
        }
    }
}

fn deserialize_localized_field<'de, D>(deserializer: D) -> std::result::Result<Localized, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(Localized::from_value(&value))
}

pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    let mut config: ModuleConfig = serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    config.migrate_legacy();
    Ok(config)
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let bytes = serde_json::to_vec(config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn localized_n_lang_roundtrip() {
        let mut loc = Localized::singleton("de", "Hallo");
        loc.set("fr", "Bonjour".into());
        assert_eq!(loc.get("de"), "Hallo");
        assert_eq!(loc.pick_with_fallback("it-IT", "de-DE"), "Hallo");
        let value = serde_json::to_value(&loc).unwrap();
        assert_eq!(value["de"], "Hallo");
        assert_eq!(value["fr"], "Bonjour");
    }

    #[test]
    fn disclaimer_plain_string_migrates() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "disclaimer": "Suggestions non partenaires"
        }))
        .unwrap();
        assert_eq!(config.disclaimer.get("fr"), "Suggestions non partenaires");
    }

    #[test]
    fn activities_stay_off_on_a_config_that_predates_them() {
        // Les configurations déjà en base n'ont pas la clé. Elles ne doivent pas se
        // mettre à afficher des liens d'affiliation parce qu'on a publié une version.
        let config: ModuleConfig = serde_json::from_value(json!({ "spots": [] })).unwrap();
        assert!(!config.activities.enabled);
        assert!(config.activities.destination.is_empty());
        assert!(config.activities.links.is_empty());
    }

    #[test]
    fn tiqets_stays_off_on_a_config_that_predates_it() {
        let config: ModuleConfig = serde_json::from_value(json!({ "spots": [] })).unwrap();
        assert_eq!(config.tiqets, TiqetsConfig::default());
        assert!(!config.tiqets.enabled);
        assert_eq!(config.tiqets.radius_km, TIQETS_DEFAULT_RADIUS_KM);
        assert_eq!(config.tiqets.normalized_min_rating(), None);
    }

    #[test]
    fn tiqets_settings_written_outside_the_form_stay_in_range() {
        let config = TiqetsConfig {
            enabled: true,
            radius_km: 5000,
            min_rating: 9,
        };
        assert_eq!(config.normalized_radius_km(), 100);
        assert_eq!(config.normalized_min_rating(), Some(5));
        let config = TiqetsConfig {
            radius_km: 0,
            ..config
        };
        assert_eq!(config.normalized_radius_km(), 1);
    }

    #[test]
    fn activities_are_off_by_default() {
        assert!(!ModuleConfig::default().activities.enabled);
        assert!(!ActivitiesConfig::default().enabled);
    }

    fn spot(value: Value) -> SpotRow {
        serde_json::from_value(value).expect("spot")
    }

    #[test]
    fn coordinates_off_the_planet_are_refused() {
        assert_eq!(valid_coords(43.5513, 7.0128), Some((43.5513, 7.0128)));
        assert_eq!(valid_coords(91.0, 7.0), None);
        assert_eq!(valid_coords(-90.5, 7.0), None);
        assert_eq!(valid_coords(43.0, 181.0), None);
        // Null Island : ce que rend un formulaire dont les deux champs sont restés vides.
        assert_eq!(valid_coords(0.0, 0.0), None);
        // Mais une seule des deux à zéro reste un point comme un autre — le méridien de
        // Greenwich passe par Villers-sur-Mer, et l'équateur par Quito.
        assert_eq!(valid_coords(0.0, 7.0128), Some((0.0, 7.0128)));
        assert_eq!(valid_coords(43.5513, 0.0), Some((43.5513, 0.0)));
    }

    #[test]
    fn a_spot_reaches_the_map_only_with_both_coordinates() {
        let located = spot(json!({
            "id": "s1", "title": { "fr": "Plage" }, "lat": 43.55, "lng": 7.01
        }));
        assert_eq!(located.coords(), Some((43.55, 7.01)));

        // Une seule des deux ne situe rien.
        let half = spot(json!({ "id": "s1", "title": { "fr": "Plage" }, "lat": 43.55 }));
        assert_eq!(half.coords(), None);

        // Et une configuration écrite avant la carte n'en a aucune : elle se relit sans
        // erreur, elle ne s'affiche simplement pas sur le plan.
        let legacy = spot(json!({
            "id": "s1", "title": { "fr": "Plage" }, "category": "Plage", "tag": "Coup de cœur"
        }));
        assert_eq!(legacy.coords(), None);
        assert_eq!(legacy.category.as_deref(), Some("Plage"));
    }

    #[test]
    fn activity_label_accepts_a_plain_string() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "activities": {
                "links": [{ "url": "https://gyg.me/aBcD12", "label": "Visite du Suquet" }]
            }
        }))
        .unwrap();
        assert_eq!(
            config.activities.links[0].label.get("fr"),
            "Visite du Suquet"
        );
    }
}
