//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Mirrors design `editorPrearrival` / `prearrival-editor-v1`:
//! - when to show the guest form (`show_when`)
//! - which questions are enabled (`ask_*`)

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::time::PropertyTz;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// When the guest form becomes available.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowWhen {
    /// From booking confirmation (as soon as the stay exists).
    Confirm,
    /// 48 h before check-in.
    #[default]
    Before,
    /// On check-in day.
    Checkin,
}

impl ShowWhen {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "confirm" => Self::Confirm,
            "checkin" | "at_checkin" | "at-checkin" => Self::Checkin,
            _ => Self::Before,
        }
    }

    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Before => "before",
            Self::Checkin => "checkin",
        }
    }

    pub const CHOICE_LIST_WIRE_VALUES: &'static [&'static str] = &["confirm", "before", "checkin"];
}

/// La limite pour remplir le formulaire (spec Pré-arrivée §2.3), à l'heure du logement.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Deadline {
    /// Trois jours avant l'arrivée, jusqu'au soir : minuit, fin de J-3.
    #[serde(rename = "j-3")]
    J3,
    /// La veille de l'arrivée, à 18 h.
    #[default]
    #[serde(rename = "j-1-18h")]
    J1At18,
    /// Le jour de l'arrivée, à midi.
    #[serde(rename = "j0-12h")]
    J0At12,
}

impl Deadline {
    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::J3 => "j-3",
            Self::J1At18 => "j-1-18h",
            Self::J0At12 => "j0-12h",
        }
    }

    pub const CHOICE_LIST_WIRE_VALUES: &'static [&'static str] = &["j-3", "j-1-18h", "j0-12h"];

    /// L'instant de la limite pour une arrivée à `checkin`, dans le fuseau du logement (UTC
    /// quand le SDK ne connaît pas le fuseau, comme [`crate::slots::checkin_hour`]).
    pub fn at(&self, checkin: DateTime<Utc>, timezone: &str) -> DateTime<Utc> {
        let tz = PropertyTz::parse(timezone);
        let day = tz.map_or(checkin.date_naive(), |tz| tz.to_local(checkin).date_naive());
        let (days_before, hour) = match self {
            Self::J3 => (2, 0),
            Self::J1At18 => (1, 18),
            Self::J0At12 => (0, 12),
        };
        let local = (day - Duration::days(days_before))
            .and_hms_opt(hour, 0, 0)
            .unwrap_or_default();
        tz.map_or(local.and_utc(), |tz| tz.from_local(local))
    }
}

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
/// The questions sit flat, as the form sends them; the KV kept them under `questions`.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(
        kind = "select",
        options = ["confirm", "before", "checkin"],
        label = "host.when"
    )]
    pub show_when: ShowWhen,
    #[field(label = "host.question.arrival")]
    pub ask_arrival_time: bool,
    #[field(label = "host.question.occasion")]
    pub ask_occasion: bool,
    #[field(label = "host.question.allergies")]
    pub ask_allergies: bool,
    #[field(label = "host.question.guestCount")]
    pub ask_guest_count: bool,
    #[field(label = "host.question.specialNeeds")]
    pub ask_special_needs: bool,
    #[field(label = "host.question.idDocument")]
    pub ask_id_document: bool,
    /// « Vous arrivez en ? » — voiture, train, avion (§2.20).
    ///
    /// Elle décide ce que l'hôte prépare : une place de parking, un horaire de train, un
    /// transfert d'aéroport. Éteinte par défaut, comme toute question neuve : un hôte qui n'a
    /// pas demandé une question ne doit pas la voir apparaître dans son formulaire.
    #[field(label = "config.askTransport")]
    pub ask_transport: bool,
    /// Le pas des créneaux d'arrivée : `ranges` (les trois plages d'avant ce réglage), ou `15`,
    /// `30`, `60` minutes de l'heure d'entrée à [`Self::slots_until`] (spec Pré-arrivée §2.1).
    #[field(
        kind = "select",
        options = ["ranges", "15", "30", "60"],
        label = "host.slots.step"
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub slot_step: String,
    /// Le dernier créneau, `HH:MM` ; vide : 23:00.
    #[field(label = "host.slots.until")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub slots_until: String,
    /// À remplir avant (§2.3).
    #[field(
        kind = "select",
        options = ["j-3", "j-1-18h", "j0-12h"],
        label = "host.deadline"
    )]
    pub deadline: Deadline,
    /// Un e-mail au voyageur qui n'a pas répondu, la veille de la limite (§2.3).
    #[field(label = "host.reminder")]
    pub reminder: bool,
    /// Les questions de l'hôte, posées après les questions standard (§2.2), 5 au plus.
    #[field(structured, label = "host.custom.title")]
    pub custom_questions: Vec<CustomQuestion>,
}

/// Combien de questions l'hôte peut ajouter (§2.2).
pub const MAX_CUSTOM_QUESTIONS: usize = 5;
/// Les bornes des options d'une question à choix (§2.2).
pub const MIN_OPTIONS: usize = 2;
pub const MAX_OPTIONS: usize = 6;

/// Ce qu'une question de l'hôte attend du voyageur.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    Text,
    Choice,
    /// Une valeur inconnue se lit « oui / non » : elle ne doit pas rendre la config illisible.
    #[default]
    #[serde(other)]
    YesNo,
}

impl QuestionKind {
    pub const WIRE_VALUES: [&'static str; 3] = ["yes_no", "text", "choice"];

    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::YesNo => "yes_no",
            Self::Choice => "choice",
        }
    }
}

/// Une question de l'hôte, telle que le formulaire l'envoie (`custom_questions.N.*`).
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CustomQuestion {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub label: I18nText,
    #[serde(rename = "type")]
    pub kind: QuestionKind,
    /// Les pastilles d'une question à choix ; une sous-liste (`custom_questions.N.options.M`).
    pub options: Vec<QuestionOption>,
    #[serde(deserialize_with = "lenient_bool")]
    pub required: bool,
}

/// Une option d'une question à choix.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct QuestionOption {
    pub label: I18nText,
}

impl CustomQuestion {
    /// Les options saisies, vides retirées, 6 au plus.
    pub fn options(&self) -> Vec<&I18nText> {
        self.options
            .iter()
            .map(|option| &option.label)
            .filter(|label| !label.is_blank())
            .take(MAX_OPTIONS)
            .collect()
    }
}

/// Un interrupteur dans une ligne arrive en booléen ou en texte (`"true"`).
fn lenient_bool<'de, D: serde::Deserializer<'de>>(de: D) -> Result<bool, D::Error> {
    Ok(match Value::deserialize(de)? {
        Value::Bool(value) => value,
        Value::String(raw) => matches!(raw.trim(), "true" | "on" | "1"),
        _ => false,
    })
}

/// Le dernier créneau proposé quand l'hôte n'en donne pas.
pub const DEFAULT_SLOTS_UNTIL: &str = "23:00";

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            show_when: ShowWhen::Before,
            ask_arrival_time: true,
            ask_occasion: true,
            ask_allergies: true,
            ask_guest_count: true,
            ask_special_needs: false,
            ask_id_document: false,
            ask_transport: false,
            slot_step: String::new(),
            slots_until: String::new(),
            deadline: Deadline::J1At18,
            reminder: true,
            custom_questions: Vec::new(),
        }
    }
}

/// The old KV blob nested the questions under `questions`: lift them to the declared keys
/// (a flat key, already there, wins).
fn legacy(mut old: Value) -> Value {
    if let Some(blob) = old.as_object_mut() {
        if let Some(Value::Object(questions)) = blob.remove("questions") {
            for (key, value) in questions {
                blob.entry(key).or_insert(value);
            }
        }
    }
    old
}

impl ModuleConfig {
    /// Les créneaux d'arrivée d'un logement qui ouvre à `hour` : les trois plages, ou des
    /// créneaux réguliers jusqu'à l'heure de fin.
    pub fn slots(&self, hour: u32) -> Vec<crate::slots::Slot> {
        let step = self.slot_step.trim().parse::<u32>().ok();
        let until = crate::slots::minutes_of(self.slots_until.trim())
            .or_else(|| crate::slots::minutes_of(DEFAULT_SLOTS_UNTIL))
            .unwrap_or(23 * 60);
        match step {
            Some(step) => crate::slots::stepped(hour, until, step),
            None => crate::slots::slots(hour),
        }
    }

    /// Ce qui ne va pas, champ par champ (`custom_questions.0.label`…) — sous le champ, et dans
    /// `publishReadiness`.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        let mut problems: Vec<(String, I18nText)> = check::time(self.slots_until.trim())
            .map(|error| ("slots_until".to_string(), error))
            .into_iter()
            .collect();
        if self.custom_questions.len() > MAX_CUSTOM_QUESTIONS {
            problems.push((
                "custom_questions".into(),
                crate::i18n::text("host.custom.tooMany"),
            ));
        }
        for (index, question) in self.custom_questions.iter().enumerate() {
            let label = if question.label.is_blank() {
                Some(crate::i18n::text("host.custom.label.required"))
            } else {
                too_long(&question.label, 120)
            };
            if let Some(error) = label {
                problems.push((format!("custom_questions.{index}.label"), error));
            }
            if question.kind == QuestionKind::Choice {
                let options = question.options();
                let error = if options.len() < MIN_OPTIONS {
                    Some(crate::i18n::text("host.custom.options.min"))
                } else {
                    options.iter().find_map(|option| too_long(option, 40))
                };
                if let Some(error) = error {
                    problems.push((format!("custom_questions.{index}.options"), error));
                }
            }
        }
        problems
    }

    /// L'erreur d'un champ du formulaire, s'il en a une.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }

    /// Les questions que le voyageur voit : écrites, 5 au plus, et à choix seulement avec deux
    /// options.
    pub fn guest_questions(&self) -> impl Iterator<Item = &CustomQuestion> {
        self.custom_questions
            .iter()
            .filter(|question| !question.label.is_blank())
            .filter(|question| {
                question.kind != QuestionKind::Choice || question.options().len() >= MIN_OPTIONS
            })
            .take(MAX_CUSTOM_QUESTIONS)
    }

    /// At least one question is asked.
    pub fn asks_anything(&self) -> bool {
        self.ask_arrival_time
            || self.ask_occasion
            || self.ask_allergies
            || self.ask_guest_count
            || self.ask_special_needs
            || self.ask_id_document
            || self.ask_transport
            || self.guest_questions().next().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn legacy_nested_questions_are_lifted() {
        let mapped = legacy(json!({
            "show_when": "confirm",
            "questions": { "ask_occasion": false, "ask_id_document": true }
        }));
        assert_eq!(
            mapped,
            json!({ "show_when": "confirm", "ask_occasion": false, "ask_id_document": true })
        );
        // A flat key already there wins; a non-object `questions` brings nothing.
        assert_eq!(
            legacy(json!({ "ask_occasion": true, "questions": { "ask_occasion": false } })),
            json!({ "ask_occasion": true })
        );
        assert_eq!(legacy(json!({ "questions": "oops" })), json!({}));
    }

    /// The KV is read (through `legacy`) only while the platform sends no config; `{}` is a
    /// real, empty config.
    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "show_when": "confirm",
            "questions": { "ask_occasion": false, "ask_id_document": true }
        });
        let kv = serde_json::to_vec(&old).unwrap();
        let before_import = MockContext::guest()
            .with_kv("config", kv.clone())
            .run(|ctx| ModuleConfig::load(&ctx).unwrap());
        assert_eq!(before_import.show_when, ShowWhen::Confirm);
        assert!(!before_import.ask_occasion);
        assert!(before_import.ask_id_document);
        assert!(before_import.ask_allergies);

        let held = MockContext::guest()
            .with_kv("config", kv)
            .with_config(&json!({}))
            .run(|ctx| ModuleConfig::load(&ctx).unwrap());
        assert_eq!(held, ModuleConfig::default());
    }

    #[test]
    fn default_matches_design_sample() {
        let cfg = ModuleConfig::default();
        assert_eq!(cfg.show_when, ShowWhen::Before);
        assert!(cfg.ask_arrival_time);
        assert!(cfg.ask_occasion);
        assert!(cfg.ask_allergies);
        assert!(cfg.ask_guest_count);
        assert!(!cfg.ask_special_needs);
        assert!(!cfg.ask_id_document);
        assert_eq!(cfg.deadline, Deadline::J1At18);
        assert!(cfg.reminder);
        // Un réglage d'avant la limite garde la relance, comme le dit la spec.
        let old: ModuleConfig = serde_json::from_value(json!({ "show_when": "before" })).unwrap();
        assert_eq!((old.deadline, old.reminder), (Deadline::J1At18, true));
    }

    #[test]
    fn deadline_is_read_in_the_property_time() {
        let utc = |raw: &str| {
            DateTime::parse_from_rfc3339(raw)
                .unwrap()
                .with_timezone(&Utc)
        };
        // Arrivée le 20 juillet à 16 h, Paris (UTC+2).
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = |d: Deadline| d.at(checkin, "Europe/Paris");
        assert_eq!(at(Deadline::J3), utc("2026-07-17T22:00:00Z"));
        assert_eq!(at(Deadline::J1At18), utc("2026-07-19T16:00:00Z"));
        assert_eq!(at(Deadline::J0At12), utc("2026-07-20T10:00:00Z"));
        // Fuseau inconnu : l'heure UTC.
        assert_eq!(
            Deadline::J1At18.at(checkin, "Mars/Olympus"),
            utc("2026-07-19T18:00:00Z")
        );
        for wire in Deadline::CHOICE_LIST_WIRE_VALUES {
            let parsed: Deadline = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(parsed.as_wire(), *wire);
        }
        // Une limite hors liste est refusée, pas lue comme une autre.
        assert!(serde_json::from_value::<Deadline>(json!("j-5")).is_err());
    }

    #[test]
    fn show_when_choice_list_values_deserialize() {
        for wire in ShowWhen::CHOICE_LIST_WIRE_VALUES {
            let parsed = ShowWhen::parse(wire);
            assert_eq!(parsed.as_wire(), *wire);
        }
        // Une limite hors liste est refusée, pas lue comme une autre.
        assert!(serde_json::from_value::<Deadline>(json!("j-5")).is_err());
    }
}
