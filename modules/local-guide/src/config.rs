//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself and
//! stores them flat (`activities_enabled`, `tiqets_radius_km`…). The form sends the selects and
//! the map picker's coordinates as text (`"20"`, `spots.0.lat: "43.5"`); the readers below accept
//! that and numbers alike. The old KV blob went through [`legacy`].

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// The rows of the host form, blank ones included (see [`Self::parse_spots`]).
    #[field(structured, required, label = "config.spots")]
    pub spots: Vec<SpotRow>,
    #[field(label = "host.disclaimer.label")]
    pub disclaimer: I18nText,
    #[field(label = "host.activities.enabled")]
    pub activities_enabled: bool,
    #[field(label = "host.activities.destination")]
    pub activities_destination: String,
    #[field(label = "host.activities.intro")]
    pub activities_intro: I18nText,
    /// The links of the « Activités & billets » section, as the form rows `{ url, label }`.
    #[field(structured, label = "host.section.activities")]
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
            disclaimer: I18nText::default(),
            activities_enabled: false,
            activities_destination: String::new(),
            activities_intro: I18nText::default(),
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

/// Notes minimales proposées à l'hôte ; 0 = aucun filtre.
const TIQETS_MIN_RATING_CHOICES: [u32; 3] = [0, 3, 4];

/// The old KV blob onto the declared keys: the nested `activities: {enabled, destination, intro,
/// links}` and `tiqets: {enabled, radius_km, min_rating}` sections go flat (numbers to their
/// select option), `spots_json` (a JSON string, before the list) becomes `spots` when there is no
/// list, and a spot's `null` texts go (a translated text is an object or nothing).
fn legacy(old: Value) -> Value {
    let Value::Object(mut old) = old else {
        return old;
    };
    if let Some(Value::Object(mut section)) = old.remove("activities") {
        for (from, to) in [
            ("enabled", "activities_enabled"),
            ("destination", "activities_destination"),
            ("intro", "activities_intro"),
            ("links", "activities"),
        ] {
            if let Some(value) = section.remove(from) {
                old.insert(to.into(), value);
            }
        }
    }
    if let Some(Value::Object(section)) = old.remove("tiqets") {
        if let Some(enabled) = section.get("enabled") {
            old.insert("tiqets_enabled".into(), enabled.clone());
        }
        for (from, to, choices) in [
            (
                "radius_km",
                "tiqets_radius_km",
                &TIQETS_RADIUS_CHOICES_KM[..],
            ),
            (
                "min_rating",
                "tiqets_min_rating",
                &TIQETS_MIN_RATING_CHOICES[..],
            ),
        ] {
            if let Some(n) = section.get(from).and_then(Value::as_u64) {
                let nearest = choices
                    .iter()
                    .min_by_key(|choice| u64::from(**choice).abs_diff(n))
                    .expect("choices");
                old.insert(to.into(), Value::from(nearest.to_string()));
            }
        }
    }
    let listed = old
        .remove("spots_json")
        .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
    let no_spots = old
        .get("spots")
        .is_none_or(|spots| spots.as_array().is_some_and(Vec::is_empty));
    if let (Some(spots), true) = (listed, no_spots) {
        old.insert("spots".into(), spots);
    }
    if let Some(Value::Array(spots)) = old.get_mut("spots") {
        for spot in spots.iter_mut().filter_map(Value::as_object_mut) {
            spot.retain(|_, value| !value.is_null());
        }
    }
    Value::Object(old)
}

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActivitiesConfig {
    /// Affiche la section dans le livret. Éteinte tant que l'hôte ne l'allume pas.
    pub enabled: bool,
    /// Ville visée, quand celle de l'adresse ne convient pas (« Antibes »).
    pub destination: String,
    /// Phrase d'introduction affichée au-dessus des liens.
    pub intro: I18nText,
    /// Liens choisis par l'hôte, dans son ordre.
    pub links: Vec<ActivityRow>,
}

/// Un lien choisi par l'hôte. Le formulaire envoie `url` et `label` (et `id`) ; la plateforme
/// garde les autres langues du libellé.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ActivityRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub url: String,
    pub label: I18nText,
}

impl ActivityRow {
    /// Rien de saisi : un créneau laissé (ou vidé) par l'hôte.
    pub fn is_blank(&self) -> bool {
        self.url.trim().is_empty() && self.label.is_blank()
    }
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_spots().is_empty() && self.disclaimer.is_blank()
    }

    /// The named rows, for the guest, in form order. A row without an id takes its slot's
    /// (`spot-1`…), as the module's own `updateConfig` used to give it.
    pub fn parse_spots(&self) -> Vec<SpotRow> {
        self.spots
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.title.is_blank())
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

/// A place. The form sends `title`, `category`, `distance`, `tag`, `detail` and the map picker's
/// `address`, `lat`, `lng` (and `id`); the platform keeps the rest — `url`, `note`, the texts'
/// other languages.
///
/// `PartialEq` sans `Eq` : `lat` et `lng` sont des `f64`, qui n'ont pas d'égalité totale.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SpotRow {
    pub id: String,
    pub title: I18nText,
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub url: Option<String>,
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub category: Option<String>,
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub distance: Option<String>,
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub tag: Option<String>,
    pub note: Option<I18nText>,
    pub detail: I18nText,
    /// Adresse postale telle que le sélecteur de carte l'a géocodée.
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub address: Option<String>,
    /// Latitude WGS-84, absente tant que l'hôte n'a pas posé le lieu sur la carte. Le
    /// sélecteur de carte l'envoie en texte (`"43.5"`, `""`).
    #[serde(deserialize_with = "deserialize_coord")]
    pub lat: Option<f64>,
    /// Longitude WGS-84.
    #[serde(deserialize_with = "deserialize_coord")]
    pub lng: Option<f64>,
}

impl SpotRow {
    /// Position affichable du spot, ou `None`.
    pub fn coords(&self) -> Option<(f64, f64)> {
        valid_coords(self.lat?, self.lng?)
    }

    /// Nothing stored but the id: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.detail.is_blank()
            && self.note.as_ref().is_none_or(I18nText::is_blank)
            && [
                &self.url,
                &self.category,
                &self.distance,
                &self.tag,
                &self.address,
            ]
            .iter()
            .all(|text| text.is_none())
            && self.coords().is_none()
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
        let empty = json!({ "title": "", "category": "", "distance": "", "tag": "",
            "detail": "", "address": "", "lat": "0", "lng": "0" });
        let mut spots = vec![empty; 6];
        spots[2] = json!({ "title": "Plage du Midi", "category": "Plage", "distance": " ",
            "tag": "", "detail": "Sable fin", "address": "Bd du Midi, Cannes",
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
        assert!(config.spots[0].is_blank());
        assert!(!config.spots[2].is_blank());
        let parsed = config.parse_spots();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "spot-3");
        assert_eq!(parsed[0].title.get("en"), "Plage du Midi");
        assert_eq!(parsed[0].detail.get("fr"), "Sable fin");
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
            "spots_json": r#"[{"id":"s1","title":{"fr":"Plage"},"detail":null,"note":null,"url":null}]"#,
            "disclaimer": { "fr": "Suggestions", "en": "Suggestions (en)" },
            "activities": {
                "enabled": true, "destination": "Antibes", "intro": { "fr": "Bonjour" },
                "links": [{ "url": "https://gyg.me/aBcD12", "label": { "fr": "Suquet" } }]
            },
            "tiqets": { "enabled": true, "radius_km": 40, "min_rating": 3 }
        })
    }

    /// Every shape the old KV blob had: nested `activities` / `tiqets` sections (numbers for the
    /// selects), a `{fr, en}` disclaimer, the spots as a `spots_json` string, `null` texts.
    #[test]
    fn legacy_maps_the_kv_blob() {
        let mapped = legacy(legacy_blob());
        assert_eq!(
            mapped,
            json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" } }],
                "disclaimer": { "fr": "Suggestions", "en": "Suggestions (en)" },
                "activities_enabled": true,
                "activities_destination": "Antibes",
                "activities_intro": { "fr": "Bonjour" },
                "activities": [{ "url": "https://gyg.me/aBcD12", "label": { "fr": "Suquet" } }],
                "tiqets_enabled": true,
                "tiqets_radius_km": "40",
                "tiqets_min_rating": "3"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
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
    }

    #[test]
    fn legacy_spots_rows_win_over_spots_json_and_lose_their_nulls() {
        let mapped = legacy(json!({
            "spots": [{ "id": "s2", "title": { "fr": "Port", "en": "Harbour" }, "detail": null,
                        "note": { "fr": "Gratuit" }, "lat": 43.5, "lng": 7.0, "url": null }],
            "spots_json": r#"[{"id":"s1","title":{"fr":"Plage"}}]"#
        }));
        assert_eq!(
            mapped,
            json!({ "spots": [{ "id": "s2", "title": { "fr": "Port", "en": "Harbour" },
                                "note": { "fr": "Gratuit" }, "lat": 43.5, "lng": 7.0 }] })
        );
        let spot = &serde_json::from_value::<ModuleConfig>(mapped)
            .unwrap()
            .spots[0];
        assert_eq!(spot.note.as_ref().unwrap().get("en"), "Gratuit");
        assert_eq!(spot.coords(), Some((43.5, 7.0)));
        // An empty list is no list: `spots_json` fills it.
        assert_eq!(
            legacy(json!({ "spots": [], "spots_json": r#"[{"id":"s1"}]"# })),
            json!({ "spots": [{ "id": "s1" }] })
        );
        // Unreadable: nothing to import, nothing invented.
        assert_eq!(legacy(json!({ "spots_json": "[oops" })), json!({}));
    }

    #[test]
    fn legacy_snaps_a_number_off_the_select_to_the_nearest_choice() {
        let mapped = legacy(json!({ "tiqets": { "radius_km": 100, "min_rating": 5 } }));
        assert_eq!(mapped["tiqets_radius_km"], "40");
        assert_eq!(mapped["tiqets_min_rating"], "4");
        assert!(mapped.get("tiqets_enabled").is_none());
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&legacy_blob()).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert!(config.activities().enabled);
                assert_eq!(config.tiqets().radius_km, 40);
                assert_eq!(config.parse_spots().len(), 1);
                assert_eq!(config.disclaimer.get("en"), "Suggestions (en)");
            });
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&legacy_blob()).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }
}
