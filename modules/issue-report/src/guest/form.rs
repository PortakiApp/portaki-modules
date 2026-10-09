//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, Field, Form, Icon, ImageUpload, Stack, Text, TextArea, TextInput,
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
    // Le numéro de l'option « Tout de suite » : celui que l'hôte a donné, sinon son profil.
    let urgent = config
        .urgent_phone()
        .then(|| {
            Some(config.urgent_number.trim().to_string())
                .filter(|phone| !phone.is_empty())
                .or_else(|| {
                    ctx.host
                        .as_ref()
                        .and_then(|host| host.phone.as_deref())
                        .map(|phone| phone.trim().to_string())
                        .filter(|phone| !phone.is_empty())
                })
        })
        .flatten();
    Ok(build_form_surface(&FormInputs {
        categories: config.categories(),
        photo: config.photo_allowed(),
        urgent,
        response_time: config.response_time(),
    }))
}

/// Ce que la feuille a besoin de savoir de la configuration.
pub struct FormInputs {
    pub categories: Vec<&'static str>,
    /// « Ajouter une photo » proposé.
    pub photo: bool,
    /// Le numéro appelé par « Tout de suite », quand l'option est ouverte et qu'il y en a un.
    pub urgent: Option<String>,
    /// `within_day`, `within_hour` ou `none`.
    pub response_time: &'static str,
}

pub fn build_form_surface(inputs: &FormInputs) -> Surface {
    Surface::new(build_form(inputs)).with_id(GUEST_FORM)
}

/// Ce que la feuille annonce avant la première question (§2.17).
///
/// Le livret hisse l'en-tête d'un formulaire dans la barre du panneau : sans lui, la feuille
/// s'ouvrait sur « Catégorie » sans dire ce qu'on est en train de faire ni ce qui arrive ensuite.
fn sheet_header() -> Component {
    Component::Stack(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(12.0)
            .child(
                Icon::new()
                    .name(IconName::DangerTriangle)
                    .size(24.0)
                    .tone(Tone::Primary),
            )
            .child(
                Stack::new()
                    .gap(4.0)
                    .child(
                        Text::new()
                            .text("i18n:form.head.title")
                            .variant(TextVariant::Title),
                    )
                    .child(
                        Text::new()
                            .text("i18n:form.head.lead")
                            .variant(TextVariant::Caption)
                            .emphasis(Emphasis::Subtle),
                    ),
            ),
    )
}

fn build_form(inputs: &FormInputs) -> Form {
    let categories = inputs.categories.as_slice();
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);

    // Une seule liste, posée en une fois : `children` **remplace** là où `child` allonge, et un
    // `child` suivi d'un `children` perdait silencieusement l'en-tête.
    let mut children = vec![sheet_header()];
    children.extend(category_field(categories));
    let mut form = Form::new();
    for child in children {
        form = form.child(child);
    }
    form = form
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
        );
    if inputs.photo {
        form = form.child(
            Field::new()
                .name("photo")
                .label("i18n:form.photo.label")
                .child(ImageUpload::new().name("photo")),
        );
    }
    form = form.child(
        Button::new()
            .label("i18n:form.submit")
            .action(submit_action),
    );
    // « Réponse dans la journée » : ce que l'hôte s'engage à tenir, sous le bouton (§2.2).
    if inputs.response_time != "none" {
        form = form.child(
            Text::new()
                .text(format!("i18n:form.responseTime.{}", inputs.response_time))
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle),
        );
    }
    // L'urgence : l'hôte au téléphone quand il l'a ouverte, sinon la page des numéros d'urgence
    // (le 112 pour les urgences vitales).
    match &inputs.urgent {
        Some(phone) => form
            .child(
                Text::new()
                    .text("i18n:form.urgent.callNote")
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            )
            .child(
                Button::new()
                    .label("i18n:form.urgent.call")
                    .variant(ButtonVariant::Ghost)
                    .action(Action::external(format!(
                        "tel:{}",
                        crate::config::compact(phone)
                    ))),
            ),
        None => form
            .child(
                Text::new()
                    .text("i18n:form.urgent.note")
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            )
            .child(
                Button::new()
                    .label("i18n:form.urgent.link")
                    .variant(ButtonVariant::Ghost)
                    .action(Action::navigate(NavigateTarget::path(AIDE), None)),
            ),
    }
}

/// La page de la section Aide, où les numéros d'urgence vivent.
const AIDE: &str = "aide";

/// Le champ de catégorie — rien du tout quand l'hôte n'en offre qu'une.
///
/// Choisir dans une liste d'un seul élément n'est pas un choix : c'est une ligne de plus à lire
/// dans une feuille qu'on ouvre parce que quelque chose ne va pas. La catégorie part quand même
/// au signalement, le formulaire la porte en valeur plutôt qu'en question.
fn category_field(categories: &[&str]) -> Vec<Component> {
    match categories {
        [] => Vec::new(),
        [only] => vec![Component::TextInput(
            TextInput::new().name("category").value(*only),
        )],
        many => vec![Component::Field(
            Field::new()
                .name("category")
                .label("i18n:form.category.label")
                .required(true)
                .child(category_choice_list(many)),
        )],
    }
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
        "wifi" => option.icon(IconName::Wifi),
        "outdoor" => option.icon(IconName::Sun),
        _ => option.icon(IconName::MessageCircle),
    }
}
