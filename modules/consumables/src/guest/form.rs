//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, Field, FieldHint, Form, InfoBanner, TextArea,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::{load_guest_consumables, GuestConsumablesData};
use crate::labels;
use crate::level;

/// Bottom-sheet consumables shortage form (inputs live here — not on the home card).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "consumables/form",
    label_key = "nav.consumables"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    Ok(match load_guest_consumables(&ctx)? {
        Some(data) => build_form_surface(&data),
        None => super::empty_catalog_card(GUEST_FORM),
    })
}

pub fn build_form_surface(data: &GuestConsumablesData) -> Surface {
    Surface::new(build_form(data)).with_id(GUEST_FORM)
}

fn build_form(data: &GuestConsumablesData) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);
    Form::new()
        .child(
            Field::new()
                .name("itemIds")
                .label("i18n:form.item.label")
                .required(true)
                .child(item_choice_list(data)),
        )
        // Le choix multiple n'est pas évident dans une grille de tuiles : un voyageur qui a
        // besoin de café et de sacs poubelle envoyait deux signalements.
        .child(FieldHint::new().text("i18n:form.item.hint"))
        .child(
            Field::new()
                .name("level")
                .label("i18n:form.level.label")
                .required(true)
                .child(level_choice_list()),
        )
        .child(
            Field::new()
                .name("note")
                .label("i18n:form.note.label")
                .child(
                    TextArea::new()
                        .name("note")
                        .placeholder("i18n:form.note.placeholder"),
                ),
        )
        .child(
            Button::new()
                .label("i18n:form.submit")
                .action(submit_action),
        )
        // Ce que l'envoi déclenche, dit par le module et non par l'hôte : c'est vrai de tous les
        // logements, et un hôte qui n'aurait pas rempli ses horaires ne doit pas laisser le
        // voyageur se demander si quelqu'un l'a lu.
        .child(
            InfoBanner::new()
                .tone(Tone::Neutral)
                .message("i18n:form.notice"),
        )
}

/// Rien n'est présélectionné : en choix multiple, une case déjà cochée part au signalement sans
/// que le voyageur l'ait voulu, et c'est l'hôte qui se déplace pour rien.
fn item_choice_list(data: &GuestConsumablesData) -> ChoiceList {
    let choices: Vec<ChoiceOption> = data
        .items
        .iter()
        .map(|item| {
            let label = labels::pick_label(
                &labels::labels_from_item(item),
                &data.locale,
                &data.property_locale,
            );
            ChoiceOption::new(item.id.to_string(), label).icon(IconName::Package)
        })
        .collect();

    // Grille et choix multiple (§2.5). Un voyageur qui constate qu'il manque le papier *et* le café
    // le disait en deux envois ; la liste compacte à choix unique l'y obligeait.
    ChoiceList::new()
        .name("itemIds")
        .layout(ChoiceListLayout::Grid)
        .multi(true)
        .choices(choices)
}

fn level_choice_list() -> ChoiceList {
    // Deux choix courts côte à côte : c'est ce que le §2.5 appelle des segments.
    ChoiceList::new()
        .name("level")
        .layout(ChoiceListLayout::Segmented)
        .value(level::DEFAULT)
        .choices(vec![
            // « Il n'y en a plus » porte le ton d'alerte du dessin : les deux segments ne
            // demandent pas la même chose à l'hôte.
            ChoiceOption::new("missing", "i18n:form.level.missing").icon(IconName::DangerTriangle),
            ChoiceOption::new("low", "i18n:form.level.low").icon(IconName::Gauge),
        ])
}
