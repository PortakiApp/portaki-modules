//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, Field, Form, ImageUpload, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

/// Bottom-sheet issue report form (inputs live here — not on the home card).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "issue-report/form",
    label_key = "nav.issue-report"
)]
pub fn render_guest_form(_ctx: GuestContext) -> Result<Surface> {
    Ok(build_form_surface())
}

pub fn build_form_surface() -> Surface {
    Surface::new(build_form()).with_id(GUEST_FORM)
}

fn build_form() -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);

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
            Field::new()
                .name("photo")
                .label("i18n:form.photo.label")
                .child(ImageUpload::new().name("photo")),
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
            ChoiceOption::new("appliance", "i18n:form.category.appliance").icon(IconName::Plug),
            ChoiceOption::new("cleanliness", "i18n:form.category.cleanliness")
                .icon(IconName::Sparkles),
            ChoiceOption::new("noise", "i18n:form.category.noise").icon(IconName::Volume2),
            ChoiceOption::new("access", "i18n:form.category.access").icon(IconName::Key),
            ChoiceOption::new("other", "i18n:form.category.other").icon(IconName::MessageCircle),
        ])
}
