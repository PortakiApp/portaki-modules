//! Shared guest SDUI body for facility hours.

use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{
    BadgeSpec, DetailRow, KeyValueLayout, Leading, SurfaceLevel, Tone, Trailing, TrailingVisual,
};
use portaki_sdk::sdui::primitives::{
    Button, Card, Grid, InfoBanner, KeyValue, ListItem, Stack, Text,
};

use chrono::Datelike;

use crate::schedule::{State, WEEK};

use super::load::GuestData;

/// Le nombre de lignes que la carte d'accueil montre avant de renvoyer à la liste (§2.6).
const HOME_ROWS: usize = 3;

/// L'état d'une ligne, dit comme le §2.6 le demande : « Ouvert », « Ouvre à HH:MM », « Fermé ».
///
/// Rien n'est rendu sans horaires structurés : une ligne qui n'a qu'une phrase garde sa phrase, et
/// n'affiche pas un état qu'on aurait deviné.
fn state_badge(state: &State) -> Trailing {
    let (label, tone) = match state {
        State::AlwaysOpen | State::Open => ("i18n:guest.state.open".to_string(), Tone::Success),
        State::OpensAt(minutes) => (
            t!("guest.state.opensAt", time = format_minutes(*minutes))
                .unwrap_or_else(|_| "i18n:guest.state.opensAtPlain".to_string()),
            Tone::Neutral,
        ),
        State::Closed => ("i18n:guest.state.closed".to_string(), Tone::Neutral),
        // Hors saison : « Fermé » ferait croire à une fermeture du jour, et le voyageur
        // reviendrait demain devant la même porte.
        State::OutOfSeason => ("i18n:guest.state.outOfSeason".to_string(), Tone::Neutral),
    };
    Trailing::Visual(Box::new(TrailingVisual {
        badge: Some(BadgeSpec {
            label,
            tone,
            dot: false,
        }),
        ..TrailingVisual::default()
    }))
}

/// `HH:MM` depuis des minutes après minuit.
fn format_minutes(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Les sept jours d'une ligne, lundi en tête, le jour courant marqué (§2.6).
fn week_rows(
    row: &crate::config::FacilityRow,
    locale: &str,
    today: Option<chrono::Weekday>,
) -> Vec<DetailRow> {
    let schedule = row.schedule();
    WEEK.iter()
        .map(|day| {
            let value = if schedule.all_day {
                "i18n:guest.state.open".to_string()
            } else {
                match schedule.span_on(*day) {
                    Some(span) => format!(
                        "{} – {}",
                        format_minutes(span.opens),
                        format_minutes(span.closes)
                    ),
                    None => "i18n:guest.state.closed".to_string(),
                }
            };
            DetailRow {
                label: time::weekday_name(*day, locale).to_string(),
                value,
                current: today == Some(*day),
            }
        })
        .collect()
}

pub fn build_hours_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    // L'heure du rendu, lue une fois : deux lignes de la même carte ne doivent pas répondre à deux
    // instants différents. Sans horloge, aucune ligne n'affiche d'état — mieux vaut rien qu'un
    // « Ouvert » tiré d'une heure inventée.
    let now = time::now().ok();
    let tz = PropertyTz::parse(&data.timezone);
    let today = now.and_then(|now| {
        tz.as_ref()
            .map(|tz| tz.to_local(now).naive_local().weekday())
            .or_else(|| Some(now.naive_utc().weekday()))
    });

    if !data.general_note.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.note.title")
                .message(data.general_note.clone()),
        ));
    }

    // La carte d'accueil s'arrête à trois lignes et renvoie au reste ; la feuille montre tout.
    let shown: Vec<&crate::config::FacilityRow> = if enriched {
        data.facilities.iter().collect()
    } else {
        data.facilities.iter().take(HOME_ROWS).collect()
    };
    let hidden = data.facilities.len().saturating_sub(shown.len());

    let mut rows: Vec<(Option<String>, Component)> = Vec::new();
    for facility in shown {
        let title = facility.title.get(&data.locale);
        let lines = facility.lines(&data.locale);
        let hours = facility
            .hours
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| (!lines.is_empty()).then(|| lines.join(" · ")))
            .unwrap_or_default();

        if enriched {
            let mut item = ListItem::new().title(title);
            if let Some(icon) = facility.icon_name() {
                item = item.leading(Leading::Icon(icon.to_string()));
            }
            if !hours.is_empty() {
                item = item.subtitle(hours.clone());
            }
            // L'état en direct et la semaine dépliable, pour les lignes qui portent des heures.
            let schedule = facility.schedule();
            if let Some(state) = now.and_then(|now| schedule.state_at(now, tz.as_ref())) {
                item = item.trailing(state_badge(&state));
                item = item.details(week_rows(facility, &data.locale, today));
            }
            for line in lines {
                item = item.child(Text::new().text(line).variant(TextVariant::Caption));
            }
            let note = facility.note.get(&data.locale);
            if !note.trim().is_empty() {
                item = item.child(Text::new().text(note).variant(TextVariant::Caption));
            }
            rows.push((
                facility.group_label().map(str::to_string),
                Component::ListItem(item),
            ));
        } else {
            let schedule = facility.schedule();
            let row = match now.and_then(|now| schedule.state_at(now, tz.as_ref())) {
                /*
                 * Sur la carte aussi : l'horaire, l'état, et la semaine dépliable.
                 *
                 * <p>L'état remplaçait l'horaire, « ce qu'on lit d'un coup d'œil ». Mais
                 * « Ouvert » ne dit pas jusqu'à quand, et c'est la question suivante — il
                 * fallait ouvrir la feuille pour la poser. La maquette (§2.6) montre les trois
                 * sur la carte, et la semaine ne coûte rien tant qu'elle est repliée.
                 */
                Some(state) => {
                    let mut item = ListItem::new().title(title);
                    if let Some(icon) = facility.icon_name() {
                        item = item.leading(Leading::Icon(icon.to_string()));
                    }
                    if !hours.is_empty() {
                        item = item.subtitle(hours.clone());
                    }
                    item = item.trailing(state_badge(&state)).details(week_rows(
                        facility,
                        &data.locale,
                        today,
                    ));
                    Component::ListItem(item)
                }
                None => Component::KeyValue(KeyValue::new().key(title).value(hours)),
            };
            rows.push((None, row));
        }
    }

    // La sous-page range les lignes par groupe, une carte par groupe, dans l'ordre où l'hôte les a
    // saisies (§2.6) : « Séjour », « Équipements », « Services ». Une liste de huit lignes d'affilée
    // se lit comme un tableau d'horaires de gare. La carte d'accueil, elle, n'en montre que trois :
    // les grouper y ferait trois cartes d'une ligne.
    if enriched && rows.iter().any(|(group, _)| group.is_some()) {
        children.push(Component::Text(
            Text::new()
                .text(count_caption(data.facilities.len()))
                .variant(TextVariant::Caption),
        ));
        let mut order: Vec<Option<String>> = Vec::new();
        for (group, _) in &rows {
            if !order.contains(group) {
                order.push(group.clone());
            }
        }
        // Les lignes sans groupe en dernier : elles ferment la liste au lieu de la couper.
        order.sort_by_key(Option::is_none);
        for group in order {
            let items: Vec<Component> = rows
                .iter()
                .filter(|(row_group, _)| *row_group == group)
                .map(|(_, row)| row.clone())
                .collect();
            let mut card = Card::new().surface(SurfaceLevel::Elevated);
            if let Some(label) = &group {
                card = card.title(label.clone()).icon(IconName::Clock);
            }
            children.push(Component::Card(
                card.child(Stack::new().gap(4.0).children(items)),
            ));
        }
    } else {
        children.extend(rows.into_iter().map(|(_, row)| row));
    }

    // « Voir tous les horaires », et seulement s'il y en a d'autres à voir : le §2.6 ne veut pas de
    // bouton à trois lignes ou moins, qui promettrait une liste identique à celle qu'on lit déjà.
    if hidden > 0 {
        children.push(Component::Button(
            Button::new()
                .label("i18n:home.card.seeAll")
                .variant(ButtonVariant::Outline)
                .action(Action::open_overlay(
                    OverlayPresentation::BottomSheet,
                    crate::guest::EXPLORE_DETAIL,
                    OverlayArgs::new()
                        .icon(IconName::Clock)
                        .title("i18n:home.card.title"),
                )),
        ));
    }

    children
}

/// « 8 lignes d'horaires » au-dessus de la liste complète.
fn count_caption(count: usize) -> String {
    let key = if count == 1 {
        "guest.page.count.one"
    } else {
        "guest.page.count"
    };
    t!(key, count = count).unwrap_or_else(|_| format!("i18n:{key}"))
}

/// Les tuiles « Arrivée » et « Départ », tirées des dates du séjour (§2.6).
///
/// L'hôte ne les saisit pas : ce sont des données de séjour, et les redemander donnerait deux
/// vérités pour la même heure. Sans séjour — un aperçu, une consultation hors réservation — la
/// grille disparaît plutôt que d'afficher des tirets.
pub fn stay_tiles(data: &GuestData) -> Option<Component> {
    let tz = PropertyTz::parse(&data.timezone);
    let at = |instant: Option<portaki_sdk::prelude::DateTime<portaki_sdk::prelude::Utc>>| {
        instant.map(|instant| match tz.as_ref() {
            Some(tz) => tz.to_local(instant).format("%H:%M").to_string(),
            None => instant.format("%H:%M").to_string(),
        })
    };

    let mut tiles = Vec::new();
    if let Some(hour) = at(data.checkin_at) {
        tiles.push(stay_tile(
            "i18n:guest.stay.checkin",
            IconName::Key,
            "guest.stay.fromHour",
            hour,
        ));
    }
    if let Some(hour) = at(data.checkout_at) {
        tiles.push(stay_tile(
            "i18n:guest.stay.checkout",
            IconName::Clock,
            "guest.stay.beforeHour",
            hour,
        ));
    }
    if tiles.is_empty() {
        return None;
    }
    Some(Component::Grid(
        Grid::new()
            .minColumnWidth(120.0)
            .plain(true)
            .children(tiles),
    ))
}

/// « dès 16:00 », « avant 10:00 » — et c'est la langue qui place l'heure.
///
/// Le qualificatif ne se colle pas devant l'heure : `{hour} から` en japonais, `{hour} 前` en
/// chinois la veulent en tête. L'heure est donc passée à la clé paramétrée, et chaque bundle
/// décide de l'ordre. Sans hôte pour traduire, la tuile garde l'heure seule.
fn stay_tile(label: &str, icon: IconName, qualifier_key: &str, hour: String) -> Component {
    Component::KeyValue(
        KeyValue::new()
            .key(label)
            .value(t!(qualifier_key, hour = &hour).unwrap_or(hour))
            .layout(KeyValueLayout::Tile)
            .icon(icon)
            .mono(true),
    )
}
