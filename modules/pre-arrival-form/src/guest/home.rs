//! Guest Accueil formalities card — composes host police fragment + pre-arrival task.
//!
//! Design (`Portaki Guest.dc.html` `policeBanner` / `arrivalTasks`): one tinted banner with
//! checklist rows. Police UI is a host fragment; this module never owns regulatory fields.

use portaki_sdk::contracts::host_fragments;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, HostFragment, ListItem, Stack};
use portaki_sdk::sdui::surface::Surface;

use crate::config::FormQuestions;

pub enum FormTaskState {
    NotYet,
    Pending,
    Done,
}

/// Accueil card matching design « Avant votre arrivée » formalities banner.
///
/// When the form is gated (`NotYet`), omit the form row / soon teaser entirely —
/// keep only the host police fragment. Guest shell hides the card if neither task
/// is visible (police not required).
pub fn build_formalities_card(form_state: FormTaskState) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::Fullscreen,
        crate::ids::GUEST_FORM,
        OverlayArgs::new()
            .icon("clipboard")
            .title("i18n:home.card.title"),
    );

    let subtitle = match form_state {
        FormTaskState::Done => "i18n:home.formalities.allReady",
        // Gated: no form teaser — shell progress subtitle covers police-only case.
        FormTaskState::NotYet | FormTaskState::Pending => "i18n:home.formalities.pending",
    };

    let icon = match form_state {
        FormTaskState::Done => "check-circle",
        FormTaskState::NotYet | FormTaskState::Pending => "clock-circle",
    };

    let mut children: Vec<Component> = Vec::new();

    // Host-owned police task row — shell renders or omits when not required.
    children.push(
        HostFragment::new()
            .fragmentId(host_fragments::POLICE_FORM.as_str())
            .mode("taskRow")
            .into(),
    );

    match form_state {
        FormTaskState::Done => {
            children.push(
                ListItem::new()
                    .title("i18n:home.task.preArrival.label")
                    .subtitle("i18n:home.task.completed")
                    .leading("clipboard")
                    .chevron(false)
                    .into(),
            );
        }
        // Gated: no ListItem / notYet copy — police fragment may still show.
        FormTaskState::NotYet => {}
        FormTaskState::Pending => {
            children.push(
                ListItem::new()
                    .title("i18n:home.task.preArrival.label")
                    .subtitle("i18n:home.task.preArrival.sub")
                    .leading("clipboard")
                    .chevron(true)
                    .action(open_form)
                    .into(),
            );
        }
    }

    Surface::new(
        Card::new()
            .icon(icon)
            .title("i18n:home.card.title")
            .subtitle(subtitle)
            .tone(match form_state {
                FormTaskState::Done => Tone::Success,
                FormTaskState::NotYet | FormTaskState::Pending => Tone::Primary,
            })
            .child(Stack::new().gap(0.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
}

/// Fullscreen overlay form body (design `prearrivalBody` — no nested Card chrome).
pub fn build_form_surface(questions: &FormQuestions) -> Surface {
    use portaki_sdk::sdui::primitives::{
        Button, Field, Form, Text, TextArea, TextInput, TimePicker,
    };

    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);
    let mut form_children: Vec<Component> = Vec::new();

    form_children.push(
        Text::new()
            .text("i18n:home.card.intro")
            .variant(TextVariant::Body)
            .into(),
    );

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

    // Page chrome owns the title; body is the form only (no nested Card).
    Surface::new(Form::new().children(form_children)).with_id(crate::ids::GUEST_FORM)
}
