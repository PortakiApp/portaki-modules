//! Guest home booklet card — progress + inline toggles.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, ChecklistItem as ChecklistItemView, Pressable, Text};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestChecklistData;
use crate::labels;

pub fn build_home_card(data: &GuestChecklistData) -> Surface {
    let progress = format!("{} / {} — {}%", data.done, data.total, data.percent);
    // Nav-row summary on the description line, e.g. "3/6 complété".
    let row_summary = t!("guest.rowSummary", done = data.done, total = data.total)
        .unwrap_or_else(|_| format!("{} / {}", data.done, data.total));

    let mut children = vec![Text::new()
        .text(progress)
        .variant(TextVariant::Caption)
        .into()];

    for item in &data.items {
        let checked = data.completed.contains(&item.id);
        let label = labels::pick_label(
            &labels::labels_from_item(item),
            &data.locale,
            &data.property_locale,
        );
        let command_name = if checked {
            crate::ids::UNCOMPLETE_ITEM
        } else {
            crate::ids::COMPLETE_ITEM
        };
        let action = crate::ids::module_id().command(
            command_name,
            crate::commands::ItemIdArgs { item_id: item.id },
        );

        children.push(
            Pressable::new()
                .action(action)
                .child(ChecklistItemView::new().label(label).checked(checked))
                .into(),
        );
    }

    Surface::new(
        Card::new()
            .icon("list-checks")
            .title("i18n:home.card.title")
            .subtitle(row_summary)
            .children(children),
    )
    .with_id(crate::ids::HOME_CARD)
}
