//! Les réponses aux questions de l'hôte (§2.2), gardées avec la question telle qu'elle était
//! posée : l'hôte qui la réécrit ou la retire ne change pas ce que le voyageur a répondu.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{CustomQuestion, ModuleConfig, QuestionKind, MAX_CUSTOM_QUESTIONS};
use crate::entities::PreArrivalResponse;

/// Une réponse, et la question qui l'a demandée.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomAnswer {
    /// L'identifiant de la ligne du formulaire hôte, pour retrouver la question réécrite.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub question: I18nText,
    pub answer: Answer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Answer {
    Text(String),
    YesNo(bool),
    /// L'option choisie, dans toutes ses langues.
    Choice(I18nText),
}

/// Le nom du champ de la question `index` dans le formulaire du voyageur (`custom0`…).
pub fn field_name(index: usize) -> String {
    format!("custom{index}")
}

/// Les réponses envoyées, question par question ; une question obligatoire sans réponse
/// valable refuse l'envoi.
pub fn collect(
    config: &ModuleConfig,
    raw: [Option<String>; MAX_CUSTOM_QUESTIONS],
) -> Result<Vec<CustomAnswer>> {
    let mut answers = Vec::new();
    for (question, raw) in config.guest_questions().zip(raw) {
        let raw = raw
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let answer = raw.and_then(|value| parse(question, &value));
        match answer {
            Some(answer) => answers.push(CustomAnswer {
                id: question.id.clone(),
                question: question.label.clone(),
                answer,
            }),
            None if question.required => {
                return Err(PortakiError::Host("custom_answer_required".to_string()))
            }
            None => {}
        }
    }
    Ok(answers)
}

/// Une valeur hors de ce que la question propose ne compte pas comme une réponse.
fn parse(question: &CustomQuestion, value: &str) -> Option<Answer> {
    match question.kind {
        QuestionKind::Text => Some(Answer::Text(value.chars().take(500).collect())),
        QuestionKind::YesNo => match value {
            "yes" => Some(Answer::YesNo(true)),
            "no" => Some(Answer::YesNo(false)),
            _ => None,
        },
        QuestionKind::Choice => value
            .parse::<usize>()
            .ok()
            .and_then(|index| question.options().get(index).map(|label| (*label).clone()))
            .map(Answer::Choice),
    }
}

/// Les réponses gardées ; rien pour une réponse d'avant ces questions ou illisible.
pub fn stored(row: &PreArrivalResponse) -> Vec<CustomAnswer> {
    row.custom_answers
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default()
}

/// La valeur qui pré-remplit `question` : la réponse gardée à la même question (même ligne, ou
/// même libellé), s'il y en a une.
pub fn prefill(question: &CustomQuestion, stored: &[CustomAnswer]) -> Option<String> {
    let same = |a: &&CustomAnswer| {
        if question.id.is_empty() {
            a.question == question.label
        } else {
            a.id == question.id
        }
    };
    let answer = &stored.iter().find(same)?.answer;
    match answer {
        Answer::Text(text) => Some(text.clone()),
        Answer::YesNo(yes) => Some(if *yes { "yes" } else { "no" }.to_string()),
        Answer::Choice(label) => question
            .options()
            .iter()
            .position(|option| *option == label)
            .map(|index| index.to_string()),
    }
}

/// La réponse, lue dans la langue de `ctx`.
pub fn display(answer: &Answer, ctx: &Context) -> String {
    match answer {
        Answer::Text(text) => text.clone(),
        Answer::YesNo(true) => "i18n:form.custom.yes".to_string(),
        Answer::YesNo(false) => "i18n:form.custom.no".to_string(),
        Answer::Choice(label) => label.for_ctx(ctx).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QuestionOption;

    fn question(kind: QuestionKind, required: bool) -> CustomQuestion {
        CustomQuestion {
            id: kind.as_wire().into(),
            label: I18nText::new(kind.as_wire(), kind.as_wire()),
            kind,
            options: ["Oui", "Non merci"]
                .into_iter()
                .map(|label| QuestionOption {
                    label: I18nText::new(label, label),
                })
                .collect(),
            required,
        }
    }

    fn config(questions: Vec<CustomQuestion>) -> ModuleConfig {
        ModuleConfig {
            custom_questions: questions,
            ..ModuleConfig::default()
        }
    }

    #[test]
    fn answers_follow_their_question() {
        let config = config(vec![
            question(QuestionKind::YesNo, false),
            question(QuestionKind::Choice, false),
            question(QuestionKind::Text, false),
        ]);
        let answers = collect(
            &config,
            [
                Some("yes".into()),
                Some("1".into()),
                Some("  canapé  ".into()),
                None,
                None,
            ],
        )
        .unwrap();
        assert_eq!(
            answers.iter().map(|a| &a.answer).collect::<Vec<_>>(),
            [
                &Answer::YesNo(true),
                &Answer::Choice(I18nText::new("Non merci", "Non merci")),
                &Answer::Text("canapé".into()),
            ]
        );
        // Hors liste : pas une réponse.
        let none = collect(
            &config,
            [Some("maybe".into()), Some("9".into()), None, None, None],
        )
        .unwrap();
        assert!(none.is_empty());
        // Et la réponse revient pré-remplie.
        assert_eq!(
            prefill(&config.custom_questions[1], &answers).as_deref(),
            Some("1")
        );
    }

    #[test]
    fn a_required_question_needs_an_answer() {
        let config = config(vec![question(QuestionKind::YesNo, true)]);
        let refused = collect(&config, [None, None, None, None, None]).unwrap_err();
        assert!(refused.to_string().contains("custom_answer_required"));
        assert!(collect(&config, [Some("no".into()), None, None, None, None]).is_ok());
    }
}
