//! Guest home booklet card — pre-arrival form or thank-you state.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, Text, TextArea, TextInput, TimePicker,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::FormQuestions;

pub fn build_form_card(questions: &FormQuestions) -> Surface {
    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);

    let mut form_children: Vec<Component> = Vec::new();

    if questions.ask_arrival_time {
        form_children.push(
            Field::new()
                .name("arrivalTimeEstimated")
                .label("i18n:form.arrival.label")
                .required(true)
                .child(TimePicker::new().name("arrivalTimeEstimated"))
                .into(),
        );
    }
    if questions.ask_occasion {
        form_children.push(
            Field::new()
                .name("guestOccasion")
                .label("i18n:form.occasion.label")
                .child(
                    TextInput::new()
                        .name("guestOccasion")
                        .placeholder("i18n:form.occasion.placeholder"),
                )
                .into(),
        );
    }
    if questions.ask_allergies {
        form_children.push(
            Field::new()
                .name("guestAllergies")
                .label("i18n:form.allergies.label")
                .child(
                    TextInput::new()
                        .name("guestAllergies")
                        .placeholder("i18n:form.allergies.placeholder"),
                )
                .into(),
        );
    }
    if questions.ask_guest_count {
        form_children.push(
            Field::new()
                .name("guestCount")
                .label("i18n:form.guestCount.label")
                .child(
                    TextInput::new()
                        .name("guestCount")
                        .placeholder("i18n:form.guestCount.placeholder"),
                )
                .into(),
        );
    }
    if questions.ask_special_needs {
        form_children.push(
            Field::new()
                .name("specialNeeds")
                .label("i18n:form.specialNeeds.label")
                .child(
                    TextInput::new()
                        .name("specialNeeds")
                        .placeholder("i18n:form.specialNeeds.placeholder"),
                )
                .into(),
        );
    }
    if questions.ask_id_document {
        form_children.push(
            Field::new()
                .name("idDocument")
                .label("i18n:form.idDocument.label")
                .child(
                    TextInput::new()
                        .name("idDocument")
                        .placeholder("i18n:form.idDocument.placeholder"),
                )
                .into(),
        );
    }

    form_children.push(
        Field::new()
            .name("messageToHost")
            .label("i18n:form.message.label")
            .child(
                TextArea::new()
                    .name("messageToHost")
                    .placeholder("i18n:form.message.placeholder"),
            )
            .into(),
    );
    form_children.push(
        Button::new()
            .label("i18n:form.submit")
            .action(submit_action)
            .into(),
    );

    Surface::new(
        Card::new()
            .icon("clipboard-list")
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.intro")
                    .variant(TextVariant::Body),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id(crate::ids::HOME_CARD)
}

pub fn build_completed_card() -> Surface {
    Surface::new(
        Card::new()
            .icon("clipboard-list")
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.thanks")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(crate::ids::HOME_CARD)
}

pub fn build_not_yet_card() -> Surface {
    Surface::new(
        Card::new()
            .icon("clipboard-list")
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.notYet")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(crate::ids::HOME_CARD)
}
