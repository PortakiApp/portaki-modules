//! Guest Accueil formalities card — composes host police fragment + pre-arrival task.
//!
//! Design (`Portaki Guest.dc.html` `policeBanner` / `arrivalTasks`): one tinted banner with
//! checklist rows. Police UI is a host fragment; this module never owns regulatory fields.

use portaki_sdk::contracts::host_fragments;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, HostFragment, ListItem, Stack};
use portaki_sdk::sdui::surface::Surface;

use crate::config::FormQuestions;
use crate::entities::PreArrivalResponse;

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
                    .chevron(true)
                    // Reopen overlay to edit until check-in, or review after.
                    .action(open_form)
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
///
/// When `existing` is set, fields are prefilled so the guest can edit / resubmit
/// until check-in.
pub fn build_form_surface(
    questions: &FormQuestions,
    existing: Option<&PreArrivalResponse>,
    completed: bool,
) -> Surface {
    use portaki_sdk::sdui::primitives::{
        Button, Field, Form, Text, TextArea, TimePicker,
    };

    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);
    let submit_label = if completed {
        "i18n:form.submitUpdate"
    } else {
        "i18n:form.submit"
    };
    let mut form_children: Vec<Component> = Vec::new();

    form_children.push(
        Text::new()
            .text("i18n:home.card.intro")
            .variant(TextVariant::Body)
            .into(),
    );

    if questions.ask_arrival_time {
        let mut picker = TimePicker::new().name("arrivalTimeEstimated");
        if let Some(value) = existing.and_then(|row| row.arrival_time.as_deref()) {
            picker = picker.value(value);
        }
        form_children.push(
            Field::new()
                .name("arrivalTimeEstimated")
                .label("i18n:form.arrival.label")
                .required(true)
                .child(picker)
                .into(),
        );
    }
    if questions.ask_occasion {
        form_children.push(
            Field::new()
                .name("guestOccasion")
                .label("i18n:form.occasion.label")
                .child(text_input(
                    "guestOccasion",
                    "i18n:form.occasion.placeholder",
                    existing.and_then(|row| row.occasion.as_deref()),
                ))
                .into(),
        );
    }
    if questions.ask_allergies {
        form_children.push(
            Field::new()
                .name("guestAllergies")
                .label("i18n:form.allergies.label")
                .child(text_input(
                    "guestAllergies",
                    "i18n:form.allergies.placeholder",
                    existing.and_then(|row| row.allergies.as_deref()),
                ))
                .into(),
        );
    }
    if questions.ask_guest_count {
        form_children.push(
            Field::new()
                .name("guestCount")
                .label("i18n:form.guestCount.label")
                .child(text_input(
                    "guestCount",
                    "i18n:form.guestCount.placeholder",
                    existing.and_then(|row| row.guest_count.as_deref()),
                ))
                .into(),
        );
    }
    if questions.ask_special_needs {
        form_children.push(
            Field::new()
                .name("specialNeeds")
                .label("i18n:form.specialNeeds.label")
                .child(text_input(
                    "specialNeeds",
                    "i18n:form.specialNeeds.placeholder",
                    existing.and_then(|row| row.special_needs.as_deref()),
                ))
                .into(),
        );
    }
    if questions.ask_id_document {
        form_children.push(
            Field::new()
                .name("idDocument")
                .label("i18n:form.idDocument.label")
                .child(text_input(
                    "idDocument",
                    "i18n:form.idDocument.placeholder",
                    existing.and_then(|row| row.id_document.as_deref()),
                ))
                .into(),
        );
    }

    form_children.push(
        Field::new()
            .name("messageToHost")
            .label("i18n:form.message.label")
            .child({
                let mut area = TextArea::new()
                    .name("messageToHost")
                    .placeholder("i18n:form.message.placeholder");
                if let Some(value) = existing.and_then(|row| row.guest_message.as_deref()) {
                    area = area.value(value);
                }
                area
            })
            .into(),
    );
    form_children.push(Button::new().label(submit_label).action(submit_action).into());

    // Page chrome owns the title; body is the form only (no nested Card).
    Surface::new(Form::new().children(form_children)).with_id(crate::ids::GUEST_FORM)
}

/// Read-only summary after check-in (answers no longer editable).
pub fn build_readonly_surface(
    questions: &FormQuestions,
    response: &PreArrivalResponse,
) -> Surface {
    use portaki_sdk::sdui::primitives::Text;

    let mut children: Vec<Component> = Vec::new();
    children.push(
        Text::new()
            .text("i18n:home.card.thanks")
            .variant(TextVariant::Body)
            .into(),
    );
    children.push(
        Text::new()
            .text("i18n:home.card.lockedHint")
            .variant(TextVariant::Caption)
            .into(),
    );

    if questions.ask_arrival_time {
        children.push(readonly_row(
            "clock-circle",
            "i18n:form.arrival.label",
            display_or_dash(response.arrival_time.as_deref()),
        ));
    }
    if questions.ask_occasion {
        children.push(readonly_row(
            "star",
            "i18n:form.occasion.label",
            display_or_dash(response.occasion.as_deref()),
        ));
    }
    if questions.ask_allergies {
        children.push(readonly_row(
            "danger-triangle",
            "i18n:form.allergies.label",
            display_or_dash(response.allergies.as_deref()),
        ));
    }
    if questions.ask_guest_count {
        children.push(readonly_row(
            "users",
            "i18n:form.guestCount.label",
            display_or_dash(response.guest_count.as_deref()),
        ));
    }
    if questions.ask_special_needs {
        children.push(readonly_row(
            "home",
            "i18n:form.specialNeeds.label",
            display_or_dash(response.special_needs.as_deref()),
        ));
    }
    if questions.ask_id_document {
        children.push(readonly_row(
            "clipboard",
            "i18n:form.idDocument.label",
            display_or_dash(response.id_document.as_deref()),
        ));
    }
    if let Some(message) = response
        .guest_message
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        children.push(readonly_row(
            "message",
            "i18n:form.message.label",
            message.to_string(),
        ));
    }

    Surface::new(Stack::new().gap(8.0).children(children)).with_id(crate::ids::GUEST_FORM)
}

fn text_input(
    name: &str,
    placeholder: &str,
    value: Option<&str>,
) -> portaki_sdk::sdui::primitives::TextInput {
    use portaki_sdk::sdui::primitives::TextInput;

    let mut input = TextInput::new().name(name).placeholder(placeholder);
    if let Some(value) = value {
        input = input.value(value);
    }
    input
}

fn readonly_row(leading: &str, label_i18n: &str, value: String) -> Component {
    ListItem::new()
        .title(label_i18n)
        .subtitle(value)
        .leading(leading)
        .chevron(false)
        .into()
}

fn display_or_dash(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("—")
        .to_string()
}
