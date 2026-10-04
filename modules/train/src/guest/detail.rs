//! La page des trains : le sens, la gare, les prochains départs (§2.17).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{
    BadgeSpec, Emphasis, Leading, LeadingVisual, SurfaceLevel, Tone, Trailing, TrailingVisual,
};
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, EmptyState, ErrorState, Field, FieldHint, ListItem, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::board::{BoardView, ALL_STATIONS};
use crate::sncf::{BoardError, Stop, Way};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageParams<'a> {
    dir: &'a str,
    dest: &'a str,
}

pub fn build_detail_page(view: &BoardView, way: Way, selected: &str) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(vec![
            Component::Text(
                Text::new()
                    .text(view.station.label.clone())
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
            direction_choice(way, selected),
            station_field(view, way, selected),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.upcoming")
                    .variant(TextVariant::Title),
            ),
            schedule(view),
            disclaimer(view),
        ]))
    .with_id(crate::guest::EXPLORE_DETAIL)
}

/// Ce que le voyageur voit quand il n'y a pas de tableau : jamais une page vide, jamais un
/// tableau inventé (§2.17).
pub fn build_error_page(error: &BoardError) -> Surface {
    let surface = match error {
        // Sans gare, le module est incomplet : c'est à l'hôte de la donner, pas au voyageur de
        // comprendre une panne.
        BoardError::NoStation => Surface::new(
            EmptyState::new()
                .title("i18n:explore.detail.noStation.title")
                .description("i18n:explore.detail.noStation.description")
                .icon(IconName::Train),
        ),
        BoardError::UnknownStation => Surface::new(
            EmptyState::new()
                .title("i18n:explore.detail.unknownStation.title")
                .description("i18n:explore.detail.unknownStation.description")
                .icon(IconName::Train),
        ),
        BoardError::MissingKey | BoardError::Unavailable => Surface::new(
            ErrorState::new()
                .title("i18n:explore.detail.unavailable.title")
                .message("i18n:explore.detail.unavailable.description"),
        ),
    };
    surface.with_id(crate::guest::EXPLORE_DETAIL)
}

/// Le sens, en contrôle segmenté : deux termes côte à côte, touchés d'un doigt.
///
/// Une barre de pastilles de filtre disait « Nice-Ville » et « Cannes » sans jamais dire si on
/// partait ou si on revenait — la question qu'on se pose d'abord sur un quai.
fn direction_choice(way: Way, selected: &str) -> Component {
    let choices: Vec<ChoiceOption> = [Way::From, Way::To]
        .into_iter()
        .map(|value| {
            ChoiceOption::new(
                value.wire(),
                format!("i18n:explore.detail.direction.{}", value.wire()),
            )
            .icon(match value {
                Way::To => IconName::Home,
                Way::From => IconName::Train,
            })
        })
        .collect();
    Component::ChoiceList(
        ChoiceList::new()
            .name("dir")
            .layout(ChoiceListLayout::Segmented)
            .value(way.wire().to_string())
            .choices(choices)
            // Le choix recharge la page : c'est le livret qui rend, le module n'a pas d'état.
            .action(navigate(way, selected))
            .emitOnChange(true),
    )
}

/// La gare, en champ de recherche : une ligne dessert plus de gares qu'une barre n'a de place, et
/// on cherche « Monaco » en le tapant plus vite qu'en le trouvant.
///
/// Les options sortent du tableau lui-même : aucune liste de gares n'est écrite dans le module, et
/// aucune n'est demandée à l'hôte.
fn station_field(view: &BoardView, way: Way, selected: &str) -> Component {
    let mut choices = vec![ChoiceOption::new(
        ALL_STATIONS,
        "i18n:explore.detail.station.all",
    )];
    choices.extend(
        view.destinations
            .iter()
            .map(|destination| ChoiceOption::new(destination, destination).icon(IconName::Train)),
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
                            .action(navigate(way, selected))
                            .emitOnChange(true),
                    ),
            )
            .child(
                FieldHint::new().text(format!("i18n:explore.detail.station.hint.{}", way.wire())),
            ),
    )
}

/// Recharger la page avec le sens et la gare courants — le livret y glisse ce qui a changé.
fn navigate(way: Way, selected: &str) -> Action {
    Action::navigate(
        NavigateTarget::path("train"),
        Some(json_value(PageParams {
            dir: way.wire(),
            dest: selected,
        })),
    )
}

/// Les prochains départs, ou l'état vide — un filtre peut ne rien laisser, et les 24 h qui suivent
/// peuvent n'avoir aucun train.
fn schedule(view: &BoardView) -> Component {
    if view.stops.is_empty() {
        return Component::EmptyState(
            EmptyState::new()
                .title("i18n:explore.detail.empty.title")
                .description("i18n:explore.detail.empty.description")
                .icon(IconName::Train),
        );
    }
    let children = view
        .stops
        .iter()
        .map(|stop| departure_row(view, stop))
        .collect();
    Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .children(children),
    )
}

/// La phrase de l'hôte si elle existe, la mention de la source sinon.
fn disclaimer(view: &BoardView) -> Component {
    let text = if view.note.is_empty() {
        "i18n:explore.detail.disclaimer".to_string()
    } else {
        view.note.clone()
    };
    Component::Text(
        Text::new()
            .text(text)
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )
}

/// Une ligne de départ : l'heure devant, le mode en pastille de fin, la fiche au bout.
pub fn departure_row(view: &BoardView, stop: &Stop) -> Component {
    let mut subtitle_parts: Vec<String> = Vec::new();
    if let Some(headsign) = stop.headsign.as_deref() {
        subtitle_parts.push(headsign.to_string());
    }
    // « Demain » devant le premier train du matin : c'est ce qui distingue une nuit sans train
    // d'un tableau vide (§2.17).
    if view.is_later_day(stop) {
        subtitle_parts.push(later_day_label());
    }

    let mut item = ListItem::new()
        .title(stop.direction.clone())
        .leading(Leading::Visual(Box::new(LeadingVisual {
            time: Some(stop.time.clone()),
            ..LeadingVisual::default()
        })));
    if !subtitle_parts.is_empty() {
        item = item.subtitle(subtitle_parts.join(" · "));
    }
    if let Some(mode) = stop.mode.as_deref() {
        item = item.trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(BadgeSpec {
                label: mode.to_string(),
                // Un horaire temps réel est une bonne nouvelle, une fiche horaire une
                // information : le ton les sépare sans qu'on lise le mot.
                tone: if stop.realtime {
                    Tone::Success
                } else {
                    Tone::Neutral
                },
                dot: false,
            }),
            ..TrailingVisual::default()
        })));
    }
    Component::ListItem(item.chevron(true).action(Action::navigate(
        NavigateTarget::path(format!("train/{}", stop.route_id())),
        None,
    )))
}

fn later_day_label() -> String {
    t!("explore.detail.laterDay").unwrap_or_else(|_| "i18n:explore.detail.laterDay".into())
}
