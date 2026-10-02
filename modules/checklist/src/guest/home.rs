//! Guest home booklet card — une seule liste à cocher (§2.9).
//!
//! Le module donne les étapes, leur rubrique et leur précision ; le livret rend la barre de
//! progression, le « n / N », le repli au-delà de cinq, les titres de rubrique, les étapes faites
//! barrées en bas et le bandeau de fin. Un module qui aurait dessiné tout ça aurait décidé à la
//! place du livret, et aurait dû le redessiner à chaque emplacement.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, ChoiceList};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestChecklistData;
use crate::labels;

pub fn build_home_card(data: &GuestChecklistData, surface_id: SurfaceId) -> Surface {
    // Le titre nomme le moment, le sous-titre le situe : « Avant de partir », puis « Départ samedi
    // à 10:00 · cochez au fur et à mesure ». Sans date de séjour, le nom du module suffit.
    let title = t!("home.card.beforeLeaving").unwrap_or_else(|_| "i18n:home.card.title".into());
    let subtitle =
        match super::depart::format_departure(data.checkout_at, &data.property_timezone) {
            Some(departure) => t!("guest.cardSubtitle", departure = &departure)
                .unwrap_or_else(|_| departure.clone()),
            None => t!("guest.rowTeaser").unwrap_or_else(|_| "i18n:guest.rowTeaser".into()),
        };

    let named = data.lists.len() > 1;
    let mut choices: Vec<ChoiceOption> = Vec::new();
    for (list, items) in &data.lists {
        for item in items {
            let label = labels::pick_label(
                &labels::labels_from_item(item),
                &data.locale,
                &data.property_locale,
            );
            if label.trim().is_empty() {
                continue;
            }
            let mut choice = ChoiceOption::new(item.id.to_string(), label);
            let description = labels::pick_label(
                &labels::decode_map(&item.description_i18n),
                &data.locale,
                &data.property_locale,
            );
            if !description.trim().is_empty() {
                choice.description = Some(description);
            }
            // La rubrique de l'étape, et à défaut le nom de sa liste quand il y en a plusieurs : le
            // voyageur doit savoir d'où vient une étape, sans qu'on lui annonce une liste unique.
            let group = labels::pick_label(
                &labels::decode_map(&item.group_i18n),
                &data.locale,
                &data.property_locale,
            );
            let group = if group.trim().is_empty() && named {
                labels::pick_label(
                    &labels::labels_from_list(list),
                    &data.locale,
                    &data.property_locale,
                )
            } else {
                group
            };
            if !group.trim().is_empty() {
                choice.group = Some(group);
            }
            choices.push(choice);
        }
    }

    let ticked: Vec<String> = choices
        .iter()
        .filter(|choice| {
            choice
                .value
                .parse()
                .is_ok_and(|id| data.completed.contains(&id))
        })
        .map(|choice| choice.value.clone())
        .collect();

    Surface::new(
        Card::new()
            .icon(IconName::ListChecks)
            .title(title)
            .subtitle(subtitle)
            .child(
                ChoiceList::new()
                    .name("item_ids")
                    .layout(ChoiceListLayout::Checklist)
                    .multi(true)
                    .limit(CARD_GLANCE_LIMIT)
                    // L'état part du serveur : une coche doit survivre à la fermeture du livret.
                    .value(ticked.join(","))
                    .choices(choices)
                    .emitOnChange(true)
                    .action(crate::ids::module_id().command_empty(crate::commands::SET_COMPLETED))
                    .doneTitle(
                        t!("guest.done.title").unwrap_or_else(|_| "i18n:guest.done.title".into()),
                    )
                    .doneMessage(
                        t!("guest.done.message")
                            .unwrap_or_else(|_| "i18n:guest.done.message".into()),
                    ),
            ),
    )
    .with_id(surface_id)
}

/// Cinq étapes non faites visibles, le reste replié derrière son propre bouton (§2.9).
const CARD_GLANCE_LIMIT: u32 = 5;
