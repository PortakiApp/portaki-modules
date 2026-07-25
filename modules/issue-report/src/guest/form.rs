//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, ChoiceList, Field, Form, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;

use super::empty::{empty_runtime_error_state, log_render_failure};
use super::load::{load_guest_reports, GuestLoad};

/// Bottom-sheet issue report form (inputs live here — not on the home card).
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
    match load_guest_reports(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::Ready(_) => Ok(build_form_surface()),
    }
}

pub fn build_form_surface() -> Surface {
    Surface::new(build_form()).with_id(crate::ids::GUEST_FORM)
}

fn build_form() -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);

    Form::new()
        .child(
            Field::new()
                .name("category")
                .label("i18n:form.category.label")
                .required(true)
                .child(category_choice_list()),
        )
        .child(
            Field::new()
                .name("summary")
                .label("i18n:form.summary.label")
                .required(true)
                .child(
                    TextInput::new()
                        .name("summary")
                        .placeholder("i18n:form.summary.placeholder"),
                ),
        )
        .child(
            Field::new()
                .name("details")
                .label("i18n:form.details.label")
                .child(
                    TextArea::new()
                        .name("details")
                        .placeholder("i18n:form.details.placeholder"),
                ),
        )
        .child(
            Button::new()
                .label("i18n:form.submit")
                .action(submit_action),
        )
}

fn category_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("category")
        .layout(ChoiceListLayout::Compact)
        .choices(vec![
            ChoiceOption::new("appliance", "i18n:form.category.appliance").icon("plug"),
            ChoiceOption::new("cleanliness", "i18n:form.category.cleanliness").icon("sparkles"),
            ChoiceOption::new("noise", "i18n:form.category.noise").icon("volume-2"),
            ChoiceOption::new("access", "i18n:form.category.access").icon("key"),
            ChoiceOption::new("other", "i18n:form.category.other").icon("message-circle"),
        ])
}
