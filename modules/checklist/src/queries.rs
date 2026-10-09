//! Module queries — guest items and stay completions.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;

use crate::entities::{Checklist, ChecklistItem};
use crate::i18n::text;
use crate::lists;
use crate::storage;

/// Public item DTO returned by `listItems`.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct ChecklistItemDto {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
}

impl From<ChecklistItem> for ChecklistItemDto {
    fn from(value: ChecklistItem) -> Self {
        Self {
            id: value.id,
            checklist_id: value.checklist_id,
            label_fr: value.label_fr,
            label_en: value.label_en,
            sort_order: value.sort_order,
        }
    }
}

/// Items of every guest list, list by list.
pub fn guest_items() -> Result<Vec<ChecklistItem>> {
    let mut items = Vec::new();
    for list in storage::list_checklists()? {
        if list.audience == lists::GUEST {
            items.extend(storage::items_of(list.id)?);
        }
    }
    Ok(items)
}

#[portaki_sdk::query(name = "listItems", example(label = "Tâches du livret"))]
pub fn list_items(_ctx: Context) -> Result<Vec<ChecklistItemDto>> {
    Ok(guest_items()?
        .into_iter()
        .map(ChecklistItemDto::from)
        .collect())
}

#[portaki_sdk::query(name = "listCompletions", example(label = "Tâches cochées du séjour"))]
pub fn list_completions(ctx: Context) -> Result<Vec<Uuid>> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;
    Ok(storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect())
}

/// Ce qui bloque la publication : une étape au moins, puis pour chaque liste ce que
/// [`list_problems`] relève ; et l'avertissement d'un code écrit en clair dans une étape du livret
/// (spec Checklist §3).
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let all = storage::all_items()?;
    let mut items = vec![PublishCheck {
        id: "checklist".into(),
        level: PublishLevel::Required,
        ok: !storage::list_items()?.is_empty(),
        label: text("publish.checklist.label", &[]),
        hint: text("publish.checklist.hint", &[]),
    }];
    for list in storage::list_checklists()? {
        let steps: Vec<&ChecklistItem> = all.iter().filter(|i| i.checklist_id == list.id).collect();
        let name = list.name_fr.clone();
        for (field, error) in list_problems(&list, &steps) {
            // `steps` / `labels` keep the ids they had before the display settings.
            let id = match field {
                "steps" | "labels" => format!("{field}.{}", list.id),
                _ => format!("config.{field}.{}", list.id),
            };
            items.push(PublishCheck {
                id,
                level: PublishLevel::Required,
                ok: false,
                label: text("publish.steps.label", &[("list", &name)]),
                hint: error,
            });
        }
        // Un code dans une étape du livret se lit sans calendrier de révélation.
        if list.audience == lists::GUEST
            && steps.iter().any(|step| {
                crate::labels::labels_from_item(step)
                    .values()
                    .chain(crate::labels::decode_map(&step.description_i18n).values())
                    .any(|text| looks_like_code(text))
            })
        {
            items.push(PublishCheck {
                id: format!("code.{}", list.id),
                level: PublishLevel::Recommended,
                ok: false,
                label: text("publish.steps.label", &[("list", &name)]),
                hint: text("publish.code.hint", &[]),
            });
        }
    }
    Ok(PublishReadiness { items })
}

/// `(champ, message)` of one list, as stored — the same list feeds `Field::error` in the editor
/// and `publishReadiness`, so both say the same thing (spec Checklist §2.1, §2.2).
///
/// `steps`: more than [`MAX_STEPS`]. `labels`: the first step at fault — blank, label over
/// [`LABEL_MAX`], group over [`GROUP_MAX`], detail over [`DETAIL_MAX`]. `visible_limit` and
/// `done_message`: guest lists only.
pub fn list_problems(list: &Checklist, steps: &[&ChecklistItem]) -> Vec<(&'static str, I18nText)> {
    let mut problems = Vec::new();
    if steps.len() > MAX_STEPS {
        problems.push(("steps", text("publish.steps.tooMany", &[])));
    }
    if let Some(error) = steps.iter().find_map(|step| step_problem(step)) {
        problems.push(("labels", error));
    }
    if list.audience != lists::GUEST {
        return problems;
    }
    let display = storage::display::read(list.id);
    if let Some(error) = display.visible_limit.and_then(|n| {
        check::between(
            f64::from(n),
            f64::from(storage::display::MIN_VISIBLE_LIMIT),
            f64::from(storage::display::MAX_VISIBLE_LIMIT),
        )
    }) {
        problems.push(("visible_limit", error));
    }
    if let Some(error) = crate::labels::decode_map(&display.done_message)
        .values()
        .find_map(|message| check::max_chars(message, DONE_MESSAGE_MAX))
    {
        problems.push(("done_message", error));
    }
    problems
}

fn step_problem(step: &ChecklistItem) -> Option<I18nText> {
    let labels = crate::labels::labels_from_item(step);
    if labels.is_empty() {
        return Some(text("publish.step.required", &[]));
    }
    let over = |raw: &str, max: usize| {
        crate::labels::decode_map(raw)
            .values()
            .find_map(|value| check::max_chars(value, max))
    };
    labels
        .values()
        .find_map(|label| check::max_chars(label, LABEL_MAX))
        .or_else(|| over(&step.group_i18n, GROUP_MAX))
        .or_else(|| over(&step.description_i18n, DETAIL_MAX))
}

/// Combien d'étapes une liste accepte, et la longueur d'une étape (spec Checklist §2.2).
pub const MAX_STEPS: usize = 40;
pub const LABEL_MAX: usize = 80;
pub const GROUP_MAX: usize = 30;
pub const DETAIL_MAX: usize = 120;
/// « Message final » (spec Checklist §2.1).
pub const DONE_MESSAGE_MAX: usize = 120;

/// « Code 4821 », « digicode : 1234A » : le mot, ou une suite de quatre chiffres et plus.
///
/// ponytail: heuristique — un numéro de rue à quatre chiffres déclenche aussi l'avertissement,
/// qui ne bloque pas ; un code en lettres passe. Assez pour le cas que la spec décrit.
fn looks_like_code(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("code") {
        return true;
    }
    let mut run = 0;
    for c in lower.chars() {
        run = if c.is_ascii_digit() { run + 1 } else { 0 };
        if run >= 4 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::looks_like_code;

    #[test]
    fn a_code_is_spotted_by_its_word_or_its_digits() {
        assert!(looks_like_code("Code 4821"));
        assert!(looks_like_code("Boîte à clés : 0512"));
        assert!(looks_like_code("Le digicode est sur la porte"));
        assert!(!looks_like_code("Sortir les 3 poubelles"));
        assert!(!looks_like_code(""));
    }
}
