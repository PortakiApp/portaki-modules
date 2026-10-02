//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, ChoiceList, Field, Form, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;

/// Bottom-sheet lost/found form (inputs live here — not on the home card).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "lost-found/form",
    label_key = "nav.lost-found"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    let config = crate::config::ModuleConfig::load(&ctx)?;
    Ok(build_form_surface(config.offers_shipping()))
}

pub fn build_form_surface(ask_address: bool) -> Surface {
    Surface::new(build_form(ask_address)).with_id(GUEST_FORM)
}

fn build_form(ask_address: bool) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);

    // Les enfants sont assemblés puis posés d'un coup : `children` remplace la liste, il ne
    // l'allonge pas, et l'appeler au milieu d'une chaîne de `child` efface ce qui précède.
    let mut children: Vec<Component> = vec![
        Field::new()
            .name("kind")
            .label("i18n:form.kind.label")
            .required(true)
            .child(kind_choice_list())
            .into(),
        Field::new()
            .name("itemDescription")
            .label("i18n:form.itemDescription.label")
            .required(true)
            .child(
                TextInput::new()
                    .name("itemDescription")
                    .placeholder("i18n:form.itemDescription.placeholder"),
            )
            .into(),
        Field::new()
            .name("contactHint")
            .label("i18n:form.contactHint.label")
            .child(
                TextInput::new()
                    .name("contactHint")
                    .placeholder("i18n:form.contactHint.placeholder"),
            )
            .into(),
    ];
    children.extend(address_field(ask_address));
    children.push(
        Field::new()
            .name("details")
            .label("i18n:form.details.label")
            .child(
                TextArea::new()
                    .name("details")
                    .placeholder("i18n:form.details.placeholder"),
            )
            .into(),
    );
    children.push(
        Button::new()
            .label("i18n:form.submit")
            .action(submit_action)
            .into(),
    );

    Form::new().children(children)
}

/// L'adresse de renvoi, seulement quand l'hôte propose le renvoi (§10).
///
/// La demander sans proposer le renvoi ferait écrire son adresse à un voyageur pour rien — et une
/// adresse postale est une donnée qu'on ne collecte pas « au cas où ».
fn address_field(ask_address: bool) -> Vec<Component> {
    if !ask_address {
        return Vec::new();
    }
    vec![Field::new()
        .name("returnAddress")
        .label("i18n:form.returnAddress.label")
        .child(
            TextArea::new()
                .name("returnAddress")
                .placeholder("i18n:form.returnAddress.placeholder"),
        )
        .into()]
}

fn kind_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("kind")
        .layout(ChoiceListLayout::Compact)
        .choices(vec![
            ChoiceOption::new("lost", "i18n:form.kind.lost").icon(IconName::SearchX),
            ChoiceOption::new("found", "i18n:form.kind.found").icon(IconName::PackageSearch),
        ])
}
