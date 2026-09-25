//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself and
//! stores what the form sends, flat (`activities_enabled`, `tiqets_radius_km`…) and every text a
//! string (`spots.0.lat: "43.5"`, `spots.0.name`). The old KV blob nested two sections
//! (`activities: {enabled, destination, intro, links}`, `tiqets: {…}`): [`ModuleConfig::read`]
//! still finds them there until the host saves the form.

use std::collections::BTreeMap;

use portaki_sdk::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// The six slots of the host form, empty ones included (see [`Self::parse_spots`]).
    #[field(structured, required, label = "config.spots")]
    pub spots: Vec<SpotRow>,
    #[field(kind = "textarea", label = "host.disclaimer.label")]
    #[serde(deserialize_with = "deserialize_localized_field")]
    pub disclaimer: Localized,
    #[field(label = "host.activities.enabled")]
    pub activities_enabled: bool,
    #[field(label = "host.activities.destination")]
    pub activities_destination: String,
    #[field(kind = "textarea", label = "host.activities.intro")]
    #[serde(deserialize_with = "deserialize_localized_field")]
    pub activities_intro: Localized,
    /// The links of the « Activités & billets » section, as the form rows `{ url, label }`.
    #[field(structured, label = "host.section.activities")]
    #[serde(deserialize_with = "deserialize_activity_links")]
    pub activities: Vec<ActivityRow>,
    #[field(label = "host.tiqets.enabled")]
    pub tiqets_enabled: bool,
    #[field(
        kind = "select",
        options = ["5", "10", "20", "40"],
        label = "host.tiqets.radius"
    )]
    #[serde(deserialize_with = "deserialize_radius")]
    pub tiqets_radius_km: u32,
    #[field(kind = "select", options = ["0", "3", "4"], label = "host.tiqets.minRating")]
    #[serde(deserialize_with = "deserialize_min_rating")]
    pub tiqets_min_rating: u8,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            spots: Vec::new(),
            disclaimer: Localized::default(),
            activities_enabled: false,
            activities_destination: String::new(),
            activities_intro: Localized::default(),
            activities: Vec::new(),
            tiqets_enabled: false,
            tiqets_radius_km: TIQETS_DEFAULT_RADIUS_KM,
            tiqets_min_rating: 0,
        }
    }
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
    #[serde(default)]
    pub url: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub label: Localized,
}

impl ModuleConfig {
    /// The config of this install.
    ///
    /// The platform imports the old KV blob once, but only its declared keys at their declared
    /// types: the nested `activities` / `tiqets` sections, a `{fr, en}` disclaimer and the
    /// pre-`spots` `spots_json` do not come across. Each such key still absent from what the
    /// platform holds (the host has not saved the form since) is read from the KV. A key present,
    /// even empty, is the host's: the platform wins.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let held = ctx.module_config.as_ref().and_then(Value::as_object);
        let absent = |key: &str| held.is_none_or(|held| held.get(key).is_none_or(Value::is_null));
        if LEGACY_KEYS.iter().any(|key| absent(key)) {
            config.fill_from_legacy(&portaki_sdk::config::legacy_config()?, absent);
        }
        Ok(config)
    }

    fn fill_from_legacy(&mut self, legacy: &Value, absent: impl Fn(&str) -> bool) {
        let activities = &legacy["activities"];
        let tiqets = &legacy["tiqets"];
        if absent("disclaimer") && !legacy["disclaimer"].is_null() {
            self.disclaimer = Localized::from_value(&legacy["disclaimer"]);
        }
        if absent("activities_enabled") {
            if let Some(enabled) = activities["enabled"].as_bool() {
                self.activities_enabled = enabled;
            }
        }
        if absent("activities_destination") {
            if let Some(destination) = activities["destination"].as_str() {
                self.activities_destination = destination.to_string();
            }
        }
        if absent("activities_intro") && !activities["intro"].is_null() {
            self.activities_intro = Localized::from_value(&activities["intro"]);
        }
        if absent("tiqets_enabled") {
            if let Some(enabled) = tiqets["enabled"].as_bool() {
                self.tiqets_enabled = enabled;
            }
        }
        if absent("tiqets_radius_km") {
            if let Some(radius) = tiqets["radius_km"].as_u64() {
                self.tiqets_radius_km = u32::try_from(radius).unwrap_or(u32::MAX);
            }
        }
        if absent("tiqets_min_rating") {
            if let Some(rating) = tiqets["min_rating"].as_u64() {
                self.tiqets_min_rating = u8::try_from(rating).unwrap_or(u8::MAX);
            }
        }
        if absent("spots") && self.spots.is_empty() {
            if let Some(Ok(spots)) = legacy["spots_json"]
                .as_str()
                .map(serde_json::from_str::<Vec<SpotRow>>)
            {
                self.spots = spots;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.parse_spots().is_empty() && self.disclaimer.is_empty()
    }

    /// The named slots, in form order. A row saved by the host form has no id: it takes its
    /// slot's (`spot-1`…), as the module's own `updateConfig` used to give it.
    pub fn parse_spots(&self) -> Vec<SpotRow> {
        self.spots
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.title.is_empty())
            .map(|(index, s)| {
                let mut s = s.clone();
                if s.id.trim().is_empty() {
                    s.id = format!("spot-{}", index + 1);
                }
                s
            })
            .collect()
    }

    /// The « Activités & billets » section. Links and destination are the host's raw input:
    /// `activities::resolve` normalizes them (GetYourGuide only, partner id) at render.
    pub fn activities(&self) -> ActivitiesConfig {
        ActivitiesConfig {
            enabled: self.activities_enabled,
            destination: self.activities_destination.trim().to_string(),
            intro: self.activities_intro.clone(),
            links: self
                .activities
                .iter()
                .filter(|row| !row.url.trim().is_empty())
                .cloned()
                .collect(),
        }
    }

    pub fn tiqets(&self) -> TiqetsConfig {
        TiqetsConfig {
            enabled: self.tiqets_enabled,
            radius_km: self.tiqets_radius_km,
            min_rating: self.tiqets_min_rating,
        }
    }
}

/// The keys the platform's import can miss (see [`ModuleConfig::read`]).
const LEGACY_KEYS: [&str; 8] = [
    "disclaimer",
    "activities_enabled",
    "activities_destination",
    "activities_intro",
    "tiqets_enabled",
    "tiqets_radius_km",
    "tiqets_min_rating",
    "spots",
];

/// `Eq` en moins des autres structures du module : `lat` et `lng` sont des `f64`, qui
/// n'ont pas d'égalité totale. `PartialEq` suffit partout où on compare des spots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotRow {
    #[serde(default)]
    pub id: String,
    /// `name` in the host form.
    #[serde(
        default,
        alias = "name",
        deserialize_with = "deserialize_localized_field"
    )]
    pub title: Localized,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub distance: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub tag: Option<String>,
    #[serde(default)]
    pub note: Option<Localized>,
    /// `description` in the host form.
    #[serde(
        default,
        alias = "description",
        deserialize_with = "deserialize_localized_opt"
    )]
    pub detail: Option<Localized>,
    /// Adresse postale telle que le sélecteur de carte l'a géocodée.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub address: Option<String>,
    /// Latitude WGS-84, absente tant que l'hôte n'a pas posé le lieu sur la carte. Le
    /// sélecteur de carte l'envoie en texte (`"43.5"`, `""`).
    #[serde(default, deserialize_with = "deserialize_coord")]
    pub lat: Option<f64>,
    /// Longitude WGS-84.
    #[serde(default, deserialize_with = "deserialize_coord")]
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

fn deserialize_localized_opt<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Localized>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Localized::from_value(&Value::deserialize(deserializer)?);
    Ok((!value.is_empty()).then_some(value))
}

/// A blank text input is no value.
fn deserialize_nonempty<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

/// `43.5`, `"43.5"`, or `""` / `null` (none). An unparsable text is none too: dropping the pin
/// beats refusing the whole config.
fn deserialize_coord<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

/// The form rows `[{ url, label }]`, or the old `{ links: […] }` section the import carried
/// over whole (it is a JSON object, which `structured` accepts).
fn deserialize_activity_links<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<ActivityRow>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = match Value::deserialize(deserializer)? {
        Value::Object(mut section) => section.remove("links").unwrap_or(Value::Null),
        other => other,
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}

/// A number, or the select's value as text (`"10"`); `""` is the default.
fn number_or_text<'de, D, N>(deserializer: D, default: N) -> std::result::Result<N, D::Error>
where
    D: Deserializer<'de>,
    N: std::str::FromStr + TryFrom<u64>,
{
    let invalid =
        |raw: &dyn std::fmt::Display| serde::de::Error::custom(format!("not a choice: {raw}"));
    match Value::deserialize(deserializer)? {
        Value::Number(n) => n
            .as_u64()
            .and_then(|n| N::try_from(n).ok())
            .ok_or_else(|| invalid(&n)),
        Value::String(s) if s.trim().is_empty() => Ok(default),
        Value::String(s) => s.trim().parse().map_err(|_| invalid(&s)),
        other => Err(invalid(&other)),
    }
}

fn deserialize_radius<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    number_or_text(deserializer, TIQETS_DEFAULT_RADIUS_KM)
}

fn deserialize_min_rating<'de, D>(deserializer: D) -> std::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    number_or_text(deserializer, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
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
        assert_eq!(config.activities(), ActivitiesConfig::default());
    }

    #[test]
    fn tiqets_stays_off_on_a_config_that_predates_it() {
        let config: ModuleConfig = serde_json::from_value(json!({ "spots": [] })).unwrap();
        assert_eq!(config.tiqets(), TiqetsConfig::default());
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
        assert!(!ModuleConfig::default().activities().enabled);
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
            "activities": [{ "url": "https://gyg.me/aBcD12", "label": "Visite du Suquet" }]
        }))
        .unwrap();
        assert_eq!(config.activities[0].label.get("fr"), "Visite du Suquet");
    }

    /// What the host form sends: every text a string, empty slots included.
    #[test]
    fn the_host_form_shape_reads() {
        let empty = json!({ "name": "", "category": "", "distance": "", "tag": "",
            "description": "", "address": "", "lat": "0", "lng": "0" });
        let mut spots = vec![empty; 6];
        spots[2] = json!({ "name": "Plage du Midi", "category": "Plage", "distance": " ",
            "tag": "", "description": "Sable fin", "address": "Bd du Midi, Cannes",
            "lat": "43.548", "lng": "7.005" });
        let config: ModuleConfig = serde_json::from_value(json!({
            "spots": spots,
            "disclaimer": "Suggestions",
            "activities_enabled": true,
            "activities_destination": " Antibes ",
            "activities_intro": "",
            "activities": [{ "url": "", "label": "" }, { "url": "https://gyg.me/aBcD12", "label": "" }],
            "tiqets_enabled": true,
            "tiqets_radius_km": "20",
            "tiqets_min_rating": "4"
        }))
        .unwrap();
        let parsed = config.parse_spots();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "spot-3");
        assert_eq!(parsed[0].title.pick("en"), "Plage du Midi");
        assert_eq!(
            parsed[0].detail.as_ref().map(|d| d.pick("fr")).as_deref(),
            Some("Sable fin")
        );
        assert_eq!(parsed[0].distance, None);
        assert_eq!(parsed[0].coords(), Some((43.548, 7.005)));
        let activities = config.activities();
        assert!(activities.enabled);
        assert_eq!(activities.destination, "Antibes");
        assert_eq!(activities.links.len(), 1);
        assert_eq!(
            config.tiqets(),
            TiqetsConfig {
                enabled: true,
                radius_km: 20,
                min_rating: 4
            }
        );
    }

    fn legacy_blob() -> Value {
        json!({
            "spots_json": r#"[{"id":"s1","title":{"fr":"Plage"}}]"#,
            "disclaimer": { "fr": "Suggestions", "en": "Suggestions (en)" },
            "activities": {
                "enabled": true, "destination": "Antibes", "intro": { "fr": "Bonjour" },
                "links": [{ "url": "https://gyg.me/aBcD12", "label": { "fr": "Suquet" } }]
            },
            "tiqets": { "enabled": true, "radius_km": 40, "min_rating": 3 }
        })
    }

    /// The import only brings `activities` (an object, which `structured` takes): the nested
    /// settings, the tiqets section, the `{fr, en}` disclaimer and `spots_json` come from the KV
    /// until the host saves.
    #[test]
    #[serial_test::serial]
    fn what_the_import_skips_is_read_from_the_kv() {
        let blob = legacy_blob();
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&blob).unwrap())
            .with_config(&json!({ "activities": blob["activities"] }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.parse_spots()[0].id, "s1");
                assert_eq!(config.disclaimer.get("en"), "Suggestions (en)");
                let activities = config.activities();
                assert!(activities.enabled);
                assert_eq!(activities.destination, "Antibes");
                assert_eq!(activities.intro.get("fr"), "Bonjour");
                assert_eq!(activities.links[0].label.get("fr"), "Suquet");
                assert_eq!(
                    config.tiqets(),
                    TiqetsConfig {
                        enabled: true,
                        radius_km: 40,
                        min_rating: 3
                    }
                );
            });
        // Before the platform holds the config (no `moduleConfig`), the same.
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&blob).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert!(config.activities().enabled);
                assert_eq!(config.tiqets().radius_km, 40);
                assert_eq!(config.parse_spots().len(), 1);
            });
    }

    #[test]
    #[serial_test::serial]
    fn once_saved_the_platform_wins_over_the_kv() {
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&legacy_blob()).unwrap())
            .with_config(&json!({
                "spots": [], "disclaimer": "", "activities_enabled": false,
                "activities_destination": "", "activities_intro": "", "activities": [],
                "tiqets_enabled": false, "tiqets_radius_km": "10", "tiqets_min_rating": "0"
            }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert!(config.parse_spots().is_empty());
                assert!(config.disclaimer.is_empty());
                assert_eq!(config.activities(), ActivitiesConfig::default());
                assert_eq!(config.tiqets(), TiqetsConfig::default());
            });
    }
}
