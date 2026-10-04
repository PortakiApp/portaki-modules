//! La page des trains : le sens, la gare, les prochains départs (§2.15).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{
    BadgeSpec, Emphasis, Leading, LeadingVisual, SurfaceLevel, Tone, Trailing, TrailingVisual,
};
use portaki_sdk::sdui::primitives::{Card, ChoiceList, Field, FieldHint, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{
    departure_id, schedule_for, station_caption, Departure, DESTINATIONS, DIRECTIONS,
};

/// La valeur que le combobox renvoie pour « toutes les gares ».
pub const ALL_STATIONS: &str = "all";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageParams<'a> {
    dir: &'a str,
    dest: &'a str,
}

pub fn build_detail_page(_ctx: &GuestContext, direction: &str, selected: &str) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(vec![
            Component::Text(
                Text::new()
                    .text(station_caption())
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
            direction_choice(direction, selected),
            station_field(direction, selected),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.upcoming")
                    .variant(TextVariant::Title),
            ),
            Component::Card(schedule_card(selected)),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.disclaimer")
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
        ]))
    .with_id(crate::guest::EXPLORE_DETAIL)
}

/// Le sens, en contrôle segmenté : deux termes côte à côte, touchés d'un doigt.
///
/// Une barre de pastilles de filtre disait « Nice-Ville » et « Cannes » sans jamais dire si on
/// partait ou si on revenait — la question qu'on se pose d'abord sur un quai.
fn direction_choice(direction: &str, selected: &str) -> Component {
    let choices: Vec<ChoiceOption> = DIRECTIONS
        .iter()
        .map(|value| {
            ChoiceOption::new(*value, format!("i18n:explore.detail.direction.{value}")).icon(
                if *value == "to" {
                    IconName::Home
                } else {
                    IconName::Train
                },
            )
        })
        .collect();
    Component::ChoiceList(
        ChoiceList::new()
            .name("dir")
            .layout(ChoiceListLayout::Segmented)
            .value(direction.to_string())
            .choices(choices)
            // Le choix recharge la page : c'est le livret qui rend, le module n'a pas d'état.
            .action(navigate(direction, selected))
            .emitOnChange(true),
    )
}

/// La gare, en champ de recherche : une région a plus de gares qu'une barre n'a de place, et on
/// cherche « Monaco » en le tapant plus vite qu'en le trouvant.
fn station_field(direction: &str, selected: &str) -> Component {
    let mut choices = vec![ChoiceOption::new(
        ALL_STATIONS,
        "i18n:explore.detail.station.all",
    )];
    choices.extend(
        DESTINATIONS
            .iter()
            .map(|destination| ChoiceOption::new(*destination, *destination).icon(IconName::Train)),
    );
    // `Field` ne porte pas d'indice dans le contrat SDUI : il vient en `FieldHint`, juste
    // dessous, comme les autres modules le font.
    Component::Stack(
        Stack::new()
            .gap(4.0)
            .child(
                Field::new()
                    .name("dest")
                    .label("i18n:explore.detail.station.label")
                    .child(
                        ChoiceList::new()
                            .name("dest")
                            .layout(ChoiceListLayout::Combobox)
                            .value(selected.to_string())
                            .choices(choices)
                            .action(navigate(direction, selected))
                            .emitOnChange(true),
                    ),
            )
            .child(FieldHint::new().text(format!("i18n:explore.detail.station.hint.{direction}"))),
    )
}

/// Recharger la page avec le sens et la gare courants — le livret y glisse ce qui a changé.
fn navigate(direction: &str, selected: &str) -> Action {
    Action::navigate(
        NavigateTarget::path("train"),
        Some(json_value(PageParams {
            dir: direction,
            dest: selected,
        })),
    )
}

/// Les prochains départs : l'heure devant, la correspondance en fin de ligne, la fiche au bout.
fn schedule_card(selected: &str) -> Card {
    let children = schedule_for(selected)
        .into_iter()
        .map(|departure| departure_row(selected, departure))
        .collect();
    Card::new()
        .surface(SurfaceLevel::Elevated)
        .children(children)
}

/// Une ligne de départ.
///
/// `TimedEntry` ne portait ni action ni emplacement de fin : un train ne s'ouvrait pas, et
/// « direct » ou « chgt Nice » se perdait au milieu du sous-titre, là où la maquette en fait une
/// pastille de fin de ligne.
pub fn departure_row(destination: &str, departure: Departure) -> Component {
    Component::ListItem(
        ListItem::new()
            .title(destination.to_string())
            .subtitle(departure.platform.to_string())
            .leading(Leading::Visual(Box::new(LeadingVisual {
                time: Some(departure.time.to_string()),
                ..LeadingVisual::default()
            })))
            .trailing(Trailing::Visual(Box::new(TrailingVisual {
                badge: Some(BadgeSpec {
                    label: departure.note.to_string(),
                    // Un direct est une bonne nouvelle, une correspondance une information : le
                    // ton les sépare sans qu'on lise le mot.
                    tone: if departure.note == "direct" {
                        Tone::Success
                    } else {
                        Tone::Neutral
                    },
                    dot: false,
                }),
                ..TrailingVisual::default()
            })))
            .chevron(true)
            .action(Action::navigate(
                NavigateTarget::path(format!(
                    "train/{}",
                    departure_id(destination, departure.time)
                )),
                None,
            )),
    )
}
