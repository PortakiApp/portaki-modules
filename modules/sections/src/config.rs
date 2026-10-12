//! Host configuration, held by the platform (`#[portaki_sdk::config]`) — spec Bienvenue §2.
//!
//! The host writes in the listing's « Fiche & coordonnées » card; the draft publishes with the
//! listing's own « Publier ». Before this, sections lived in the module's own tables
//! (`section_item` / `section_item_locale`): they stay readable until the first save writes
//! `sections` (spec §8).
//!
//! Copie depuis un autre logement (§6) : titres, textes et `show_from` sont copiés ; les photos ne
//! le sont jamais. Rien ne le marque ici : `#[field(local)]` vaut pour un champ entier, et la photo
//! est une sous-clé des lignes de `sections` — c'est la copie de la plateforme qui l'écarte.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};

use crate::content::body_plain_text;

/// « Afficher à partir de » (§2.1) : quand la ligne « Le mot de {hôte} » apparaît.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShowFrom {
    /// Dès la réservation.
    Booking,
    /// La veille de l'arrivée, dès minuit.
    #[default]
    DayBefore,
    /// Le jour de l'arrivée, dès minuit.
    ArrivalDay,
}

impl ShowFrom {
    pub const ALL: [ShowFrom; 3] = [Self::Booking, Self::DayBefore, Self::ArrivalDay];

    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Booking => "booking",
            Self::DayBefore => "day_before",
            Self::ArrivalDay => "arrival_day",
        }
    }

    /// The i18n key suffix of the option (`host.showFrom.{key}`).
    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Booking => "booking",
            Self::DayBefore => "dayBefore",
            Self::ArrivalDay => "arrivalDay",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_wire() == raw.trim())
    }
}

/// One section (§2.2). The first is the host's word of welcome.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SectionRow {
    /// Keeps a row through reorders and saves: the platform matches rows by id.
    pub id: String,
    pub title: I18nText,
    /// TipTap JSON (what `RichTextEditor` sends) or plain text / Markdown from before.
    pub body: I18nText,
    /// Photo d'ouverture du panneau : une URL `https://` ou une référence `portaki-file:`.
    pub photo: String,
}

impl SectionRow {
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self
                .body
                .by_language()
                .all(|(_, body)| body_plain_text(body).trim().is_empty())
    }
}

pub const MAX_SECTIONS: usize = 10;
pub const TITLE_MAX: usize = 60;
pub const BODY_MAX: usize = 3_000;
/// The first sentence is the booklet's teaser: past this, it is cut at a word (§3).
pub const TEASER_MAX: usize = 160;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(
        required,
        kind = "select",
        options = ["booking", "day_before", "arrival_day"],
        label = "host.showFrom.label"
    )]
    pub show_from: ShowFrom,
    /// `None` until the host saves once: the sections of the module's tables are read instead.
    #[field(structured, label = "host.sections.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<SectionRow>>,
}

impl ModuleConfig {
    /// The sections to show and edit: the saved list, or the module's tables from before it.
    pub fn sections(&self) -> Vec<SectionRow> {
        match &self.sections {
            Some(rows) => rows.clone(),
            None => crate::store::legacy_rows().unwrap_or_default(),
        }
    }
}

/// What is wrong with one field, for the card (`Field::error`) and `publishReadiness`.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// `config.<key>` — the field the dashboard scrolls to.
    pub id: String,
    /// The i18n key of the field's label.
    pub label: &'static str,
    pub message: I18nText,
    /// Blocks « Publier » ; otherwise a warning.
    pub blocking: bool,
}

impl ModuleConfig {
    /// Every rule of the spec, in the card's order. A legacy section is checked like any other:
    /// it warns under its field, and only blocks once the host saves the list.
    pub fn problems(&self) -> Vec<Problem> {
        let sections = self.sections();
        let mut problems = Vec::new();
        let saved = self.sections.is_some();
        if sections.len() > MAX_SECTIONS {
            problems.push(problem(
                "config.sections",
                "host.sections.label",
                "config.sections.max",
                saved,
            ));
        }
        for (index, row) in sections.iter().enumerate() {
            let at = |key: &str| format!("config.sections[{index}].{key}");
            if row.title.is_blank() {
                problems.push(problem(
                    &at("title"),
                    "host.title.label",
                    "config.title.required",
                    saved,
                ));
            }
            if let Some(message) = row
                .title
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, TITLE_MAX))
            {
                problems.push(Problem {
                    id: at("title"),
                    label: "host.title.label",
                    message,
                    blocking: saved,
                });
            }
            let bodies: Vec<String> = row
                .body
                .by_language()
                .map(|(_, body)| body_plain_text(body))
                .collect();
            if bodies.iter().all(|body| body.trim().is_empty()) {
                problems.push(problem(
                    &at("body"),
                    "host.body.label",
                    "config.body.required",
                    saved,
                ));
            }
            if let Some(message) = bodies
                .iter()
                .find_map(|body| check::max_chars(body, BODY_MAX))
            {
                problems.push(Problem {
                    id: at("body"),
                    label: "host.body.label",
                    message,
                    blocking: saved,
                });
            }
            if index == 0
                && bodies
                    .iter()
                    .any(|body| first_sentence(body).chars().count() > TEASER_MAX)
            {
                problems.push(problem(
                    &at("body"),
                    "host.body.label",
                    "config.body.teaser",
                    false,
                ));
            }
        }
        problems
    }
}

/// The blocking problem of one field, for its `Field::error`. Takes the problems computed once:
/// before the first save each computation reads the legacy tables.
pub fn error_of(problems: &[Problem], id: &str, locale: &str) -> Option<String> {
    problems
        .iter()
        .find(|p| p.id == id && p.blocking)
        .map(|p| p.message.get(locale).to_string())
}

/// Whether the first section's first sentence is past [`TEASER_MAX`].
pub fn teaser_too_long(problems: &[Problem]) -> bool {
    problems
        .iter()
        .any(|p| p.id == "config.sections[0].body" && !p.blocking)
}

/// Up to the first `.`, `!`, `?` or `…` followed by a space or the end — the booklet's teaser.
pub fn first_sentence(text: &str) -> &str {
    let text = text.trim();
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if matches!(c, '.' | '!' | '?' | '…')
            && chars.peek().is_none_or(|(_, next)| next.is_whitespace())
        {
            return &text[..at + c.len_utf8()];
        }
    }
    text
}

fn problem(id: &str, label: &'static str, message: &str, blocking: bool) -> Problem {
    Problem {
        id: id.into(),
        label,
        message: crate::i18n::text(message),
        blocking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(title: &str, body: &str) -> SectionRow {
        SectionRow {
            id: title.into(),
            title: title.into(),
            body: body.into(),
            photo: String::new(),
        }
    }

    fn saved(rows: Vec<SectionRow>) -> ModuleConfig {
        ModuleConfig {
            sections: Some(rows),
            ..ModuleConfig::default()
        }
    }

    fn messages(config: &ModuleConfig) -> Vec<(String, String, bool)> {
        config
            .problems()
            .into_iter()
            .map(|p| (p.id, p.message.get("fr").to_string(), p.blocking))
            .collect()
    }

    #[test]
    fn a_title_and_a_body_are_required_with_the_spec_messages() {
        let config = saved(vec![row("", "")]);
        assert_eq!(
            messages(&config),
            [
                (
                    "config.sections[0].title".into(),
                    "Donnez un titre à la section.".into(),
                    true
                ),
                (
                    "config.sections[0].body".into(),
                    "Écrivez quelques mots.".into(),
                    true
                ),
            ]
        );
    }

    #[test]
    fn ten_sections_at_most() {
        let config = saved((0..11).map(|i| row(&format!("S{i}"), "Texte.")).collect());
        assert_eq!(
            messages(&config)[0],
            (
                "config.sections".into(),
                "10 sections au maximum.".into(),
                true
            )
        );
        assert!(
            saved((0..10).map(|i| row(&format!("S{i}"), "Texte.")).collect())
                .problems()
                .is_empty()
        );
    }

    #[test]
    fn a_title_of_sixty_and_a_body_of_three_thousand() {
        let config = saved(vec![row(&"t".repeat(61), &"b".repeat(3_001))]);
        let found = messages(&config);
        assert!(found.contains(&(
            "config.sections[0].title".into(),
            "60 caractères au maximum.".into(),
            true
        )));
        assert!(found.contains(&(
            "config.sections[0].body".into(),
            "3000 caractères au maximum.".into(),
            true
        )));
    }

    #[test]
    fn a_long_first_sentence_warns_without_blocking() {
        let long = format!("{}. Fin.", "mot ".repeat(45));
        let config = saved(vec![row("Bienvenue", &long), row("Suite", &long)]);
        assert_eq!(
            messages(&config),
            [(
                "config.sections[0].body".into(),
                "Votre première phrase sert d'accroche : gardez-la courte.".into(),
                false
            )]
        );
        assert!(teaser_too_long(&config.problems()));
        assert!(!teaser_too_long(
            &saved(vec![row("Bienvenue", "Court. Puis long.")]).problems()
        ));
    }

    #[test]
    fn the_first_sentence_stops_at_its_mark() {
        assert_eq!(first_sentence("Bonjour ! Bienvenue."), "Bonjour !");
        assert_eq!(first_sentence("Version 1.5 du lieu"), "Version 1.5 du lieu");
        assert_eq!(first_sentence("Une seule phrase."), "Une seule phrase.");
    }

    #[test]
    fn show_from_reads_its_wire_values() {
        let config: ModuleConfig =
            serde_json::from_value(serde_json::json!({ "show_from": "arrival_day" })).unwrap();
        assert_eq!(config.show_from, ShowFrom::ArrivalDay);
        assert_eq!(ModuleConfig::default().show_from, ShowFrom::DayBefore);
        for option in ShowFrom::ALL {
            assert_eq!(ShowFrom::parse(option.as_wire()), Some(option));
        }
    }
}
