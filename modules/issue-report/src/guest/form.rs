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
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    let config = crate::config::ModuleConfig::load(&ctx)?;
    Ok(build_form_surface(&config.categories()))
}

pub fn build_form_surface(categories: &[&str]) -> Surface {
    Surface::new(build_form(categories)).with_id(GUEST_FORM)
}

fn build_form(categories: &[&str]) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);

    Form::new()
        .child(
            Field::new()
                .name("category")
                .label("i18n:form.category.label")
                .required(true)
                .child(category_choice_list(categories)),
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

fn category_choice_list(categories: &[&str]) -> ChoiceList {
    ChoiceList::new()
        .name("category")
        .layout(ChoiceListLayout::Compact)
        .choices(categories.iter().copied().map(category_option).collect())
}

/// La pastille d'une catégorie — son libellé et son icône.
fn category_option(wire: &str) -> ChoiceOption {
    let option = ChoiceOption::new(
        wire,
        format!("i18n:{}", crate::category::category_label_key(wire)),
    );
    match wire {
        "appliance" => option.icon(IconName::Plug),
        "cleanliness" => option.icon(IconName::Sparkles),
        "noise" => option.icon(IconName::Volume2),
        "access" => option.icon(IconName::Key),
        _ => option.icon(IconName::MessageCircle),
    }
}
