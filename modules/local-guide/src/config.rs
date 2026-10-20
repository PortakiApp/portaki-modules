//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself and
//! stores them flat (`activities_enabled`, `tiqets_radius_km`…). The form sends the selects and
//! the map picker's coordinates as text (`"20"`, `spots.0.lat: "43.5"`); the readers below accept
//! that and numbers alike. The old KV blob went through [`legacy`].

use chrono::{Datelike, NaiveDate, Weekday};
use portaki_sdk::bundle_text;
use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::sdui::hours;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// The rows of the host form, blank ones included (see [`Self::parse_spots`]).
    ///
    /// Copiées vers un autre logement, marquées « À vérifier » (`local`) — catégories, conseils et
    /// avantages compris ; fusionnées ligne par ligne sur leur `id`, dans l'ordre reçu : le tableau
    /// se réordonne. Les distances ne sont jamais stockées, elles se recalculent.
    #[field(structured, required, local, label = "config.spots")]
    pub spots: Vec<SpotRow>,
    /// Les activités que l'hôte propose lui-même (§2.13, `origin: host`).
    ///
    /// Rien à voir avec les liens partenaires de « Activités & billets » : ici l'hôte décrit ce
    /// qu'il connaît — la sortie voilier de Marc —, la réservation se fait avec le prestataire,
    /// et aucune commission n'est perçue.
    #[field(structured, local, label = "config.hostActivities")]
    pub host_activities: Vec<HostActivityRow>,
    #[field(label = "host.disclaimer.label")]
    pub disclaimer: I18nText,
    #[field(label = "host.activities.enabled")]
    pub activities_enabled: bool,
    #[field(label = "host.activities.destination")]
    pub activities_destination: String,
    #[field(label = "host.activities.intro")]
    pub activities_intro: I18nText,
    /// « Vos choix dans le catalogue » : les liens collés par l'hôte, `{ url, label, tip }`, cinq
    /// en avant au plus. Copiés « À vérifier » (`local`).
    #[field(structured, local, label = "host.section.activities")]
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
    #[field(label = "host.viator.enabled")]
    pub viator_enabled: bool,
    #[field(kind = "select", options = ["0", "3", "4"], label = "host.viator.minRating")]
    #[serde(deserialize_with = "deserialize_viator_min_rating")]
    pub viator_min_rating: u8,
    /// Le bloc « Adresses » de la page publique du logement. Éteint par défaut.
    #[field(label = "host.public.enabled")]
    pub public_enabled: bool,
    /// Les adresses de la page publique, 3 à 6, par identifiant, dans l'ordre choisi.
    ///
    /// À plat et en `structured` : le choix multiple envoie un vrai tableau, qu'un champ `text`
    /// ferait refuser par la plateforme.
    #[field(kind = "structured", label = "host.public.spots")]
    #[serde(deserialize_with = "id_list")]
    pub public_spots: Vec<String>,
    // §2.2 Activités des partenaires. Absents tant que l'hôte n'a pas enregistré la nouvelle
    // carte : ils retombent alors sur les réglages d'avant, fournisseur par fournisseur.
    /// « Afficher le catalogue ». Absent : l'un des anciens interrupteurs allumé.
    #[field(label = "host.partners.enabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partners_enabled: Option<bool>,
    /// Les fournisseurs choisis, [`PROVIDERS`]. Absent : ceux que les anciens interrupteurs
    /// allumaient.
    #[field(kind = "structured", label = "host.partners.providers")]
    #[serde(
        default,
        deserialize_with = "optional_id_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub providers: Option<Vec<String>>,
    /// Un seul rayon pour tous les fournisseurs, 1 à 50 km. Absent : celui de Tiqets d'avant.
    #[field(label = "host.partners.radius")]
    #[serde(
        default,
        deserialize_with = "optional_number",
        skip_serializing_if = "Option::is_none"
    )]
    pub partner_radius_km: Option<u32>,
    /// Prix maximum, en euros, 1 à 1 000. Absent : pas de filtre.
    #[field(label = "host.partners.maxPrice")]
    #[serde(
        default,
        deserialize_with = "optional_number",
        skip_serializing_if = "Option::is_none"
    )]
    pub partner_max_price: Option<u32>,
    /// Note minimale commune. Absente : celle de chaque fournisseur d'avant.
    #[field(kind = "select", options = ["0", "3", "4"], label = "host.partners.minRating")]
    #[serde(
        default,
        deserialize_with = "optional_number",
        skip_serializing_if = "Option::is_none"
    )]
    pub partner_min_rating: Option<u8>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            spots: Vec::new(),
            host_activities: Vec::new(),
            disclaimer: I18nText::default(),
            activities_enabled: false,
            activities_destination: String::new(),
            activities_intro: I18nText::default(),
            activities: Vec::new(),
            tiqets_enabled: false,
            tiqets_radius_km: TIQETS_DEFAULT_RADIUS_KM,
            tiqets_min_rating: 0,
            viator_enabled: false,
            viator_min_rating: VIATOR_DEFAULT_MIN_RATING,
            public_enabled: false,
            public_spots: Vec::new(),
            partners_enabled: None,
            providers: None,
            partner_radius_km: None,
            partner_max_price: None,
            partner_min_rating: None,
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

/// Note minimale Viator par défaut : la liste suit la ville entière, le filtre écarte le tout-venant.
pub const VIATOR_DEFAULT_MIN_RATING: u8 = 4;

/// Les fournisseurs du catalogue que l'hôte peut choisir (§2.2). GetYourGuide est annoncé, pas
/// proposé.
pub const PROVIDERS: [&str; 2] = ["viator", "tiqets"];

/// Bornes du rayon commun et du prix maximum (§2.2).
pub const PARTNER_RADIUS_KM: (u32, u32) = (1, 50);
pub const PARTNER_MAX_PRICE: (u32, u32) = (1, 1000);

/// Les catégories d'une adresse (§2.1), stockées en toutes lettres : l'emoji par défaut et la page
/// publique les lisent telles quelles. Le jeton sert aux libellés traduits du formulaire.
pub const CATEGORIES: [(&str, &str); 9] = [
    ("restaurant", "Restaurant"),
    ("cafe", "Café"),
    ("shop", "Commerce"),
    ("market", "Marché"),
    ("beach", "Plage"),
    ("culture", "Culture"),
    ("sport", "Sport"),
    ("wellness", "Bien-être"),
    ("other", "Autre"),
];

/// L'ordre de prix proposé (§2.1) ; l'ancien texte libre reste lisible.
pub const PRICE_LEVELS: [&str; 4] = ["€", "€€", "€€€", "€€€€"];

/// Au-delà, l'adresse est sans doute mal placée (§2.1, position ≤ 50 km) : un avertissement.
pub const SPOT_MAX_KM: f64 = 50.0;

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
    /// Prix maximum en euros ; `None` = aucun filtre.
    #[serde(default)]
    pub max_price: Option<u32>,
}

impl Default for TiqetsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            radius_km: TIQETS_DEFAULT_RADIUS_KM,
            min_rating: 0,
            max_price: None,
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

/// Configuration de la section Viator.
///
/// **Éteinte par défaut**, comme les sections GetYourGuide et Tiqets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViatorConfig {
    pub enabled: bool,
    /// Note minimale des avis, 1 à 5 ; 0 = aucun filtre.
    pub min_rating: u8,
    /// Prix maximum en euros ; `None` = aucun filtre.
    pub max_price: Option<u32>,
}

impl Default for ViatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_rating: VIATOR_DEFAULT_MIN_RATING,
            max_price: None,
        }
    }
}

/// Sous le prix maximum, ou sans prix connu : un produit sans prix n'est pas « trop cher ».
pub fn within_price(price: Option<f64>, max: Option<u32>) -> bool {
    match (price, max) {
        (Some(price), Some(max)) => price <= f64::from(max),
        _ => true,
    }
}

impl ViatorConfig {
    /// 0 = pas de filtre, sinon 1 à 5 : écrite hors du formulaire, elle ne doit pas devenir un `400`.
    pub fn normalized_min_rating(&self) -> u8 {
        self.min_rating.min(5)
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
    /// Le lien de recherche GetYourGuide de la destination, en tête : seulement pour qui avait
    /// allumé l'ancienne section. Sans lui, la section ne montre que les choix de l'hôte.
    pub search: bool,
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
    /// Le conseil que l'hôte ajoute à un lien collé (§2.13) — la seule chose qu'un fournisseur
    /// n'écrira jamais. Vide quand il n'en a pas mis.
    #[serde(default)]
    pub tip: I18nText,
}

impl ActivityRow {
    /// Rien de saisi : un créneau laissé (ou vidé) par l'hôte.
    pub fn is_blank(&self) -> bool {
        self.url.trim().is_empty() && self.label.is_blank()
    }

    /// Ce qui ne va pas dans ce lien, par sous-clé ; rien pour une ligne vide.
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        let refused = matches!(
            crate::affiliate::normalize_curated_url(&self.url),
            Err(crate::affiliate::CuratedUrlError::NotGetYourGuide)
        );
        [
            (
                "url",
                refused.then(|| bundle_text!("host.activities.error.badUrl")),
            ),
            ("tip", too_long(&self.tip, 200)),
        ]
        .into_iter()
        .filter_map(|(key, error)| Some((key, error?)))
        .collect()
    }
}

/// Le premier texte trop long d'un texte traduit, dans n'importe quelle langue.
fn too_long(value: &I18nText, max: usize) -> Option<I18nText> {
    value
        .by_language()
        .find_map(|(_, text)| check::max_chars(text, max))
}

fn plain_too_long(value: &Option<String>, max: usize) -> Option<I18nText> {
    value
        .as_deref()
        .and_then(|text| check::max_chars(text, max))
}

/// Combien de bons plans la configuration accepte (spec Bons plans §2.1).
pub const MAX_SPOTS: usize = 30;

/// Combien d'adresses la page publique montre : en dessous, le bloc ne sort pas ; au-delà, les
/// premières dans l'ordre du formulaire.
pub const PUBLIC_MIN: usize = 3;
pub const PUBLIC_MAX: usize = 6;

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_spots().is_empty() && self.disclaimer.is_blank()
    }

    /// Ce qui ne va pas, champ par champ (`spots.<i>.title`…) — sous le champ dans le formulaire,
    /// et dans `publishReadiness`. La position manquante n'y est pas : elle avertit sans bloquer.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        let filled = self.spots.iter().filter(|s| !s.is_blank()).count();
        if filled > MAX_SPOTS {
            problems.push(("spots".into(), bundle_text!("host.spots.tooMany")));
        }
        for (index, spot) in self.spots.iter().enumerate() {
            for (key, error) in spot.problems() {
                problems.push((format!("spots.{index}.{key}"), error));
            }
        }
        let activities = self
            .host_activities
            .iter()
            .filter(|a| !a.is_blank())
            .count();
        if activities > MAX_HOST_ACTIVITIES {
            problems.push((
                "host_activities".into(),
                bundle_text!("host.hostActivities.tooMany"),
            ));
        }
        for (index, activity) in self.host_activities.iter().enumerate() {
            for (key, error) in activity.problems() {
                problems.push((format!("host_activities.{index}.{key}"), error));
            }
        }
        if self.partners_on() {
            // Sans fournisseur choisi, rien ne s'afficherait — sauf l'ancienne recherche
            // GetYourGuide d'un hôte qui l'avait allumée : elle tient le catalogue à elle seule.
            if self.providers.as_ref().is_some_and(Vec::is_empty) && !self.activities_enabled {
                problems.push((
                    "providers".into(),
                    bundle_text!("host.partners.providers.required"),
                ));
            }
            let (min, max) = PARTNER_RADIUS_KM;
            if self
                .partner_radius_km
                .is_some_and(|km| !(min..=max).contains(&km))
            {
                problems.push((
                    "partner_radius_km".into(),
                    bundle_text!("host.partners.radius.range"),
                ));
            }
            let (min, max) = PARTNER_MAX_PRICE;
            if self
                .partner_max_price
                .is_some_and(|price| !(min..=max).contains(&price))
            {
                problems.push((
                    "partner_max_price".into(),
                    bundle_text!("host.partners.maxPrice.range"),
                ));
            }
            let links = self.activities.iter().filter(|a| !a.is_blank()).count();
            if links > crate::affiliate::MAX_CURATED_LINKS {
                problems.push((
                    "activities".into(),
                    bundle_text!("host.activities.error.tooMany"),
                ));
            }
            for (index, link) in self.activities.iter().enumerate() {
                for (key, error) in link.problems() {
                    problems.push((format!("activities.{index}.{key}"), error));
                }
            }
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
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

    /// Les adresses que le livret montre le jour `today` : celles de [`Self::parse_spots`], moins
    /// celles hors saison (§3). Sans date, toutes : mieux vaut montrer une plage en hiver que de
    /// masquer un restaurant un jour deviné.
    pub fn guest_spots(&self, today: Option<NaiveDate>) -> Vec<SpotRow> {
        let mut spots = self.parse_spots();
        if let Some(today) = today {
            spots.retain(|spot| spot.in_season(today));
        }
        spots
    }

    /// Les adresses publiées choisies pour la page publique, dans l'ordre du choix. Un id qui
    /// n'est plus celui d'une adresse publiée ne compte pas.
    pub fn public_chosen(&self) -> Vec<SpotRow> {
        let spots = self.parse_spots();
        let mut out: Vec<SpotRow> = Vec::new();
        for id in &self.public_spots {
            if out.iter().any(|spot| &spot.id == id) {
                continue;
            }
            if let Some(spot) = spots.iter().find(|spot| &spot.id == id) {
                out.push(spot.clone());
            }
        }
        out
    }

    /// Ce que la page publique montre : rien quand le bloc est éteint ou qu'il manque des
    /// adresses, sinon les [`PUBLIC_MAX`] premières cochées.
    pub fn public_spots(&self) -> Vec<SpotRow> {
        if !self.public_enabled {
            return Vec::new();
        }
        let chosen = self.public_chosen();
        if chosen.len() < PUBLIC_MIN {
            return Vec::new();
        }
        chosen.into_iter().take(PUBLIC_MAX).collect()
    }

    /// The « Activités & billets » section. Links and destination are the host's raw input:
    /// `activities::resolve` normalizes them (GetYourGuide only, partner id) at render.
    ///
    /// Allumée avec le catalogue, dès qu'il y a un choix de l'hôte — ou que l'ancienne section
    /// l'était : son lien de recherche reste alors en tête.
    pub fn activities(&self) -> ActivitiesConfig {
        let links: Vec<ActivityRow> = self
            .activities
            .iter()
            .filter(|row| !row.url.trim().is_empty())
            .cloned()
            .collect();
        let on = self.partners_on();
        ActivitiesConfig {
            enabled: on && (self.activities_enabled || !links.is_empty()),
            search: on && self.activities_enabled,
            destination: self.activities_destination.trim().to_string(),
            intro: self.activities_intro.clone(),
            links,
        }
    }

    /// « Afficher le catalogue » ; avant la carte, l'un des anciens interrupteurs allumé.
    pub fn partners_on(&self) -> bool {
        self.partners_enabled
            .unwrap_or(self.tiqets_enabled || self.viator_enabled || self.activities_enabled)
    }

    /// Les fournisseurs choisis, dans l'ordre de [`PROVIDERS`] ; avant la carte, ceux que les
    /// anciens interrupteurs allumaient.
    pub fn providers(&self) -> Vec<&'static str> {
        PROVIDERS
            .into_iter()
            .filter(|provider| match &self.providers {
                Some(chosen) => chosen.iter().any(|c| c == provider),
                None => match *provider {
                    "viator" => self.viator_enabled,
                    _ => self.tiqets_enabled,
                },
            })
            .collect()
    }

    fn provider_on(&self, provider: &str) -> bool {
        self.partners_on() && self.providers().contains(&provider)
    }

    /// Le rayon commun ; avant la carte, celui de Tiqets.
    pub fn partner_radius(&self) -> u32 {
        self.partner_radius_km.unwrap_or(self.tiqets_radius_km)
    }

    pub fn tiqets(&self) -> TiqetsConfig {
        TiqetsConfig {
            enabled: self.provider_on("tiqets"),
            radius_km: self.partner_radius(),
            min_rating: self.partner_min_rating.unwrap_or(self.tiqets_min_rating),
            max_price: self.partner_max_price,
        }
    }

    pub fn viator(&self) -> ViatorConfig {
        ViatorConfig {
            enabled: self.provider_on("viator"),
            min_rating: self.partner_min_rating.unwrap_or(self.viator_min_rating),
            max_price: self.partner_max_price,
        }
    }
}

/// Une activité que l'hôte décrit lui-même (§2.13, `origin: host`).
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HostActivityRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Le nom de l'activité — « Sortie voilier avec Marc ».
    pub title: I18nText,
    /// Le prestataire, nom et rôle : « Marc, skipper au port ». C'est lui qu'on réserve, pas
    /// l'hôte ; sans lui, l'activité ne sort pas.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub provider: Option<String>,
    /// Le prix, écrit tel quel : « 60 € / pers. ». Un prix fixe, pas un « dès » — l'hôte connaît
    /// le tarif de son ami, le fournisseur partenaire annonce un plancher.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub price: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub duration: Option<String>,
    /// Les conditions d'annulation, en une phrase.
    #[serde(default)]
    pub cancel: I18nText,
    /// Le point de rendez-vous : « Port Vauban, ponton C ».
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub meet: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub languages: Option<String>,
    /// Ce qui est inclus, une ligne par élément.
    #[serde(default)]
    pub included: I18nText,
    /// Le conseil de l'hôte, celui qu'aucun fournisseur n'écrira.
    #[serde(default)]
    pub tip: I18nText,
    /// Le téléphone de réservation, en forme internationale.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub phone: Option<String>,
    /// Le site du prestataire, quand il en a un.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub url: Option<String>,
    /// La photo déposée par l'hôte, en référence `portaki-file:`.
    #[serde(default)]
    pub photo: String,
}

impl HostActivityRow {
    /// L'identifiant de route de cette activité, pour `local-guide/activity/:activityId`.
    pub fn route_id(&self, index: usize) -> String {
        let id = self.id.trim();
        if id.is_empty() {
            format!("a{index}")
        } else {
            id.to_string()
        }
    }

    /// De quoi s'afficher : un nom et un prestataire. Sans prestataire, on ne saurait pas qui
    /// réserver, et la fiche promettrait une activité sans porte d'entrée.
    pub fn is_complete(&self, locale: &str) -> bool {
        !self.title.get(locale).trim().is_empty() && self.provider.is_some()
    }

    /// Rien que le formulaire montre : une ligne que l'hôte a laissée (ou vidée).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank() && self.provider.is_none() && self.photo.trim().is_empty()
    }

    /// Ce qui ne va pas dans cette activité, par sous-clé ; rien pour une ligne vide.
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        [
            ("provider", plain_too_long(&self.provider, 60)),
            ("duration", plain_too_long(&self.duration, 30)),
            ("phone", self.phone.as_deref().and_then(phone_error)),
            ("url", self.url.as_deref().and_then(check::https_url)),
        ]
        .into_iter()
        .filter_map(|(key, error)| Some((key, error?)))
        .collect()
    }

    /// La photo déposée, en référence, ou `None`.
    pub fn photo_ref(&self) -> Option<&str> {
        let photo = self.photo.trim();
        (!photo.is_empty()).then_some(photo)
    }

    /// Ce qui est inclus, une ligne par élément, dix au plus.
    pub fn included_lines(&self, locale: &str) -> Vec<String> {
        self.included
            .get(locale)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(MAX_INCLUDED_LINES)
            .map(String::from)
            .collect()
    }
}

/// Au-delà, « Inclus » devient une liste qu'on ne lit plus.
pub const MAX_INCLUDED_LINES: usize = 10;

/// Les activités que l'hôte décrit lui-même : six lignes, comme les autres listes du module.
pub const MAX_HOST_ACTIVITIES: usize = 6;

/// Photos par fiche, côté hôte comme côté fournisseur (§2.12, §2.13).
///
/// Cinq, la borne du PROMPT : au-delà, on fait défiler un album au lieu de choisir une adresse.
pub const MAX_GALLERY: usize = 5;

/// Les sept jours, dans l'ordre de la semaine : le jeton stocké et le jour qu'il désigne.
///
/// Une liste figée plutôt qu'un analyseur : le formulaire n'envoie que ces jetons, et tout le
/// reste doit rester sans effet.
pub const WEEKDAYS: [(&str, Weekday); 7] = [
    ("mon", Weekday::Mon),
    ("tue", Weekday::Tue),
    ("wed", Weekday::Wed),
    ("thu", Weekday::Thu),
    ("fri", Weekday::Fri),
    ("sat", Weekday::Sat),
    ("sun", Weekday::Sun),
];

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
    /// L'avantage, écrit en entier : « 10 % sur le menu du soir, sur présentation de cette page. »
    /// Le `tag` en est la version courte, celle du badge ; celui-ci est la phrase de la fiche.
    #[serde(default)]
    pub perk: I18nText,
    /// L'ordre de prix, tel que l'hôte l'écrit : « €€€ », « dès 18 € / jour », « transat 25 € ».
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub price: Option<String>,
    /// Les horaires, en une ligne : « 12:00 – 14:30 · 19:00 – 22:30 ».
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub hours: Option<String>,
    /// Le jour de fermeture hebdomadaire, `mon` … `sun`. Vide quand l'adresse ouvre tous les jours.
    ///
    /// ponytail: un seul jour, là où certains commerces en ferment deux. Un deuxième créneau par
    /// adresse alourdirait un formulaire qui porte déjà quinze champs ; à ajouter si des hôtes le
    /// demandent. `opening` reste la phrase libre, et rien ici ne la réécrit.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub closed_day: Option<String>,
    /// Les jours d'ouverture : « Fermé le lundi », « D'avril à octobre ».
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub opening: Option<String>,
    /// Où se garer : « Parking du port », « Stationnement minute devant ».
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub parking: Option<String>,
    /// Le téléphone, en forme internationale — le bouton « Appeler » s'en sert tel quel.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub phone: Option<String>,
    /// La première photo, telle que le formulaire l'envoyait avant la galerie.
    ///
    /// Gardée en lecture : les configurations écrites avant [`SpotRow::photos`] la portent, et
    /// [`SpotRow::photo_refs`] les réunit pour que le rendu n'ait qu'une source.
    #[serde(default)]
    pub photo: String,
    /// Les photos de la fiche, dans l'ordre de l'hôte, cinq au plus (§2.12).
    ///
    /// Le formulaire n'en dessine qu'une de plus que celles déjà déposées : une rangée de cinq
    /// champs vides par adresse, sur douze adresses, serait un formulaire qu'on ne lit plus.
    #[serde(default)]
    pub photos: Vec<String>,
    /// L'emoji de la tuile et du repère ; vide, celui de la catégorie ([`SpotRow::emoji`]).
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub emoji: Option<String>,
    /// Le début de la saison, `JJ/MM`. Vide, avec [`SpotRow::season_to`] : toute l'année.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub season_from: Option<String>,
    /// La fin de la saison, `JJ/MM`, incluse. Avant le début : la saison passe le nouvel an.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub season_to: Option<String>,
    /// « Réservation indispensable en août » : le bandeau d'avertissement de la fiche.
    #[serde(default)]
    pub warning: I18nText,
    /// « J'ai un accord avec ce lieu » : un avantage sans accord avertit, sans bloquer (§3).
    #[serde(default)]
    pub perk_agreement: bool,
    /// Les horaires par jour, la valeur d'un `WeeklyHours` (`mon=12:00-14:00,19:00-22:00;…`).
    /// Vide : les anciens `hours` / `closed_day` valent toujours.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub weekly_hours: String,
}

/// L'emoji d'une catégorie de la spec (§2.1), quand l'hôte n'en a pas choisi. Comparé sans
/// casse : la catégorie est encore une saisie libre.
const CATEGORY_EMOJIS: [(&str, &str); 9] = [
    ("restaurant", "🍽️"),
    ("café", "☕"),
    ("commerce", "🛍️"),
    ("marché", "🧺"),
    ("plage", "🏖️"),
    ("culture", "🏛️"),
    ("sport", "⚽"),
    ("bien-être", "🧘"),
    ("autre", "📍"),
];

/// `JJ/MM` en rang dans une année bissextile (le 29/02 existe), ou `None` si la date ne se lit pas.
fn day_of_year(day: &str) -> Option<u32> {
    NaiveDate::parse_from_str(&format!("{}/2000", day.trim()), "%d/%m/%Y")
        .ok()
        .map(|date| date.ordinal())
}

impl SpotRow {
    /// L'emoji choisi, sinon celui de la catégorie, sinon rien.
    pub fn emoji(&self) -> Option<String> {
        if let Some(emoji) = &self.emoji {
            return Some(emoji.clone());
        }
        let category = self.category.as_deref()?.trim().to_lowercase();
        CATEGORY_EMOJIS
            .iter()
            .find(|(name, _)| *name == category)
            .map(|(_, emoji)| emoji.to_string())
    }

    /// Ouverte ce jour-là. Toute l'année quand une borne manque ou ne se lit pas : une saison
    /// mal saisie ne doit pas faire disparaître l'adresse (l'erreur est sous le champ).
    pub fn in_season(&self, today: NaiveDate) -> bool {
        let bound = |day: &Option<String>| day.as_deref().and_then(day_of_year);
        let (Some(from), Some(to)) = (bound(&self.season_from), bound(&self.season_to)) else {
            return true;
        };
        // Le même rang bissextile pour aujourd'hui, pour comparer des jours et non des années.
        let Some(today) = day_of_year(&format!("{:02}/{:02}", today.day(), today.month())) else {
            return true;
        };
        if from <= to {
            (from..=to).contains(&today)
        } else {
            today >= from || today <= to
        }
    }

    /// L'identifiant de route de cette adresse, pour `local-guide/:spotId`.
    pub fn route_id(&self, index: usize) -> String {
        let id = self.id.trim();
        if id.is_empty() {
            format!("s{index}")
        } else {
            id.to_string()
        }
    }

    /// La photo de couverture : la première de la galerie.
    pub fn photo_ref(&self) -> Option<&str> {
        self.photo_refs().into_iter().next()
    }

    /// Toutes les photos, l'ancienne d'abord, sans doublon ni vide, cinq au plus.
    ///
    /// Une seule source pour le rendu : sans ça, chaque appelant devait se souvenir que `photo`
    /// existe encore à côté de `photos`, et l'un l'aurait oublié.
    pub fn photo_refs(&self) -> Vec<&str> {
        let mut refs: Vec<&str> = Vec::new();
        for candidate in
            std::iter::once(self.photo.as_str()).chain(self.photos.iter().map(String::as_str))
        {
            let candidate = candidate.trim();
            if !candidate.is_empty() && !refs.contains(&candidate) {
                refs.push(candidate);
            }
            if refs.len() == MAX_GALLERY {
                break;
            }
        }
        refs
    }

    /// Position affichable du spot, ou `None`.
    pub fn coords(&self) -> Option<(f64, f64)> {
        valid_coords(self.lat?, self.lng?)
    }

    /// Ce qui ne va pas dans cette adresse, par sous-clé ; rien pour une ligne vide. La position
    /// manquante n'y est pas : elle avertit sans bloquer (`publishReadiness`).
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        let season = [&self.season_from, &self.season_to]
            .into_iter()
            .any(|day| day.as_deref().is_some_and(|d| day_of_year(d).is_none()))
            .then(|| bundle_text!("host.spot.season.invalid"));
        let note = self.note.as_ref().and_then(|note| too_long(note, 280));
        [
            (
                "title",
                if self.title.is_blank() {
                    Some(bundle_text!("host.spot.title.required"))
                } else {
                    too_long(&self.title, 60)
                },
            ),
            (
                "category",
                self.category
                    .is_none()
                    .then(|| bundle_text!("host.spot.category.required")),
            ),
            ("perk", too_long(&self.perk, 120)),
            ("note", note),
            ("warning", too_long(&self.warning, 120)),
            ("season_from", season),
            ("detail", too_long(&self.detail, 280)),
            ("price", plain_too_long(&self.price, 30)),
            ("parking", plain_too_long(&self.parking, 80)),
            ("phone", self.phone.as_deref().and_then(phone_error)),
            ("url", self.url.as_deref().and_then(check::https_url)),
        ]
        .into_iter()
        .filter_map(|(key, error)| Some((key, error?)))
        .collect()
    }

    /// Un avantage annoncé sans « J'ai un accord avec ce lieu » (§3) : un avertissement.
    pub fn perk_without_agreement(&self) -> bool {
        !self.perk.is_blank() && !self.perk_agreement
    }

    /// Le jour de fermeture, s'il est l'un des sept : un jeton hors de la liste ne ferme rien
    /// plutôt que de fermer un jour deviné.
    pub fn closed_weekday(&self) -> Option<Weekday> {
        let raw = self.closed_day.as_deref()?.trim().to_ascii_lowercase();
        WEEKDAYS
            .iter()
            .find(|(token, _)| *token == raw)
            .map(|(_, day)| *day)
    }

    /// Fermé le jour demandé : sans plage ce jour-là dans [`SpotRow::weekly_hours`] quand il est
    /// rempli, sinon l'ancien jour de fermeture.
    pub fn closed_on(&self, day: Weekday) -> bool {
        match self.week() {
            Some(week) => week[day.num_days_from_monday() as usize].is_empty(),
            None => self.closed_weekday() == Some(day),
        }
    }

    /// Les plages du jour demandé, « 12:00 – 14:00, 19:00 – 22:00 », quand les horaires par jour
    /// sont remplis et que ce jour-là en a.
    pub fn hours_on(&self, day: Weekday) -> Option<String> {
        let ranges = &self.week()?[day.num_days_from_monday() as usize];
        (!ranges.is_empty()).then(|| {
            ranges
                .iter()
                .map(|(from, to)| format!("{from} – {to}"))
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    /// L'hôte a saisi des horaires par jour : ils remplacent l'ancienne ligne libre.
    pub fn has_weekly_hours(&self) -> bool {
        self.week().is_some()
    }

    /// Les horaires par jour, ou `None` tant que l'hôte n'en a saisi aucune plage.
    fn week(&self) -> Option<[Vec<hours::Range>; 7]> {
        let week = hours::parse_week(&self.weekly_hours);
        week.iter().any(|day| !day.is_empty()).then_some(week)
    }

    /// Rien de stocké que l'id : un emplacement que l'hôte a laissé (ou vidé).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.detail.is_blank()
            && self.warning.is_blank()
            && self.note.as_ref().is_none_or(I18nText::is_blank)
            && self.photo_refs().is_empty()
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

/// Un téléphone en E.164, une fois retirés les espaces, points, tirets et parenthèses que l'hôte
/// tape — la règle d'`emergency-contacts`, sans ses numéros courts (15, 112) : on ne réserve pas
/// une sortie en mer au 15.
fn phone_error(phone: &str) -> Option<I18nText> {
    let compact: String = phone
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '.' | '-' | '(' | ')'))
        .collect();
    check::phone(&compact)
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

/// A list of ids, as an array or as text: the multiple choice sends an array; older drafts kept
/// it as JSON in a string (`"[\"a\",\"b\"]"`), a bare value, or a comma list.
fn id_list<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(|id| id.trim().to_string()))
            .filter(|id| !id.is_empty())
            .collect(),
        Value::String(text) => serde_json::from_str::<Vec<String>>(&text).unwrap_or_else(|_| {
            text.split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect()
        }),
        _ => Vec::new(),
    })
}

/// [`id_list`], absent kept apart from empty: `None` is « never saved », `Some([])` « none chosen ».
fn optional_id_list<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    id_list(deserializer).map(Some)
}

/// A number from the form — `12`, `12.6` (rounded), `"12"` — or none: an empty or unreadable value
/// drops the setting rather than refusing the whole config.
fn optional_number<'de, D, N>(deserializer: D) -> std::result::Result<Option<N>, D::Error>
where
    D: Deserializer<'de>,
    N: TryFrom<u64>,
{
    let number = match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().replace(',', ".").parse::<f64>().ok(),
        _ => None,
    };
    Ok(number
        .filter(|n| n.is_finite() && *n >= 0.0)
        .and_then(|n| N::try_from(n.round() as u64).ok()))
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

fn deserialize_viator_min_rating<'de, D>(deserializer: D) -> std::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    number_or_text(deserializer, VIATOR_DEFAULT_MIN_RATING)
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
    fn viator_stays_off_on_a_config_that_predates_it() {
        let config: ModuleConfig = serde_json::from_value(json!({ "spots": [] })).unwrap();
        assert_eq!(config.viator(), ViatorConfig::default());
        assert_eq!(config.viator().min_rating, 4);
        let config: ModuleConfig = serde_json::from_value(json!({
            "viator_enabled": true, "viator_min_rating": "0"
        }))
        .unwrap();
        assert_eq!(
            config.viator(),
            ViatorConfig {
                enabled: true,
                min_rating: 0,
                max_price: None,
            }
        );
        assert_eq!(
            ViatorConfig {
                enabled: true,
                min_rating: 9,
                max_price: None,
            }
            .normalized_min_rating(),
            5
        );
    }

    #[test]
    fn tiqets_settings_written_outside_the_form_stay_in_range() {
        let config = TiqetsConfig {
            enabled: true,
            radius_km: 5000,
            min_rating: 9,
            max_price: None,
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
                min_rating: 4,
                max_price: None,
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
                min_rating: 3,
                max_price: None,
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

    /// Le jour de fermeture : les sept jetons du formulaire, et rien d'autre.
    #[test]
    fn a_closing_day_closes_only_that_day() {
        for (token, day) in WEEKDAYS {
            let row = SpotRow {
                closed_day: Some(token.to_string()),
                ..SpotRow::default()
            };
            assert_eq!(row.closed_weekday(), Some(day));
            assert!(row.closed_on(day));
            assert!(!row.closed_on(day.succ()));
        }
        // Les espaces et la casse du formulaire passent ; un jeton inventé ne ferme rien, plutôt
        // que de fermer un jour deviné.
        let tolerant = SpotRow {
            closed_day: Some(" MON ".to_string()),
            ..SpotRow::default()
        };
        assert_eq!(tolerant.closed_weekday(), Some(Weekday::Mon));
        for raw in ["", "lundi", "monday", "8", "mo"] {
            let row = SpotRow {
                closed_day: Some(raw.to_string()),
                ..SpotRow::default()
            };
            assert_eq!(row.closed_weekday(), None, "{raw}");
        }
        // Une adresse sans jour renseigné n'est jamais fermée, et le champ ne la rend pas remplie.
        let blank = SpotRow {
            closed_day: Some("sun".to_string()),
            ..SpotRow::default()
        };
        assert!(blank.is_blank());
        assert_eq!(SpotRow::default().closed_weekday(), None);
    }

    /// Les photos d'une adresse : l'ancien champ d'abord, sans doublon ni vide, cinq au plus.
    ///
    /// L'ancien `photo` est gardé en lecture parce que toute configuration écrite avant la galerie
    /// le porte. Le réunir ici est ce qui permet au rendu de n'avoir qu'une source — sinon chaque
    /// appelant devait se souvenir des deux, et l'un l'aurait oublié.
    #[test]
    fn a_gallery_joins_the_old_photo_with_the_new_ones() {
        // Une configuration d'avant la galerie.
        let legacy = SpotRow {
            photo: "portaki-file:a".into(),
            ..SpotRow::default()
        };
        assert_eq!(legacy.photo_refs(), ["portaki-file:a"]);
        assert_eq!(legacy.photo_ref(), Some("portaki-file:a"));

        // L'ancienne vient en tête, et le doublon que le formulaire renvoie ne compte qu'une fois.
        let both = SpotRow {
            photo: "portaki-file:a".into(),
            photos: vec![
                "portaki-file:a".into(),
                "  ".into(),
                "portaki-file:b".into(),
            ],
            ..SpotRow::default()
        };
        assert_eq!(both.photo_refs(), ["portaki-file:a", "portaki-file:b"]);

        // La couverture est la première, pas l'ancien champ par principe.
        let only_new = SpotRow {
            photos: vec!["portaki-file:b".into(), "portaki-file:c".into()],
            ..SpotRow::default()
        };
        assert_eq!(only_new.photo_ref(), Some("portaki-file:b"));

        // Jamais plus de cinq : au-delà on ferait défiler un album.
        let many = SpotRow {
            photos: (0..9).map(|n| format!("portaki-file:{n}")).collect(),
            ..SpotRow::default()
        };
        assert_eq!(many.photo_refs().len(), MAX_GALLERY);

        // Une adresse qui n'a que des photos n'est pas un emplacement vide.
        assert!(SpotRow::default().is_blank());
        assert!(!only_new.is_blank());
    }

    /// Les erreurs nomment leur champ ; une ligne vide n'en a pas.
    #[test]
    fn spot_problems_name_their_field() {
        let config: ModuleConfig = serde_json::from_value(serde_json::json!({
            "spots": [
                { "title": "" },
                { "title": "", "detail": "Super" },
                { "title": "Le Bacon", "url": "http://bacon.fr", "phone": "04 93 61 50 02" }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "spots.1.title",
                "spots.1.category",
                "spots.2.category",
                "spots.2.phone",
                "spots.2.url"
            ]
        );
    }

    fn message(config: &ModuleConfig, field: &str) -> Option<String> {
        config.error_of(field).map(|e| e.get("fr").to_string())
    }

    /// La catégorie est obligatoire, l'avertissement borné à 120, la saison lisible.
    #[test]
    fn spot_category_warning_and_season_are_checked() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "spots": [
                { "title": "Le Bacon" },
                { "title": "Plage", "category": "Plage", "warning": "x".repeat(121),
                  "season_from": "31/02", "season_to": "30/09" },
                { "title": "Plage", "category": "Plage", "warning": "Réservation indispensable",
                  "season_from": "01/06", "season_to": "30/09" }
            ]
        }))
        .unwrap();
        assert_eq!(
            message(&config, "spots.0.category").as_deref(),
            Some("Choisissez une catégorie.")
        );
        assert_eq!(
            message(&config, "spots.1.warning").as_deref(),
            Some("120 caractères au maximum.")
        );
        assert_eq!(
            message(&config, "spots.1.season_from").as_deref(),
            Some("Date au format 23/08.")
        );
        assert!(config
            .problems()
            .iter()
            .all(|(field, _)| !field.starts_with("spots.2")));
    }

    /// Les activités de l'hôte : bornes, téléphone, lien, et six au plus.
    #[test]
    fn host_activity_rows_are_checked() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "host_activities": [
                { "title": "Voilier", "provider": "p".repeat(61), "duration": "d".repeat(31),
                  "phone": "06 12 34 56 78", "url": "http://marc.fr" },
                { "title": "Kayak", "provider": "Marc", "duration": "3 h",
                  "phone": "+33 (6) 12 34 56 78", "url": "https://marc.fr" }
            ]
        }))
        .unwrap();
        assert_eq!(
            message(&config, "host_activities.0.provider").as_deref(),
            Some("60 caractères au maximum.")
        );
        assert_eq!(
            message(&config, "host_activities.0.duration").as_deref(),
            Some("30 caractères au maximum.")
        );
        assert_eq!(
            message(&config, "host_activities.0.phone").as_deref(),
            Some("Ce numéro n'est pas valide. Vérifiez l'indicatif.")
        );
        assert_eq!(
            message(&config, "host_activities.0.url").as_deref(),
            Some("L'adresse doit commencer par https://")
        );
        assert!(config
            .problems()
            .iter()
            .all(|(field, _)| !field.starts_with("host_activities.1")));

        let seven: Vec<Value> = (0..7)
            .map(|n| json!({ "title": format!("A{n}"), "provider": "Marc" }))
            .collect();
        let config: ModuleConfig =
            serde_json::from_value(json!({ "host_activities": seven })).unwrap();
        assert_eq!(
            message(&config, "host_activities").as_deref(),
            Some("6 activités au maximum.")
        );
    }

    /// L'emoji choisi, sinon celui de la catégorie, sans casse ; une catégorie libre n'en a pas.
    #[test]
    fn a_spot_emoji_falls_back_on_its_category() {
        let row = |emoji: Option<&str>, category: Option<&str>| SpotRow {
            emoji: emoji.map(String::from),
            category: category.map(String::from),
            ..SpotRow::default()
        };
        assert_eq!(
            row(Some("🦞"), Some("Restaurant")).emoji().as_deref(),
            Some("🦞")
        );
        assert_eq!(row(None, Some(" plage ")).emoji().as_deref(), Some("🏖️"));
        assert_eq!(row(None, Some("Bien-être")).emoji().as_deref(), Some("🧘"));
        assert_eq!(row(None, Some("Coup de cœur")).emoji(), None);
        assert_eq!(row(None, None).emoji(), None);
    }

    /// La saison : bornes incluses, passage du nouvel an, et toute l'année sans les deux bornes.
    #[test]
    fn a_season_hides_a_spot_outside_it() {
        let day = |d: u32, m: u32| NaiveDate::from_ymd_opt(2026, m, d).unwrap();
        let row = |from: &str, to: &str| SpotRow {
            title: I18nText::from("Plage"),
            season_from: Some(from.into()),
            season_to: Some(to.into()),
            ..SpotRow::default()
        };
        let summer = row("01/06", "30/09");
        assert!(summer.in_season(day(1, 6)));
        assert!(summer.in_season(day(30, 9)));
        assert!(!summer.in_season(day(1, 10)));
        assert!(!summer.in_season(day(31, 5)));
        let winter = row("01/11", "31/03");
        assert!(winter.in_season(day(1, 1)));
        assert!(winter.in_season(day(15, 11)));
        assert!(!winter.in_season(day(1, 7)));
        // Une borne seule, ou illisible : toute l'année.
        assert!(row("01/06", "").in_season(day(1, 1)));
        assert!(row("31/02", "30/09").in_season(day(1, 1)));
        assert!(SpotRow::default().in_season(day(1, 1)));

        let config = ModuleConfig {
            spots: vec![summer, row("01/01", "31/12")],
            ..ModuleConfig::default()
        };
        assert_eq!(config.guest_spots(Some(day(1, 1))).len(), 1);
        assert_eq!(config.guest_spots(Some(day(1, 7))).len(), 2);
        assert_eq!(config.guest_spots(None).len(), 2);
    }

    /// Avant la nouvelle carte, chaque fournisseur suit son ancien interrupteur ; enregistrée, elle
    /// gagne : un rayon, une note, un prix pour tous, et le catalogue éteint coupe tout.
    #[test]
    fn the_partner_settings_fall_back_on_the_old_switches() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "viator_enabled": true, "tiqets_radius_km": "40", "viator_min_rating": "3"
        }))
        .unwrap();
        assert!(config.partners_on());
        assert_eq!(config.providers(), ["viator"]);
        assert!(config.viator().enabled);
        assert!(!config.tiqets().enabled);
        assert_eq!(config.tiqets().radius_km, 40);
        assert_eq!(config.viator().min_rating, 3);

        let config: ModuleConfig = serde_json::from_value(json!({
            "viator_enabled": true, "tiqets_radius_km": "40",
            "partners_enabled": true, "providers": ["tiqets", "viator"],
            "partner_radius_km": "15", "partner_max_price": 80.4, "partner_min_rating": "4"
        }))
        .unwrap();
        assert_eq!(config.providers(), ["viator", "tiqets"]);
        assert_eq!(
            config.tiqets(),
            TiqetsConfig {
                enabled: true,
                radius_km: 15,
                min_rating: 4,
                max_price: Some(80)
            }
        );
        assert_eq!(config.viator().max_price, Some(80));
        assert_eq!(config.viator().min_rating, 4);

        let off: ModuleConfig = serde_json::from_value(json!({
            "partners_enabled": false, "providers": ["viator"], "activities_enabled": true,
            "activities": [{ "url": "https://gyg.me/aBcD12" }]
        }))
        .unwrap();
        assert!(!off.viator().enabled);
        assert!(!off.activities().enabled);

        // Les choix de l'hôte s'affichent sans l'ancienne recherche GetYourGuide.
        let picks: ModuleConfig = serde_json::from_value(json!({
            "partners_enabled": true, "providers": ["viator"],
            "activities": [{ "url": "https://www.tiqets.com/fr/musee-p1/" }]
        }))
        .unwrap();
        assert!(picks.activities().enabled);
        assert!(!picks.activities().search);
    }

    #[test]
    fn the_partner_settings_are_checked_only_when_the_catalogue_is_on() {
        let raw = json!({
            "partners_enabled": true, "providers": [], "partner_radius_km": 80,
            "partner_max_price": 5000,
            "activities": [{ "url": "https://example.com/x", "tip": "x".repeat(201) }]
        });
        let config: ModuleConfig = serde_json::from_value(raw.clone()).unwrap();
        for (field, expected) in [
            ("providers", "Choisissez au moins un fournisseur."),
            ("partner_radius_km", "Entre 1 et 50 km."),
            ("partner_max_price", "Entre 1 et 1 000 €."),
            (
                "activities.0.url",
                "Ce lien n’est pas une activité d’un fournisseur connecté.",
            ),
            ("activities.0.tip", "200 caractères au maximum."),
        ] {
            assert_eq!(
                message(&config, field).as_deref(),
                Some(expected),
                "{field}"
            );
        }
        let mut off = raw;
        off["partners_enabled"] = json!(false);
        let off: ModuleConfig = serde_json::from_value(off).unwrap();
        assert!(off.problems().is_empty());

        let six: Vec<Value> = (0..6)
            .map(|n| json!({ "url": format!("https://gyg.me/a{n}") }))
            .collect();
        let config: ModuleConfig =
            serde_json::from_value(json!({ "partners_enabled": true, "activities": six })).unwrap();
        assert_eq!(
            message(&config, "activities").as_deref(),
            Some("5 activités au maximum en avant.")
        );
    }

    /// Les horaires par jour : un jour sans plage est fermé ; vides, l'ancien jour de fermeture.
    #[test]
    fn weekly_hours_close_the_days_without_a_range() {
        let spot = SpotRow {
            weekly_hours: "mon=12:00-14:00,19:00-22:00;tue=12:00-14:00".into(),
            closed_day: Some("tue".into()),
            ..SpotRow::default()
        };
        assert!(spot.has_weekly_hours());
        assert!(!spot.closed_on(Weekday::Mon));
        assert!(!spot.closed_on(Weekday::Tue));
        assert!(spot.closed_on(Weekday::Wed));
        assert_eq!(
            spot.hours_on(Weekday::Mon).as_deref(),
            Some("12:00 – 14:00, 19:00 – 22:00")
        );
        assert_eq!(spot.hours_on(Weekday::Sun), None);
        let legacy = SpotRow {
            closed_day: Some("tue".into()),
            ..SpotRow::default()
        };
        assert!(!legacy.has_weekly_hours());
        assert!(legacy.closed_on(Weekday::Tue));
        assert!(!legacy.closed_on(Weekday::Wed));
    }

    /// Le conseil borné à 280 ; un avantage sans accord avertit, il n'est pas une erreur.
    #[test]
    fn the_tip_is_bounded_and_a_perk_asks_for_an_agreement() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "spots": [{ "title": "Le Bacon", "category": "Restaurant",
                        "note": "x".repeat(281), "perk": "10 %" }]
        }))
        .unwrap();
        assert_eq!(
            message(&config, "spots.0.note").as_deref(),
            Some("280 caractères au maximum.")
        );
        assert!(config.spots[0].perk_without_agreement());
        assert!(config.error_of("spots.0.perk").is_none());
        let agreed = SpotRow {
            perk_agreement: true,
            ..config.spots[0].clone()
        };
        assert!(!agreed.perk_without_agreement());
    }

    #[test]
    fn a_product_without_price_is_never_too_expensive() {
        assert!(within_price(Some(20.0), Some(20)));
        assert!(!within_price(Some(20.5), Some(20)));
        assert!(within_price(None, Some(20)));
        assert!(within_price(Some(5000.0), None));
    }
}
