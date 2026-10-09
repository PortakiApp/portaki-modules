//! Bouton du séjour (spec Signaler §1) : « Ajouter un signalement », pour un problème constaté
//! par l'hôte ou signalé par téléphone. Le corps du formulaire seulement : la modale (titre,
//! Annuler) est au dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Select, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;
use uuid::Uuid;

#[portaki_sdk::surface(
    host,
    id = "add",
    placement = HostPlacement::StayAction,
    label_key = "catalog.host.add",
    icon = IconName::DangerTriangle
)]
pub fn render_host_add(ctx: HostContext) -> Surface {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let body: Component = match stay_id {
        None => Text::new()
            .text("i18n:host.stay.missingStay")
            .variant(TextVariant::Caption)
            .into(),
        Some(stay_id) => form(stay_id).into(),
    };
    Surface::new(Page::new().child(body)).with_id(ADD)
}

/// Toutes les catégories : l'hôte signale ce qu'il constate, pas seulement ce que le livret
/// propose au voyageur.
fn form(stay_id: Uuid) -> Form {
    let categories = crate::category::WIRE_VALUES
        .iter()
        .map(|wire| ChoiceOption::new(*wire, format!("i18n:host.category.{wire}")))
        .collect();
    Form::new()
        .child(
            Field::new()
                .name("category")
                .label("i18n:form.category.label")
                .required(true)
                .child(
                    Select::new()
                        .name("category")
                        .options(categories)
                        .value("appliance"),
                ),
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
        .child(Button::new().label("i18n:host.add.submit").action(
            crate::ids::module_id().command(crate::commands::ADD, json!({ "stayId": stay_id })),
        ))
}
