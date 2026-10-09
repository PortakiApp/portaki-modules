//! L'écran de l'hôte : la gare, sa phrase, et l'état du fournisseur d'horaires.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, InfoBanner, Page, Stack, TagInput, Text, TextArea,
    TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{Direction, ModuleConfig, MAX_DESTINATIONS};
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

    children.push(station_card(&config, &ctx));
    children.push(destinations_card(&config, &ctx));

    // Pas de titre de page ni de bouton : la feuille des modules porte son chrome et son
    // enregistrement.
    Ok(Surface::new(Page::new().child(Form::new().children(children))).with_id(MAIN))
}

/// §2.1 Gare : la gare de référence, vérifiée auprès de la source, et le temps d'y aller.
fn station_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let resolved = config.station_name().map(sncf::station);
    let mut station = Field::new()
        .name("station")
        .label("i18n:config.station")
        .required(true)
        .child(Stack::new().children(vec![
                    FieldHint::new().text("i18n:host.station.hint").into(),
                    TextInput::new()
                        .name("station")
                        .placeholder("i18n:host.station.placeholder")
                        .value(config.station.clone())
                        .into(),
                ]));
    if let Some(Err(sncf::BoardError::UnknownStation)) = &resolved {
        station = station.error(
            crate::i18n::text("config.station.unknown")
                .get(&ctx.locale)
                .to_string(),
        );
    }
    let mut children: Vec<Component> = vec![station.into()];
    let resolved = resolved.and_then(|station| station.ok());
    // Plus de 30 km : un avertissement, pas un refus — la gare reste enregistrée (§9 #4).
    if resolved
        .as_ref()
        .is_some_and(|station| crate::access::too_far(station, ctx.property.coordinates))
    {
        children.push(
            InfoBanner::new()
                .tone(Tone::Warning)
                .title("i18n:host.station.far.title")
                .message("i18n:host.station.far.message")
                .into(),
        );
    }
    if let Some(access) =
        resolved.and_then(|station| crate::access::access(&station, ctx.property.coordinates))
    {
        let (walk, drive) = (access.walk_min.to_string(), access.drive_min.to_string());
        let line = t!("host.access.value", walk = &walk, drive = &drive)
            .ok()
            .filter(|text| text.contains(&walk))
            .unwrap_or_else(|| format!("{walk} min à pied · {drive} min en voiture"));
        // Calculé, en lecture seule : sans `name`, le formulaire ne l'envoie pas.
        children.push(
            Field::new()
                .label("i18n:host.access.label")
                .child(Text::new().text(line).variant(TextVariant::Body))
                .into(),
        );
    }
    Card::new()
        .title("i18n:host.section.station")
        .icon(IconName::Train)
        .children(children)
        .into()
}

/// §2.2 Destinations : les gares en tête du filtre du voyageur, et le sens montré d'abord.
fn destinations_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut destinations = Field::new()
        .name("destinations")
        .label("i18n:config.destinations")
        .required(false)
        .child(Stack::new().children(vec![
                    FieldHint::new().text("i18n:host.destinations.hint").into(),
                    TagInput::new()
                        .name("destinations")
                        .tags(config.destinations.clone())
                        .into(),
                ]));
    if config.proposed_destinations().len() > MAX_DESTINATIONS {
        destinations = destinations.error(
            crate::i18n::text("config.destinations.max")
                .get(&ctx.locale)
                .to_string(),
        );
    }
    Card::new()
        .title("i18n:host.section.destinations")
        .children(vec![
            destinations.into(),
            Field::new()
                .name("direction")
                .label("i18n:config.direction")
                .child(
                    ChoiceList::new()
                        .name("direction")
                        .value(config.direction.as_wire())
                        .choices(vec![
                            ChoiceOption::new(
                                Direction::From.as_wire(),
                                "i18n:config.direction.from",
                            )
                            .icon(IconName::Train),
                            ChoiceOption::new(Direction::To.as_wire(), "i18n:config.direction.to")
                                .icon(IconName::Home),
                        ]),
                )
                .into(),
            Field::new()
                .name("note")
                .label("i18n:config.note")
                .required(false)
                .child(
                    TextArea::new()
                        .name("note")
                        .placeholder("i18n:host.note.placeholder")
                        .value(config.note.host_value(ctx)),
                )
                .into(),
        ])
        .into()
}
