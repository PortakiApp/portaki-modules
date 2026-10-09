//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Combien d'organes la configuration accepte (spec Sécurité §2.2) : un logement a souvent deux
/// vannes d'eau ou un détecteur par chambre. Le nombre de lignes reste dynamique — le motif de
/// `rules` et `ical-sync` — plutôt que des emplacements figés laissés vides.
pub const MAX_SHUTOFFS: usize = 20;

/// Longueurs des textes (§2.1, §2.2).
const TITLE_MAX: usize = 60;
const LOCATION_MAX: usize = 120;
const TEXT_MAX: usize = 280;

/// Les six types d'organe, dans l'ordre du formulaire, et le glyphe de chacun.
///
/// La valeur de fil est stable et la traduction vit dans les bundles : renommer un libellé ne
/// doit pas perdre le type que l'hôte a choisi.
pub const KINDS: &[(&str, IconName)] = &[
    ("electricity", IconName::Zap),
    ("water", IconName::Droplet),
    ("gas", IconName::Flame),
    ("extinguisher", IconName::FireExtinguisher),
    ("smoke_detector", IconName::AlarmSmoke),
    ("other", IconName::InfoCircle),
];

/// Le type par défaut, quand l'hôte n'a rien choisi : « autre » plutôt qu'« électricité », parce
/// qu'un glyphe d'éclair sur une vanne d'eau se lit faux.
pub const KIND_OTHER: &str = "other";

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    /// Les organes de coupure. Pas de `required` : la porte de publication est dans
    /// `publishReadiness`, qui sait dire « au moins un organe complet » — ce qu'un champ
    /// obligatoire sur une liste ne dit pas (une ligne vide la remplit).
    #[field(label = "config.shutoffs")]
    pub shutoffs: Vec<ShutoffRow>,
    /// La consigne générale de l'hôte, affichée en bandeau d'information au-dessus des organes.
    #[field(label = "host.note.label")]
    pub general_note: I18nText,
}

impl ModuleConfig {
    /// Les organes que le voyageur voit : ceux dont le titre et l'emplacement sont écrits.
    ///
    /// Un organe sans emplacement est la seule chose que ce module existe pour dire — sous
    /// stress, « Vanne d'arrêt d'eau » sans l'endroit ne sert à rien. Il reste dans la
    /// configuration, il ne s'affiche pas.
    pub fn parse_shutoffs(&self) -> Vec<ShutoffRow> {
        self.shutoffs
            .iter()
            .filter(|row| row.is_complete())
            .take(MAX_SHUTOFFS)
            .cloned()
            .collect()
    }

    /// Les lignes commencées mais incomplètes : sans emplacement.
    pub fn shutoffs_incomplete(&self) -> usize {
        self.shutoffs
            .iter()
            .filter(|row| !row.is_blank() && !row.is_complete())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.parse_shutoffs().is_empty() && self.general_note.is_blank()
    }

    /// Ce qui ne va pas, champ par champ (`shutoffs.<i>.location`…) — sous le champ dans le
    /// formulaire, et dans `publishReadiness`. Une ligne vide n'a rien à dire, et un nom absent
    /// n'est pas une erreur : le type le donne (§4).
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        if let Some(error) = too_long(&self.general_note, TEXT_MAX) {
            problems.push(("general_note".into(), error));
        }
        let filled = self.shutoffs.iter().filter(|row| !row.is_blank()).count();
        if filled > MAX_SHUTOFFS {
            problems.push((
                "shutoffs".into(),
                crate::i18n::text("host.shutoffs.tooMany"),
            ));
        }
        for (index, row) in self.shutoffs.iter().enumerate() {
            if row.is_blank() {
                continue;
            }
            let location = if row.location.is_blank() {
                Some(crate::i18n::text("host.shutoffs.location.required"))
            } else {
                too_long(&row.location, LOCATION_MAX)
            };
            for (key, error) in [
                ("title", too_long(&row.title, TITLE_MAX)),
                ("location", location),
                ("instruction", too_long(&row.instruction, TEXT_MAX)),
            ] {
                if let Some(error) = error {
                    problems.push((format!("shutoffs.{index}.{key}"), error));
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
}

/// Un organe de coupure : où il est, et quoi faire une fois devant.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ShutoffRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Une des valeurs de [`KINDS`] ; ce qui n'en est pas une se lit « autre ».
    #[serde(skip_serializing_if = "String::is_empty")]
    pub kind: String,
    pub title: I18nText,
    pub location: I18nText,
    pub instruction: I18nText,
    /// La photo du robinet ou du disjoncteur, en référence `portaki-file:`.
    ///
    /// Une consigne écrite — « le robinet rouge, à droite du ballon » — demande de chercher ;
    /// une photo se reconnaît. En urgence, c'est la différence qui compte (§2.22).
    #[serde(default)]
    pub photo: String,
}

impl ShutoffRow {
    /// La photo déposée, en référence, ou `None` quand il n'y en a pas.
    pub fn photo_ref(&self) -> Option<&str> {
        let photo = self.photo.trim();
        (!photo.is_empty()).then_some(photo)
    }

    /// Rien que le formulaire montre : une ligne que l'hôte a laissée (ou vidée).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank() && self.location.is_blank() && self.instruction.is_blank()
    }

    /// De quoi s'afficher chez le voyageur : l'emplacement. Sans nom, le type le donne (§4).
    pub fn is_complete(&self) -> bool {
        !self.location.is_blank()
    }

    /// Le titre du bloc : celui de l'hôte, sinon le libellé du type (« Gaz »).
    pub fn title_for(&self, locale: &str) -> String {
        let title = self.title.get(locale).trim();
        if title.is_empty() {
            format!("i18n:host.kind.label.{}", self.kind_key())
        } else {
            title.to_string()
        }
    }

    /// Le type retenu, ramené à une valeur connue.
    pub fn kind_key(&self) -> &str {
        let kind = self.kind.trim();
        if KINDS.iter().any(|(wire, _)| *wire == kind) {
            kind
        } else {
            KIND_OTHER
        }
    }

    /// Le glyphe du type.
    pub fn icon(&self) -> IconName {
        let key = self.kind_key();
        KINDS
            .iter()
            .find(|(wire, _)| *wire == key)
            .map(|(_, icon)| *icon)
            .unwrap_or(IconName::InfoCircle)
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
    fn a_row_without_a_location_does_not_reach_the_guest() {
        let config = config(json!({
            "shutoffs": [
                { "kind": "water", "title": "Vanne d'eau", "location": "Trappe des WC" },
                { "kind": "gas", "title": "Robinet de gaz" },
                {}
            ]
        }));
        let shown = config.parse_shutoffs();
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].title.get("fr"), "Vanne d'eau");
        // La ligne du gaz est commencée, pas la troisième : une seule est à finir.
        assert_eq!(config.shutoffs_incomplete(), 1);
    }

    #[test]
    fn an_unknown_kind_reads_as_other() {
        let row = ShutoffRow {
            kind: "boiler".into(),
            ..ShutoffRow::default()
        };
        assert_eq!(row.kind_key(), "other");
        assert_eq!(row.icon(), IconName::InfoCircle);
        let gas = ShutoffRow {
            kind: " gas ".into(),
            ..ShutoffRow::default()
        };
        assert_eq!(gas.icon(), IconName::Flame);
    }

    #[test]
    fn the_form_sends_more_rows_than_the_bound_allows() {
        let rows: Vec<_> = (0..MAX_SHUTOFFS + 3)
            .map(|i| json!({ "title": format!("Organe {i}"), "location": "Ici" }))
            .collect();
        let config = config(json!({ "shutoffs": rows }));
        assert_eq!(config.parse_shutoffs().len(), MAX_SHUTOFFS);
    }

    /// Sans nom, le type le donne ; sans emplacement, la ligne attend, avec son erreur.
    #[test]
    fn a_row_needs_its_location_not_its_name() {
        let config = config(json!({ "shutoffs": [
            { "kind": "gas", "location": "Sous l'évier" },
            { "title": "Disjoncteur" },
            { "title": "", "location": "" }
        ] }));
        assert_eq!(config.parse_shutoffs().len(), 1);
        assert_eq!(
            config.parse_shutoffs()[0].title_for("fr"),
            "i18n:host.kind.label.gas"
        );
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(fields, ["shutoffs.1.location"]);
    }
}
