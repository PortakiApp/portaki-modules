//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

/// Combien d'itinéraires la configuration accepte (spec Randonnées §2.1).
///
/// ponytail: au-delà de huit, la spec veut un filtre par niveau en tête de la carte du voyageur ;
/// la liste reste groupée par niveau en attendant.
pub const MAX_TRAILS: usize = 30;

/// Les bornes des mesures (§2.1).
pub const DURATION_MIN: (f64, f64) = (5.0, 1_440.0);
pub const DISTANCE_KM: (f64, f64) = (0.1, 100.0);
pub const ELEVATION_M: (f64, f64) = (0.0, 5_000.0);

/// Les trois niveaux, figés et traduits (§2.23). Pas de texte libre : « assez sportif » ne se
/// compare à rien, et un niveau sert à choisir entre deux randonnées.
///
/// La forme (▲, ▲▲, ▲▲▲) vit dans les traductions, avec le libellé : c'est ce qui rend la
/// difficulté lisible sans couleur. Et la difficulté n'est pas un statut — donc jamais de vert,
/// d'orange ni de rouge.
pub const LEVELS: &[&str] = &["easy", "moderate", "hard"];

/// Boucle, aller-retour ou aller simple (un retour à prévoir).
pub const SHAPES: &[&str] = &["loop", "round_trip", "one_way"];

/// Au-delà, le départ n'est plus « devant le logement » : la fiche propose un itinéraire.
pub const FAR_START_METRES: f64 = 1_000.0;

/// En dessous, le départ est le logement : pas de second repère sur le plan, et pas d'itinéraire.
pub const AT_PROPERTY_METRES: f64 = 120.0;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
// Pas d'`Eq` : un départ porte des coordonnées, et deux flottants ne se comparent pas par égalité
// totale.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModuleConfig {
    /// Les itinéraires. La porte de publication vit dans `publishReadiness`, qui sait compter les
    /// lignes complètes — ce qu'un `required` sur une liste ne fait pas.
    #[field(label = "config.trails")]
    pub trails: Vec<TrailRow>,
    /// Le lien « tous les sentiers de la commune », en bas de la liste.
    #[field(label = "host.commune.label")]
    pub commune_url: String,
}

impl ModuleConfig {
    /// Ce qui ne va pas, champ par champ (`trails.<i>.title`…) — sous le champ dans le
    /// formulaire, et dans `publishReadiness`. Le départ sans position avertit sans bloquer.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let text = crate::i18n::text;
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        if let Some(error) = check::https_url(self.commune_url.trim()) {
            problems.push(("commune_url".into(), error));
        }
        let filled = self.trails.iter().filter(|t| !t.is_blank()).count();
        if filled > MAX_TRAILS {
            problems.push(("trails".into(), text("host.trails.tooMany")));
        }
        let out_of = |value: Option<f64>, (min, max): (f64, f64)| {
            value.is_some_and(|v| !(min..=max).contains(&v))
        };
        for (index, trail) in self.trails.iter().enumerate() {
            if trail.is_blank() {
                continue;
            }
            let title = if trail.title.is_blank() {
                Some(text("host.trails.title.required"))
            } else {
                trail
                    .title
                    .by_language()
                    .find_map(|(_, title)| check::max_chars(title, 60))
            };
            for (key, error) in [
                ("title", title),
                (
                    "level",
                    trail
                        .level_key()
                        .is_none()
                        .then(|| text("host.trails.level.required")),
                ),
                (
                    "duration_min",
                    out_of(trail.duration_min, DURATION_MIN)
                        .then(|| text("host.trails.duration.range")),
                ),
                (
                    "distance_km",
                    out_of(trail.distance_km, DISTANCE_KM)
                        .then(|| text("host.trails.distance.range")),
                ),
                (
                    "elevation_m",
                    out_of(trail.elevation_m, ELEVATION_M)
                        .then(|| text("host.trails.elevation.range")),
                ),
                ("link_url", check::https_url(trail.link_url.trim())),
                (
                    "description",
                    trail
                        .description
                        .by_language()
                        .find_map(|(_, text)| check::max_chars(text, 1_200)),
                ),
            ] {
                if let Some(error) = error {
                    problems.push((format!("trails.{index}.{key}"), error));
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

    /// Les itinéraires que le voyageur voit, dans l'ordre du formulaire.
    pub fn parse_trails(&self) -> Vec<TrailRow> {
        self.trails
            .iter()
            .filter(|row| row.is_complete())
            .take(MAX_TRAILS)
            .cloned()
            .collect()
    }

    /// Les lignes commencées mais sans niveau ni titre : l'hôte croit les avoir écrites.
    pub fn trails_incomplete(&self) -> usize {
        self.trails
            .iter()
            .filter(|row| !row.is_blank() && !row.is_complete())
            .count()
    }

    /// Les itinéraires complets auxquels il manque une mesure — ils s'affichent, avec moins de
    /// tuiles.
    pub fn trails_missing_stats(&self) -> usize {
        self.parse_trails()
            .iter()
            .filter(|row| row.duration().is_none() || row.distance().is_none())
            .count()
    }

    /// Le lien de la commune, quand c'en est un.
    pub fn commune_link(&self) -> Option<&str> {
        let url = self.commune_url.trim();
        url.starts_with("https://").then_some(url)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_trails().is_empty()
    }
}

/// Un itinéraire.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TrailRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub title: I18nText,
    /// Une des valeurs de [`LEVELS`]. Rien d'autre ne s'affiche.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub level: String,
    /// Une des valeurs de [`SHAPES`] ; absente, la tuile « Type » disparaît.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub shape: String,
    /// Les trois mesures sont des flottants, y compris la durée et le dénivelé.
    ///
    /// Le `NumberInput` du formulaire hôte envoie un nombre, pas un entier : un hôte qui tape
    /// « 7,5 » en minutes, ou la plateforme qui renvoie `60.0` après un enregistrement, suffisait
    /// à faire refuser toute la configuration par serde — et le module rendait son état d'erreur
    /// à partir du premier enregistrement. Les accesseurs arrondissent à l'affichage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
    pub description: I18nText,
    /// Le départ, tel que le sélecteur l'a résolu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    /// La fiche de l'itinéraire chez un tiers (Visorando, IGN) — le bouton « Ouvrir la trace ».
    #[serde(skip_serializing_if = "String::is_empty")]
    pub link_url: String,
    /// La trace déposée par l'hôte : `portaki-file:<id>`, jamais les octets.
    ///
    /// C'est elle qui donne le tracé sur le plan, et c'est elle que le voyageur télécharge pour
    /// son application de randonnée — donc le fichier d'origine, pas une version recalculée.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub gpx_file: String,
    /// La photo du sentier, en référence `portaki-file:` — ce qu'on voit avant de partir.
    #[serde(default)]
    pub photo: String,
}

/// Une mesure utilisable : finie et strictement positive.
fn positive(value: Option<f64>) -> Option<f64> {
    value.filter(|measure| measure.is_finite() && *measure > 0.0)
}

impl TrailRow {
    /// La référence de la trace, si elle en est bien une — une URL externe n'en est pas une.
    pub fn gpx_ref(&self) -> Option<portaki_sdk::files::FileRef> {
        portaki_sdk::files::FileRef::parse(&self.gpx_file)
    }

    /// La photo déposée, en référence, ou `None` quand il n'y en a pas.
    pub fn photo_ref(&self) -> Option<&str> {
        let photo = self.photo.trim();
        (!photo.is_empty()).then_some(photo)
    }

    /// Rien que le formulaire montre : une ligne que l'hôte a laissée (ou vidée).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.description.is_blank()
            && self.level.trim().is_empty()
            && self.coordinates().is_none()
    }

    /// De quoi s'afficher : un titre et un niveau connu.
    ///
    /// Le niveau n'a pas de défaut, et c'est voulu : annoncer « Facile » un sentier que l'hôte n'a
    /// pas qualifié envoie un voyageur en tongs sur six cents mètres de dénivelé. Sans niveau, la
    /// ligne reste dans la configuration et ne s'affiche pas.
    pub fn is_complete(&self) -> bool {
        !self.title.is_blank() && self.level_key().is_some()
    }

    /// Le niveau, s'il est l'une des trois valeurs.
    pub fn level_key(&self) -> Option<&str> {
        let level = self.level.trim();
        LEVELS.iter().copied().find(|known| *known == level)
    }

    /// La forme, si elle est l'une de [`SHAPES`].
    ///
    /// `out_and_back` est ce qu'écrivait le pré-remplissage GPX avant 0.7.1 : c'est un aller-retour,
    /// et le lire comme tel garde la tuile « Type » des itinéraires déjà enregistrés.
    pub fn shape_key(&self) -> Option<&'static str> {
        let shape = match self.shape.trim() {
            "out_and_back" => "round_trip",
            other => other,
        };
        SHAPES.iter().copied().find(|known| *known == shape)
    }

    pub fn coordinates(&self) -> Option<(f64, f64)> {
        Some((self.lat?, self.lng?))
    }

    /// La durée de marche, en minutes entières. Une durée nulle ou négative n'est pas une mesure.
    pub fn duration(&self) -> Option<u32> {
        positive(self.duration_min).map(|minutes| minutes.round() as u32)
    }

    /// La distance, au dixième de kilomètre près à l'affichage.
    pub fn distance(&self) -> Option<f64> {
        positive(self.distance_km)
    }

    /// Le dénivelé positif, en mètres entiers. Zéro est une mesure — un sentier plat — donc il
    /// s'affiche ; un négatif n'en est pas une.
    pub fn elevation(&self) -> Option<u32> {
        let metres = self.elevation_m?;
        (metres.is_finite() && metres >= 0.0).then(|| metres.round() as u32)
    }

    /// L'identifiant de route de la fiche, stable par ligne.
    pub fn route_id(&self, index: usize) -> String {
        let id = self.id.trim();
        if id.is_empty() {
            format!("t{index}")
        } else {
            id.to_string()
        }
    }

    /// Le lien tiers, quand c'en est un.
    pub fn link(&self) -> Option<&str> {
        let url = self.link_url.trim();
        url.starts_with("https://").then_some(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parsed(value: serde_json::Value) -> ModuleConfig {
        serde_json::from_value(value).expect("config")
    }

    #[test]
    fn a_trail_without_a_level_does_not_reach_the_guest() {
        let config = parsed(json!({
            "trails": [
                { "title": "Phare de la Garoupe", "level": "easy" },
                { "title": "Baou de Saint-Jeannet" },
                { "title": "Vallée de la Brague", "level": "très dur" },
                {}
            ]
        }));
        let shown = config.parse_trails();
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].title.get("fr"), "Phare de la Garoupe");
        // Deux lignes commencées sans niveau connu, pas la quatrième qui est vide.
        assert_eq!(config.trails_incomplete(), 2);
    }

    #[test]
    fn only_https_links_are_offered() {
        let refused = parsed(json!({
            "trails": [{ "title": "A", "level": "hard", "link_url": "javascript:alert(1)" }],
            "commune_url": "www.antibesjuanlespins.com"
        }));
        assert_eq!(refused.trails[0].link(), None);
        assert_eq!(refused.commune_link(), None);

        let accepted = parsed(json!({
            "trails": [{ "title": "A", "level": "hard", "link_url": " https://visorando.com/x " }],
            "commune_url": "https://www.antibesjuanlespins.com"
        }));
        assert_eq!(accepted.trails[0].link(), Some("https://visorando.com/x"));
        assert!(accepted.commune_link().is_some());
    }

    #[test]
    fn the_form_sends_more_rows_than_the_bound_allows() {
        let rows: Vec<_> = (0..MAX_TRAILS + 3)
            .map(|i| json!({ "title": format!("Sentier {i}"), "level": "easy" }))
            .collect();
        let config = parsed(json!({ "trails": rows }));
        assert_eq!(config.parse_trails().len(), MAX_TRAILS);
    }

    #[test]
    fn a_route_id_falls_back_to_the_row_index() {
        let row = TrailRow::default();
        assert_eq!(row.route_id(4), "t4");
        let named = TrailRow {
            id: "garoupe".into(),
            ..TrailRow::default()
        };
        assert_eq!(named.route_id(4), "garoupe");
    }

    /// Le formulaire hôte envoie des nombres, pas des entiers : `60.0` doit se relire.
    ///
    /// Avec un `u32`, serde refusait la ligne — donc toute la configuration — et le module rendait
    /// son état d'erreur dès le premier enregistrement d'un hôte.
    #[test]
    fn a_measure_sent_as_a_float_still_reads() {
        let config = parsed(json!({
            "trails": [{ "title": "A", "level": "easy",
                         "duration_min": 60.0, "distance_km": 2.6, "elevation_m": 80.0 }]
        }));
        let row = &config.trails[0];
        assert_eq!(row.duration(), Some(60));
        assert_eq!(row.elevation(), Some(80));
        assert_eq!(row.distance(), Some(2.6));
        // Une durée saisie à la virgule s'arrondit à la minute.
        let rounded = parsed(json!({
            "trails": [{ "title": "A", "level": "easy", "duration_min": 7.5, "elevation_m": 12.4 }]
        }));
        assert_eq!(rounded.trails[0].duration(), Some(8));
        assert_eq!(rounded.trails[0].elevation(), Some(12));
    }

    /// Ce qui n'est pas une mesure ne s'affiche pas : zéro minute, une distance négative, un NaN.
    #[test]
    fn an_absurd_measure_is_not_a_measure() {
        let config = parsed(json!({
            "trails": [{ "title": "A", "level": "easy",
                         "duration_min": 0, "distance_km": -3, "elevation_m": -10 }]
        }));
        let row = &config.trails[0];
        assert_eq!(row.duration(), None);
        assert_eq!(row.distance(), None);
        assert_eq!(row.elevation(), None);
        // Un sentier plat garde son dénivelé : zéro mètre est une mesure.
        let flat = parsed(json!({
            "trails": [{ "title": "A", "level": "easy", "elevation_m": 0 }]
        }));
        assert_eq!(flat.trails[0].elevation(), Some(0));
    }

    #[test]
    fn a_shape_the_host_did_not_choose_has_no_tile() {
        let row = TrailRow {
            shape: "ligne droite".into(),
            ..TrailRow::default()
        };
        assert_eq!(row.shape_key(), None);
        let loop_row = TrailRow {
            shape: " loop ".into(),
            ..TrailRow::default()
        };
        assert_eq!(loop_row.shape_key(), Some("loop"));
    }

    /// Le pré-remplissage GPX écrivait `out_and_back` : ces itinéraires gardent leur tuile.
    #[test]
    fn a_stored_out_and_back_reads_as_a_round_trip() {
        let row = TrailRow {
            shape: "out_and_back".into(),
            ..TrailRow::default()
        };
        assert_eq!(row.shape_key(), Some("round_trip"));
    }

    /// Les bornes de la spec, sur la ligne et le champ qui les dépassent ; une ligne vide n'a rien.
    #[test]
    fn problems_name_their_field() {
        let config: ModuleConfig = serde_json::from_value(serde_json::json!({
            "commune_url": "visorando.com",
            "trails": [
                { "title": "" },
                { "title": "Garoupe", "level": "easy", "duration_min": 2, "distance_km": 150,
                  "elevation_m": 120, "link_url": "https://www.visorando.com/x" }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "commune_url",
                "trails.1.duration_min",
                "trails.1.distance_km"
            ]
        );
    }
}
