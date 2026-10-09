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
                    .text(station_line(view))
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
            direction_choice(view, way, selected),
            station_field(view, way, selected),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.upcoming")
                    .variant(TextVariant::Title),
            ),
            schedule(view, selected != ALL_STATIONS && !selected.is_empty()),
            disclaimer(view),
        ]))
    .with_id(crate::guest::EXPLORE_DETAIL)
}

/// « Antibes · Gare à 12 min à pied » : à pied jusqu'à 25 min, en voiture au-delà (spec §2.1).
fn station_line(view: &BoardView) -> String {
    let label = view.station.label.clone();
    let Some(access) = view.access else {
        return label;
    };
    let (key, minutes) = if access.walk_min <= crate::access::WALK_LIMIT_MIN {
        ("explore.detail.access.walk", access.walk_min)
    } else {
        ("explore.detail.access.drive", access.drive_min)
    };
    let minutes = minutes.to_string();
    match t!(key, min = &minutes) {
        Ok(text) if text.contains(&minutes) => format!("{label} · {text}"),
        _ => label,
    }
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
        // Le séjour n'a pas commencé : un tableau des 24 h qui viennent ne dit rien d'utile
        // trois semaines avant l'arrivée (§0.7).
        BoardError::OutsideStay => Surface::new(
            EmptyState::new()
                .title("i18n:explore.detail.outsideStay.title")
                .description("i18n:explore.detail.outsideStay.description")
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
fn direction_choice(view: &BoardView, way: Way, selected: &str) -> Component {
    let choices: Vec<ChoiceOption> = [Way::From, Way::To]
        .into_iter()
        .map(|value| {
            ChoiceOption::new(
                value.wire(),
                // « Depuis Antibes », pas « Depuis la gare » : c'est le nom qu'on cherche sur un
                // panneau, et il doit être interpolé ici — le livret ne sait pas le faire.
                with_station(
                    &format!("explore.detail.direction.{}", value.wire()),
                    &view.station.label,
                ),
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
                    // Gare d'arrivée quand on part, gare de départ quand on rentre : c'est la
                    // gare du logement qui tient l'autre bout, toujours.
                    .label(format!("i18n:explore.detail.station.label.{}", way.wire()))
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
            .child(FieldHint::new().text(with_station(
                &format!("explore.detail.station.hint.{}", way.wire()),
                &view.station.label,
            ))),
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
fn schedule(view: &BoardView, filtered: bool) -> Component {
    if view.stops.is_empty() {
        // « Rien ne part de cette gare » est faux quand c'est le filtre qui a tout retiré : des
        // trains partent, mais pas vers la gare demandée.
        let key = if filtered { "filtered" } else { "empty" };
        return Component::EmptyState(
            EmptyState::new()
                .title(format!("i18n:explore.detail.{key}.title"))
                .description(format!("i18n:explore.detail.{key}.description"))
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

/// La phrase de l'hôte si elle existe, la fraîcheur du tableau sinon.
///
/// « Mis à jour il y a 2 min » dit ce qu'une mention de source ne dit pas : si ce qu'on lit vaut
/// encore. Un tableau d'affichage se périme en minutes.
fn disclaimer(view: &BoardView) -> Component {
    let text = if view.note.is_empty() {
        freshness(view.read_min_ago)
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

/// « Horaires temps réel · mis à jour il y a N min », ou « à l'instant » quand N vaut zéro.
fn freshness(minutes: i64) -> String {
    if minutes <= 0 {
        return t!("explore.detail.updated.now")
            .unwrap_or_else(|_| "i18n:explore.detail.updated.now".into());
    }
    t!("explore.detail.updated.ago", minutes = minutes)
        .unwrap_or_else(|_| "i18n:explore.detail.updated.ago".into())
}

/// Une clé qui porte le nom de la gare, résolue ici : le livret ne sait pas interpoler.
fn with_station(key: &str, station: &str) -> String {
    t!(key, station = station).unwrap_or_else(|_| format!("i18n:{key}"))
}

/// Une ligne de départ : l'heure devant, l'état en pastille de fin, la fiche au bout.
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
    if let Some(badge) = status_badge(stop) {
        item = item.trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(badge),
            ..TrailingVisual::default()
        })));
    }
    Component::ListItem(item.chevron(true).action(Action::navigate(
        NavigateTarget::path(format!("train/{}", stop.route_id())),
        None,
    )))
}

/// L'état du train en bout de ligne : à l'heure, en retard, ou le mode faute de temps réel.
///
/// La place de fin de rangée n'en tient qu'un, et c'est l'état qu'on cherche sur un quai — pas
/// « TER », qui est déjà dans la fiche. Sans temps réel, il n'y a pas d'état à annoncer : le mode
/// reprend la place plutôt que de laisser croire à un « à l'heure » vérifié.
pub fn status_badge(stop: &Stop) -> Option<BadgeSpec> {
    match (stop.delay_min, stop.realtime) {
        (Some(late), _) => Some(BadgeSpec {
            label: delay_label(late),
            tone: Tone::Warning,
            dot: false,
        }),
        (None, true) => Some(BadgeSpec {
            label: t!("explore.detail.status.onTime")
                .unwrap_or_else(|_| "i18n:explore.detail.status.onTime".into()),
            tone: Tone::Success,
            dot: false,
        }),
        (None, false) => stop.mode.as_deref().map(|mode| BadgeSpec {
            label: mode.to_string(),
            tone: Tone::Neutral,
            dot: false,
        }),
    }
}

/// « +5 min ».
pub fn delay_label(minutes: i64) -> String {
    t!("explore.detail.status.delayed", minutes = minutes)
        .unwrap_or_else(|_| "i18n:explore.detail.status.delayed".into())
}

fn later_day_label() -> String {
    t!("explore.detail.laterDay").unwrap_or_else(|_| "i18n:explore.detail.laterDay".into())
}
