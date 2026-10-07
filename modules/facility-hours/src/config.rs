//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use std::collections::BTreeSet;

use crate::schedule::{parse_hm, DayHours, Schedule};
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(required, label = "config.facilities")]
    pub facilities: Vec<FacilityRow>,
    #[field(label = "host.note.label")]
    pub general_note: I18nText,
}

/// The old KV blob: the facilities as a JSON string, `facilities_json`, before the form slots;
/// each row's `lines` as a list of per-language texts; a row named `name` by an early form.
fn legacy(mut old: Value) -> Value {
    if let Some(object) = old.as_object_mut() {
        map_legacy(object);
    }
    old
}

fn map_legacy(old: &mut Map<String, Value>) {
    let listed = old
        .remove("facilities_json")
        .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
    let held = old
        .get("facilities")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty());
    if let (Some(facilities), false) = (listed, held) {
        old.insert("facilities".into(), facilities);
    }
    for row in old
        .get_mut("facilities")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object_mut)
    {
        if let Some(name) = row.remove("name") {
            row.entry("title").or_insert(name);
        }
        if let Some(Value::Array(lines)) = row.get("lines") {
            let lines = joined_lines(lines);
            row.insert("lines".into(), lines);
        }
    }
}

/// A list of per-language lines as one text per language, one line each — a line missing in a
/// language falls back as it did when shown alone.
fn joined_lines(lines: &[Value]) -> Value {
    let lines: Vec<I18nText> = lines
        .iter()
        .filter_map(|line| serde_json::from_value(line.clone()).ok())
        .collect();
    let languages: BTreeSet<&str> = lines
        .iter()
        .flat_map(|line| {
            ["fr", "en"]
                .into_iter()
                .chain(line.others.keys().map(String::as_str))
        })
        .collect();
    let by_language: Map<String, Value> = languages
        .into_iter()
        .map(|language| {
            let text = lines
                .iter()
                .map(|line| line.get(language).trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            (language.to_string(), Value::String(text))
        })
        .collect();
    Value::Object(by_language)
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_facilities().is_empty() && self.general_note.is_blank()
    }

    /// The named rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_facilities(&self) -> Vec<FacilityRow> {
        self.facilities
            .iter()
            .filter(|f| !f.title.is_blank())
            .cloned()
            .collect()
    }
}

/// A facility, as the form sends it (and its `id`); the platform keeps the other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FacilityRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub title: I18nText,
    /// One line of text per line.
    pub lines: I18nText,
    /// La phrase que l'hôte a écrite. Conservée : elle s'affiche telle quelle tant qu'aucune heure
    /// structurée n'est saisie, et rien ici ne la réécrit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<String>,
    /// Ouverture et fermeture en `HH:MM`, quand l'hôte les a données (§2.6). Facultatives : sans
    /// elles, la ligne garde sa phrase et n'affiche pas d'état — ce qui est honnête plutôt que
    /// deviné.
    /// Vides quand l'hôte n'a rien mis : le formulaire renvoie `""` et non l'absence, et une config
    /// pleine de chaînes vides serait un changement enregistré chez chaque hôte qui ouvre l'écran.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub opens_at: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub closes_at: String,
    /// La coupure du midi, quand l'équipement ferme entre deux services.
    ///
    /// Les deux bornes ou aucune : une seule ne dit pas quand la porte rouvre.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub break_from: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub break_to: String,
    /// Ouvert en continu : il n'y a alors pas d'heures à comparer.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub all_day: bool,
    /// Les jours qui dérogent aux horaires habituels, ou qui sont fermés.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<DayHours>,
    pub note: I18nText,
    /// Le groupe sous lequel la ligne se rangera dans la sous-page — « Séjour », « Équipements »,
    /// « Services » (§2.6). Texte libre : l'hôte sait comment son logement se découpe mieux
    /// qu'une liste figée. Vide quand il n'a rien mis, et la ligne se range alors en dernier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Le début de la saison, en `MM-JJ` — « 04-01 » pour le 1er avril (§2.6).
    ///
    /// Sans année : une piscine ouvre « d'avril à octobre » chaque année. Vides, la ligne est de
    /// toute saison, et rien ne change pour un hôte qui n'y touche pas.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub season_from: String,
    /// La fin de la saison, en `MM-JJ`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub season_to: String,
    /// L'icône de la ligne, prise dans le vocabulaire du livret ([`ICONS`]). Vide quand l'hôte
    /// n'en a pas choisi — la ligne sort alors sans pictogramme plutôt qu'avec un deviné.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Les icônes proposées à l'hôte, celles du dessin (§2.6) plus les lieux qui reviennent.
///
/// Une liste figée, pas du texte libre : un nom hors du vocabulaire du livret ne dessine rien.
pub const ICONS: [&str; 12] = [
    "key", "logout", "sun", "zap", "sparkles", "building", "package", "recycle", "clock", "wifi",
    "car", "droplet",
];

impl FacilityRow {
    /// Le groupe, débarrassé des espaces, ou `None` quand l'hôte n'en a pas donné.
    pub fn group_label(&self) -> Option<&str> {
        self.group
            .as_deref()
            .map(str::trim)
            .filter(|group| !group.is_empty())
    }

    /// L'icône, et seulement si elle est du vocabulaire : le reste ne dessinerait rien.
    pub fn icon_name(&self) -> Option<&str> {
        self.icon
            .as_deref()
            .map(str::trim)
            .filter(|icon| ICONS.contains(icon))
    }
}

impl FacilityRow {
    /// Les horaires de cette ligne, tels que le calcul les lit.
    pub fn schedule(&self) -> Schedule {
        Schedule {
            all_day: self.all_day,
            opens_at: parse_hm(&self.opens_at),
            closes_at: parse_hm(&self.closes_at),
            exceptions: self.exceptions.clone(),
            // Les deux bornes, ou aucune : une coupure sans fin laisserait la porte close.
            break_at: parse_hm(&self.break_from).zip(parse_hm(&self.break_to)),
            // Les deux bornes, ou aucune : une saison à une seule date ne dit pas quand elle
            // s'arrête, et la deviner fermerait la ligne à une date inventée.
            season: crate::schedule::parse_month_day(&self.season_from)
                .zip(crate::schedule::parse_month_day(&self.season_to)),
        }
    }
}

impl FacilityRow {
    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.hours.as_deref().is_none_or(|h| h.trim().is_empty())
            && self.lines.is_blank()
            && self.note.is_blank()
    }

    /// The non-blank lines in `locale`.
    pub fn lines(&self, locale: &str) -> Vec<String> {
        self.lines
            .get(locale)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_blank_slots_left_out() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "facilities": [{ "title": "Piscine", "hours": "9 h – 20 h" }, { "title": "", "hours": "" }],
            "general_note": "Horaires indicatifs"
        }))
        .unwrap();
        let facilities = config.parse_facilities();
        assert_eq!(facilities.len(), 1);
        assert_eq!(facilities[0].title.get("en"), "Piscine");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    fn legacy_facilities_json_becomes_facilities() {
        let mapped = legacy(json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00","note":{"fr":"Bonnet"}}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "id": "pool", "title": { "fr": "Piscine", "en": "Pool" },
                                 "hours": "08:00 – 20:00", "note": { "fr": "Bonnet" } }],
                "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("en"), "Pool");
        assert_eq!(config.facilities[0].note.get("en"), "Bonnet");
        assert_eq!(config.general_note.get("en"), "Indicative hours");
        // A list already there wins over the old string; an unreadable string imports nothing.
        let kept = legacy(json!({ "facilities": [{ "title": "Spa" }], "facilities_json": "[]" }));
        assert_eq!(kept, json!({ "facilities": [{ "title": "Spa" }] }));
        assert_eq!(legacy(json!({ "facilities_json": "[oops" })), json!({}));
        // An empty list (the old reader's « nothing saved yet ») still takes the string.
        let empty = legacy(json!({ "facilities": [], "facilities_json": r#"[{"title":"Spa"}]"# }));
        assert_eq!(empty, json!({ "facilities": [{ "title": "Spa" }] }));
    }

    #[test]
    fn legacy_lines_become_one_text_per_language() {
        let mapped = legacy(json!({ "facilities": [{
            "id": "pool",
            "title": { "fr": "Piscine", "en": "Pool", "de": "Schwimmbad" },
            "lines": [
                { "fr": "Tous les jours", "en": "Every day" },
                { "fr": "  " , "en": "" },
                { "fr": "Enfants accompagnés", "de": "Kinder begleitet" }
            ]
        }] }));
        assert_eq!(
            mapped["facilities"][0]["lines"],
            json!({
                "fr": "Tous les jours\nEnfants accompagnés",
                "en": "Every day\nEnfants accompagnés",
                "de": "Tous les jours\nKinder begleitet"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(
            config.facilities[0].lines("en-US"),
            ["Every day", "Enfants accompagnés"]
        );
        assert_eq!(config.facilities[0].title.get("de"), "Schwimmbad");
    }

    #[test]
    fn legacy_name_becomes_title_and_a_plain_note_stays() {
        let mapped = legacy(json!({
            "facilities": [{ "name": "Accueil", "hours": "16:00" }, { "name": "x", "title": { "fr": "Spa" } }],
            "general_note": "Horaires indicatifs"
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "title": "Accueil", "hours": "16:00" }, { "title": { "fr": "Spa" } }],
                "general_note": "Horaires indicatifs"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("fr"), "Accueil");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"lines":[{"fr":"Tous les jours","en":"Every day"}],"hours":"08:00 – 20:00"}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                let facilities = config.parse_facilities();
                assert_eq!(facilities[0].title.get("en"), "Pool");
                assert_eq!(facilities[0].lines("en"), ["Every day"]);
                assert_eq!(config.general_note.get("en"), "Indicative hours");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }
}
