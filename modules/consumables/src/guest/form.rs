//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, ChoiceList, Field, Form, TextArea};
use portaki_sdk::sdui::surface::Surface;

use super::empty::{empty_runtime_error_state, log_render_failure};
use super::load::{load_guest_consumables, GuestConsumablesData, GuestLoad};
use crate::labels;
use crate::level;

/// Bottom-sheet consumables shortage form (inputs live here — not on the home card).
#[portaki_sdk::surface(guest, id = "guest.form")]
pub fn render_guest_form(ctx: GuestContext) -> Surface {
    match render_form(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::GUEST_FORM, &error);
            empty_runtime_error_state(crate::ids::GUEST_FORM)
        }
    }
}

fn render_form(ctx: &GuestContext) -> Result<Surface> {
    match load_guest_consumables(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::Ready(data) => Ok(build_form_surface(&data)),
    }
}

pub fn build_form_surface(data: &GuestConsumablesData) -> Surface {
    Surface::new(build_form(data)).with_id(crate::ids::GUEST_FORM)
}

fn build_form(data: &GuestConsumablesData) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);
    let first_id = data
        .items
        .first()
        .map(|item| item.id.to_string())
        .unwrap_or_default();

    Form::new()
        .child(
            Field::new()
                .name("itemId")
                .label("i18n:form.item.label")
                .required(true)
                .child(item_choice_list(data, &first_id)),
        )
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
}

fn item_choice_list(data: &GuestConsumablesData, selected: &str) -> ChoiceList {
    let choices: Vec<ChoiceOption> = data
        .items
        .iter()
        .map(|item| {
            let label = labels::pick_label(
                &labels::labels_from_item(item),
                &data.locale,
                &data.property_locale,
            );
            ChoiceOption::new(item.id.to_string(), label).icon("package")
        })
        .collect();

    let mut list = ChoiceList::new()
        .name("itemId")
        .layout(ChoiceListLayout::Compact)
        .choices(choices);
    if !selected.is_empty() {
        list = list.value(selected);
    }
    list
}

fn level_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("level")
        .layout(ChoiceListLayout::Compact)
        .value(level::DEFAULT)
        .choices(vec![
            ChoiceOption::new("missing", "i18n:form.level.missing").icon("circle-x"),
            ChoiceOption::new("low", "i18n:form.level.low").icon("gauge"),
        ])
}
