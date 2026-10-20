//! L'écran de l'hôte : la gare, les destinations, un conseil, et l'état du fournisseur d'horaires.

use portaki_sdk::bundle_text;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, InfoBanner, InlineNotice, Page, TagInput, TextArea,
    TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{Direction, ModuleConfig, MAX_DESTINATIONS};
use crate::sncf;

/// Trois cartes : Gare, Destinations proposées, Message.
///
/// ponytail: un nom tapé, pas une gare choisie dans une liste — les formulaires de module n'ont pas
/// de liste alimentée par une source. Le nom est résolu ici même : la gare trouvée s'affiche sous
/// le champ, un nom qui ne correspond à rien y porte l'erreur et bloque la publication.
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
    children.push(message_card(&config, &ctx));

    // Pas de titre de page ni de bouton : la feuille des modules porte son chrome et son
    // enregistrement.
    Ok(Surface::new(Page::new().child(Form::new().children(children))).with_id(MAIN))
}

/// §2.1 Gare : la gare de référence, la gare trouvée et le temps d'y aller, le sens par défaut.
fn station_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let resolved = config.station_name().map(sncf::station);
    let mut station = Field::new()
        .name("station")
        .label("i18n:config.station")
        .required(true)
        .child(
            TextInput::new()
                .name("station")
                .placeholder("i18n:host.station.placeholder")
                .value(config.station.clone()),
        )
        .child(FieldHint::new().text("i18n:host.station.hint"));
    if let Some(Err(sncf::BoardError::UnknownStation)) = &resolved {
        station = station.error(unknown_station(ctx));
    }
    let mut children: Vec<Component> = vec![station.into()];
    if let Some(Ok(resolved)) = resolved {
        // Plus de 30 km : un avertissement, pas un refus — la gare reste enregistrée (§9 #4).
        if crate::access::too_far(&resolved, ctx.property.coordinates) {
            children.push(
                InfoBanner::new()
                    .tone(Tone::Warning)
                    .title("i18n:host.station.far.title")
                    .message("i18n:host.station.far.message")
                    .into(),
            );
        }
        // Calculé, en lecture seule : sans `name`, le formulaire ne l'envoie pas.
        children.push(
            InlineNotice::new()
                .tone(Tone::Neutral)
                .message(resolved_line(&resolved, ctx))
                .into(),
        );
    }
    children.push(
        Field::new()
            .name("direction")
            .label("i18n:config.direction")
            .child(
                ChoiceList::new()
                    .name("direction")
                    .layout(ChoiceListLayout::Segmented)
                    .value(config.direction.as_wire())
                    .choices(vec![
                        ChoiceOption::new(Direction::From.as_wire(), "i18n:config.direction.from"),
                        ChoiceOption::new(Direction::To.as_wire(), "i18n:config.direction.to"),
                    ]),
            )
            .into(),
    );
    Card::new()
        .title("i18n:host.section.station")
        .icon(IconName::Train)
        .children(children)
        .into()
}

/// « Gare d'Antibes · 12 min à pied · 3 min en voiture » ; le nom seul quand le logement ou la
/// gare n'est pas placé.
fn resolved_line(station: &sncf::Station, ctx: &HostContext) -> String {
    match crate::access::access(station, ctx.property.coordinates) {
        Some(access) => {
            let (walk, drive) = (access.walk_min.to_string(), access.drive_min.to_string());
            let line = t!("host.access.value", walk = &walk, drive = &drive)
                .ok()
                .filter(|text| text.contains(&walk))
                .unwrap_or_else(|| format!("{walk} min à pied · {drive} min en voiture"));
            format!("{} · {line}", station.label)
        }
        None => station.label.clone(),
    }
}

/// §2.2 Destinations : les gares en tête du filtre du voyageur, 6 au plus, chacune résolue.
fn destinations_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let proposed = config.proposed_destinations();
    let mut destinations = Field::new()
        .name("destinations")
        .required(false)
        .child(
            TagInput::new()
                .name("destinations")
                .tags(config.destinations.clone()),
        )
        .child(FieldHint::new().text("i18n:host.destinations.hint"));
    if proposed.len() > MAX_DESTINATIONS {
        destinations = destinations.error(
            bundle_text!("config.destinations.max")
                .get(&ctx.locale)
                .to_string(),
        );
    } else if proposed.iter().any(|name| sncf::unknown(name)) {
        destinations = destinations.error(unknown_station(ctx));
    }
    Card::new()
        .title("i18n:config.destinations")
        .subtitle(format!("{} / {MAX_DESTINATIONS}", proposed.len()))
        .child(destinations)
        .into()
}

/// Le conseil de l'hôte, sous le tableau du voyageur.
fn message_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.section.message")
        .child(
            Field::new()
                .name("note")
                .label("i18n:config.note")
                .required(false)
                .child(
                    TextArea::new()
                        .name("note")
                        .placeholder("i18n:host.note.placeholder")
                        .value(config.note.host_value(ctx)),
                ),
        )
        .into()
}

fn unknown_station(ctx: &HostContext) -> String {
    bundle_text!("config.station.unknown")
        .get(&ctx.locale)
        .to_string()
}
