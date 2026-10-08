//! Guest Accueil formalities card — composes host police fragment + pre-arrival task.
//!
//! Design (`Portaki Guest.dc.html` `policeBanner` / `arrivalTasks`): one tinted banner with
//! checklist rows. Police UI is a host fragment; this module never owns regulatory fields.

use portaki_sdk::contracts::host_fragments;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::primitives::{Card, HostFragment, ListItem, Stack};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::entities::PreArrivalResponse;
use crate::slots::Slot;

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
        crate::guest::form::GUEST_FORM,
        OverlayArgs::new()
            .icon(IconName::Clipboard)
            .title("i18n:home.card.title"),
    );

    let subtitle = match form_state {
        FormTaskState::Done => "i18n:home.formalities.allReady",
        // Gated: no form teaser — shell progress subtitle covers police-only case.
        FormTaskState::NotYet | FormTaskState::Pending => "i18n:home.formalities.pending",
    };

    let icon = match form_state {
        FormTaskState::Done => IconName::CheckCircle,
        FormTaskState::NotYet | FormTaskState::Pending => IconName::ClockCircle,
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
                    .leading(Leading::Icon("clipboard".into()))
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
                    .leading(Leading::Icon("clipboard".into()))
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
    .with_id(crate::guest::HOME_CARD)
}

/// Ce que la surface du formulaire a besoin de savoir, en plus des réponses déjà données.
pub struct FormInputs<'a> {
    pub questions: &'a ModuleConfig,
    pub existing: Option<&'a PreArrivalResponse>,
    pub completed: bool,
    /// Les créneaux d'arrivée du logement. `None` sans date d'arrivée : rien ne les borne.
    pub slots: Vec<Slot>,
    /// Le prénom de l'hôte, servi par la plateforme ; vide quand elle ne le donne pas.
    pub host_name: String,
    /// Le nombre de voyageurs du séjour, quand la plateforme le connaît.
    pub party_size: Option<u32>,
}

/// Fullscreen overlay form body (design `prearrivalBody` — no nested Card chrome).
///
/// When `existing` is set, fields are prefilled so the guest can edit / resubmit
/// until check-in.
pub fn build_form_surface(inputs: &FormInputs) -> Surface {
    use portaki_sdk::sdui::primitives::{
        Button, Celebration, ChoiceList, Field, FieldHint, Form, KeyValue, Text, TextArea,
        TimePicker,
    };

    let FormInputs {
        questions,
        existing,
        completed,
        slots,
        host_name,
        party_size,
    } = inputs;
    let completed = *completed;
    let existing = *existing;

    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);
    let mut form_children: Vec<Component> = Vec::new();

    if completed {
        // « Claire a bien reçu vos informations » (§2.20). Le formulaire reste dessous, rempli :
        // c'est exactement ce qu'un bouton « Modifier » ouvrirait, sans l'aller-retour — et le
        // livret ne sait pas porter une entrée à travers un panneau.
        form_children.push(
            Celebration::new()
                .emoji("🗝️")
                .title(received_title(host_name))
                .message(received_message(existing, slots))
                .into(),
        );
    } else {
        form_children.push(
            Text::new()
                .text("i18n:home.card.intro")
                .variant(TextVariant::Body)
                .into(),
        );
    }

    if questions.ask_arrival_time {
        let answered = existing.and_then(|row| row.arrival_time.as_deref());
        let field = Field::new()
            .name("arrivalTimeEstimated")
            .label("i18n:form.arrival.label")
            .required(true);
        if slots.is_empty() {
            // Sans heure d'entrée, aucune borne : le sélecteur libre reste le seul choix honnête.
            let mut picker = TimePicker::new().name("arrivalTimeEstimated");
            if let Some(value) = answered {
                picker = picker.value(value);
            }
            form_children.push(field.child(picker).into());
        } else {
            let mut choices = ChoiceList::new()
                .name("arrivalTimeEstimated")
                .layout(ChoiceListLayout::Segmented)
                .choices(
                    slots
                        .iter()
                        .map(|slot| ChoiceOption::new(&slot.value, &slot.label))
                        .collect(),
                );
            // Rien n'est coché d'avance : la question est obligatoire, et un créneau préchoisi
            // part tel quel si le voyageur ne le regarde pas.
            if let Some(index) =
                answered.and_then(|value| crate::slots::slot_of(value, hour_of(slots)))
            {
                choices = choices.value(slots[index].value.clone());
            }
            form_children.push(field.child(choices).into());
            form_children.push(
                FieldHint::new()
                    .text(crate::slots::floor_label(&slots[0].value))
                    .into(),
            );
        }
    }
    // Le transport juste après l'heure : les deux disent ce que l'hôte doit préparer.
    if questions.ask_transport {
        let mut choices = ChoiceList::new()
            .name("transport")
            .layout(ChoiceListLayout::Compact)
            .choices(
                crate::TRANSPORTS
                    .iter()
                    .map(|value| {
                        ChoiceOption::new(*value, format!("i18n:form.transport.{value}"))
                            .icon(transport_icon(value))
                    })
                    .collect(),
            );
        if let Some(value) = existing.and_then(|row| row.transport.as_deref()) {
            choices = choices.value(value.to_string());
        }
        form_children.push(
            Field::new()
                .name("transport")
                .label("i18n:form.transport.label")
                .required(true)
                .child(choices)
                .into(),
        );
    }
    if questions.ask_occasion {
        // Des choix, pas un champ libre : l'hôte qui lit « anniversaire » sait quoi faire, celui
        // qui lit « c'est spécial pour nous » ne sait pas (§2.20).
        let mut choices = ChoiceList::new()
            .name("guestOccasion")
            .layout(ChoiceListLayout::Compact)
            .choices(
                crate::OCCASIONS
                    .iter()
                    .map(|value| ChoiceOption::new(*value, format!("i18n:form.occasion.{value}")))
                    .collect(),
            );
        if let Some(value) = existing.and_then(|row| row.occasion.as_deref()) {
            choices = choices.value(value.to_string());
        }
        form_children.push(
            Field::new()
                .name("guestOccasion")
                .label("i18n:form.occasion.label")
                .child(choices)
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
        match party_size {
            // Ne pas demander ce que la réservation dit déjà (§2.20) : le nombre s'affiche, il
            // ne se saisit pas. Sa source est la réservation — la seule que le module puisse
            // nommer : la fiche de police tient sa propre liste, et un module n'y touche pas
            // (§2.21).
            Some(size) => form_children.push(
                KeyValue::new()
                    .key("i18n:form.guestCount.label")
                    .value(counted_from_booking(*size))
                    .into(),
            ),
            None => form_children.push(
                Field::new()
                    .name("guestCount")
                    .label("i18n:form.guestCount.label")
                    .child(text_input(
                        "guestCount",
                        "i18n:form.guestCount.placeholder",
                        existing.and_then(|row| row.guest_count.as_deref()),
                    ))
                    .into(),
            ),
        }
    }
    if questions.ask_special_needs {
        // Un besoin tient rarement sur une ligne : « lit bébé et allergie aux fruits à coque »
        // était coupé par un champ de saisie d'une ligne.
        let mut needs = TextArea::new()
            .name("specialNeeds")
            .rows(2)
            .placeholder("i18n:form.specialNeeds.placeholder");
        if let Some(value) = existing.and_then(|row| row.special_needs.as_deref()) {
            needs = needs.value(value);
        }
        form_children.push(
            Field::new()
                .name("specialNeeds")
                .label("i18n:form.specialNeeds.label")
                .child(needs)
                .into(),
        );
        form_children.push(FieldHint::new().text("i18n:form.specialNeeds.hint").into());
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
    form_children.push(
        Button::new()
            .label(submit_label(completed, host_name))
            .action(submit_action)
            .into(),
    );

    // Page chrome owns the title; body is the form only (no nested Card).
    Surface::new(Form::new().children(form_children)).with_id(crate::guest::form::GUEST_FORM)
}

/// Read-only summary after check-in (answers no longer editable).
pub fn build_readonly_surface(questions: &ModuleConfig, response: &PreArrivalResponse) -> Surface {
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
            // « dès 17 h » : ce que le voyageur a annoncé est un plancher, pas un rendez-vous.
            match response.arrival_time.as_deref() {
                Some(time) => crate::slots::floor_label(time),
                None => display_or_dash(None),
            },
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

    Surface::new(Stack::new().gap(8.0).children(children)).with_id(crate::guest::form::GUEST_FORM)
}

/// « Envoyer à Claire », ou « Envoyer » quand la plateforme ne donne pas de prénom.
fn submit_label(completed: bool, host_name: &str) -> String {
    let key = match (completed, host_name.is_empty()) {
        (true, true) => return "i18n:form.submitUpdate".to_string(),
        (false, true) => return "i18n:form.submit".to_string(),
        (true, false) => "form.submitUpdate.named",
        (false, false) => "form.submit.named",
    };
    t!(key, host = host_name).unwrap_or_else(|_| format!("i18n:{key}"))
}

/// « Claire a bien reçu vos informations ».
fn received_title(host_name: &str) -> String {
    if host_name.is_empty() {
        return "i18n:form.received.title".to_string();
    }
    t!("form.received.title.named", host = host_name)
        .unwrap_or_else(|_| "i18n:form.received.title".into())
}

/// Le créneau annoncé, puis la seule chose qu'il reste à savoir : c'est encore modifiable.
fn received_message(existing: Option<&PreArrivalResponse>, slots: &[Slot]) -> String {
    let announced = existing
        .and_then(|row| row.arrival_time.as_deref())
        .filter(|_| !slots.is_empty())
        .and_then(|value| crate::slots::slot_of(value, hour_of(slots)))
        .map(|index| slots[index].label.clone());
    match announced {
        Some(slot) => t!("form.received.message.slot", slot = slot)
            .unwrap_or_else(|_| "i18n:form.received.message".into()),
        None => "i18n:form.received.message".to_string(),
    }
}

/// « 3, repris de votre réservation ».
fn counted_from_booking(size: u32) -> String {
    t!("form.guestCount.fromBooking", count = size)
        .unwrap_or_else(|_| "i18n:form.guestCount.fromBooking".into())
}

/// L'heure d'entrée du logement, relue sur le premier créneau — c'est lui qui la porte.
fn hour_of(slots: &[Slot]) -> u32 {
    slots
        .first()
        .and_then(|slot| slot.value.split_once(':'))
        .and_then(|(hour, _)| hour.parse().ok())
        .unwrap_or(0)
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
        .leading(Leading::Icon(leading.into()))
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

/// L'icône d'un moyen d'arrivée — celles du dessin (§2.20).
fn transport_icon(value: &str) -> IconName {
    match value {
        "car" => IconName::Car,
        "train" => IconName::Train,
        "plane" => IconName::Send,
        _ => IconName::InfoCircle,
    }
}
