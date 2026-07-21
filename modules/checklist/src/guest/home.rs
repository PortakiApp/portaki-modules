//! Guest home booklet card — progress + inline toggles.

use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Card, ChecklistItem as ChecklistItemView, Pressable, Text};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::load::GuestChecklistData;
use crate::labels;

pub fn build_home_card(data: &GuestChecklistData) -> Surface {
    let progress = format!("{} / {} — {}%", data.done, data.total, data.percent);

    let mut children = vec![Text::new()
        .text(json!(progress))
        .variant(json!("caption"))
        .into()];

    for item in &data.items {
        let checked = data.completed.contains(&item.id);
        let label = labels::pick_label(
            &labels::labels_from_item(item),
            &data.locale,
            &data.property_locale,
        );
        let command_name = if checked {
            "uncompleteItem"
        } else {
            "completeItem"
        };
        let action = serde_json::to_value(Action::command(
            "checklist",
            command_name,
            json!({ "itemId": item.id }),
        ))
        .unwrap_or(json!({}));

        children.push(
            Pressable::new()
                .action(action)
                .child(
                    ChecklistItemView::new()
                        .label(json!(label))
                        .checked(json!(checked)),
                )
                .into(),
        );
    }

    Surface::new(
        Card::new()
            .icon(json!("list-checks"))
            .title(json!("i18n:home.card.title"))
            .children(children),
    )
    .with_id("home.card")
}
