//! L'écran de l'hôte : la gare, sa phrase, et l'état du fournisseur d'horaires.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, InfoBanner, Page, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::sncf;

/// Un seul champ obligatoire : le nom de la gare.
///
/// ponytail: un nom tapé, pas une gare choisie dans une liste. Un sélecteur demanderait un appel
/// au fournisseur depuis cet écran, et un écran de réglages qui dépend du réseau ne s'ouvre plus
/// quand le réseau manque. Le module résout le nom au premier rendu du livret et garde la
/// correspondance ; un nom qui ne correspond à rien se voit alors côté voyageur.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Train
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    let mut children: Vec<Component> = Vec::new();

    // La clé éditeur manque : l'hôte n'y peut rien, mais il doit savoir pourquoi son tableau est
    // vide plutôt que de croire sa gare mal écrite.
    if sncf::missing_key() {
        children.push(
            InfoBanner::new()
                .tone(Tone::Warning)
                .title("i18n:host.key.missing.title")
                .message("i18n:host.key.missing.message")
                .into(),
        );
    }

    children.push(
        Card::new()
            .title("i18n:host.section.station")
            .subtitle("i18n:host.section.station.help")
            .icon(IconName::Train)
            .children(vec![
                Field::new()
                    .name("station")
                    .label("i18n:config.station")
                    .required(true)
                    .child(
                        TextInput::new()
                            .name("station")
                            .placeholder("i18n:host.station.placeholder")
                            .value(config.station.clone()),
                    )
                    .into(),
                FieldHint::new().text("i18n:host.station.hint").into(),
                Field::new()
                    .name("note")
                    .label("i18n:config.note")
                    .child(
                        TextArea::new()
                            .name("note")
                            .placeholder("i18n:host.note.placeholder")
                            .value(config.note.host_value(&ctx)),
                    )
                    .into(),
                FieldHint::new().text("i18n:host.note.hint").into(),
            ])
            .into(),
    );

    children.push(
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    // Pas de titre de page ni de bouton : la feuille des modules porte son chrome et son
    // enregistrement.
    Ok(Surface::new(Page::new().child(Form::new().children(children))).with_id(MAIN))
}
