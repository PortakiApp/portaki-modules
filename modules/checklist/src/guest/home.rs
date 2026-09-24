//! Guest home booklet card — progress + inline toggles.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, ChecklistItem as ChecklistItemView, Pressable, Text};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestChecklistData;
use crate::labels;

pub fn build_home_card(data: &GuestChecklistData, surface_id: SurfaceId) -> Surface {
    let progress = format!("{} / {} — {}%", data.done, data.total, data.percent);
    // Title leads with the departure moment ("Départ mardi à 11:00"); the description line invites
    // opening the list. Falls back to the module name until the stay carries a checkout time.
    let title = super::depart::format_departure(data.checkout_at, &data.property_timezone)
        .unwrap_or_else(|| t!("home.card.title").unwrap_or_else(|_| "Checklists".into()));
    let subtitle = t!("guest.rowTeaser").unwrap_or_else(|_| "Voir la checklist de départ".into());

    let mut children = vec![Text::new()
        .text(progress)
        .variant(TextVariant::Caption)
        .into()];

    let fr = labels::lang_code(&data.locale) == "fr";
    for (list, items) in &data.lists {
        if data.lists.len() > 1 {
            let name = if fr { &list.name_fr } else { &list.name_en };
            children.push(
                Text::new()
                    .text(name.clone())
                    .variant(TextVariant::Title)
                    .into(),
            );
        }
        for item in items {
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
    }

    Surface::new(
        Card::new()
            .icon(IconName::ListChecks)
            .title(title)
            .subtitle(subtitle)
            .children(children),
    )
    .with_id(surface_id)
}
