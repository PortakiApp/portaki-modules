//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

/// Combien d'itinéraires le formulaire accepte.
///
/// Douze : un hôte de montagne en a plus que six, et la liste est déjà groupée par niveau, donc
/// elle se lit à douze. Au-delà de huit, le §2.23 veut un filtre par niveau en tête — il n'est pas
/// dans cette v1, et la borne reste basse pour que son absence ne se voie pas.
pub const MAX_TRAILS: usize = 12;

/// Les trois niveaux, figés et traduits (§2.23). Pas de texte libre : « assez sportif » ne se
/// compare à rien, et un niveau sert à choisir entre deux randonnées.
///
/// La forme (▲, ▲▲, ▲▲▲) vit dans les traductions, avec le libellé : c'est ce qui rend la
/// difficulté lisible sans couleur. Et la difficulté n'est pas un statut — donc jamais de vert,
/// d'orange ni de rouge.
pub const LEVELS: &[&str] = &["easy", "moderate", "hard"];

/// Boucle ou aller-retour. Deux valeurs, parce qu'un sentier est l'un ou l'autre.
pub const SHAPES: &[&str] = &["loop", "round_trip"];

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
            .filter(|row| row.duration_min.is_none() || row.distance_km.is_none())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<u32>,
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
}

impl TrailRow {
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

    /// La forme, si elle est l'une des deux.
    pub fn shape_key(&self) -> Option<&str> {
        let shape = self.shape.trim();
        SHAPES.iter().copied().find(|known| *known == shape)
    }

    pub fn coordinates(&self) -> Option<(f64, f64)> {
        Some((self.lat?, self.lng?))
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

    fn config(value: serde_json::Value) -> ModuleConfig {
        serde_json::from_value(value).expect("config")
    }

    #[test]
    fn a_trail_without_a_level_does_not_reach_the_guest() {
        let config = config(json!({
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
        let refused = config(json!({
            "trails": [{ "title": "A", "level": "hard", "link_url": "javascript:alert(1)" }],
            "commune_url": "www.antibesjuanlespins.com"
        }));
        assert_eq!(refused.trails[0].link(), None);
        assert_eq!(refused.commune_link(), None);

        let accepted = config(json!({
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
        let config = config(json!({ "trails": rows }));
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
}
