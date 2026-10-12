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

use crate::schedule::{local_time, spans_text, State, WEEK};

use super::load::GuestData;

/// Le badge d'une ligne : l'état calculé, en avertissement le jour d'une fermeture exceptionnelle
/// (§9 #5). `None` 24 h/24 : « 24 h/24 » se lit dans la ligne, sans statut (§9 #1).
fn state_badge(state: &State, locale: &str, exception_today: bool) -> Option<Trailing> {
    let (label, tone) = crate::schedule::state_label(state, locale)?;
    let tone = if exception_today { Tone::Warning } else { tone };
    Some(Trailing::Visual(Box::new(TrailingVisual {
        badge: Some(BadgeSpec {
            label,
            tone,
            dot: false,
        }),
        ..TrailingVisual::default()
    })))
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
            let value = if schedule.all_day && !schedule.closed_on(*day) {
                "i18n:guest.state.open".to_string()
            } else if schedule.all_day {
                "i18n:guest.state.closed".to_string()
            } else {
                let spans = schedule.spans_on(*day);
                if spans.is_empty() {
                    "i18n:guest.state.closed".to_string()
                } else {
                    // Deux plages se lisent comme l'hôte les écrit : « 08:30 – 12:00 · 14:00 – 18:00 ».
                    spans_text(&spans)
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
    let today_date = now.map(|now| local_time(now, tz.as_ref()).date());
    let today = today_date.map(|date| date.weekday());

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
        data.facilities.iter().take(data.card_limit).collect()
    };
    let hidden = data.facilities.len().saturating_sub(shown.len());

    let mut rows: Vec<(Option<String>, Component)> = Vec::new();
    for facility in shown {
        let title = facility.title.get(&data.locale);
        let lines = facility.lines(&data.locale);
        let schedule = facility.schedule();
        let hours = facility
            .hours
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| (!lines.is_empty()).then(|| lines.join(" · ")))
            .or_else(|| match facility.mode() {
                crate::config::MODE_ON_REQUEST => Some("i18n:guest.onRequest".to_string()),
                crate::config::MODE_ALWAYS => Some("i18n:guest.allDay".to_string()),
                _ => None,
            })
            // Les plages du jour, en mono comme l'hôte les saisit (§2.2) ; le reste de la semaine
            // se déplie.
            .or_else(|| today_date.map(|date| spans_text(&schedule.spans_at(date))))
            .filter(|text| !text.is_empty())
            .unwrap_or_default();
        // La fermeture exceptionnelle du jour : le badge passe en avertissement, le motif suit.
        let exception = today_date.and_then(|date| facility.exception_on(date));
        let reason = exception
            .map(|entry| entry.reason.get(&data.locale).trim().to_string())
            .filter(|reason| !reason.is_empty());

        if enriched {
            let mut item = ListItem::new().title(title);
            if let Some(icon) = facility.icon_name() {
                item = item.leading(Leading::Icon(icon.to_string()));
            }
            if !hours.is_empty() {
                item = item.subtitle(hours.clone());
            }
            // L'état en direct et la semaine dépliable, pour les lignes qui portent des heures.
            if let Some(state) = now.and_then(|now| schedule.state_at(now, tz.as_ref())) {
                if let Some(badge) = state_badge(&state, &data.locale, exception.is_some()) {
                    item = item.trailing(badge);
                }
                if schedule.is_structured() {
                    item = item.details(week_rows(facility, &data.locale, today));
                }
            }
            if let Some(reason) = &reason {
                item = item.child(
                    Text::new()
                        .text(reason.clone())
                        .variant(TextVariant::Caption),
                );
            }
            for line in lines {
                item = item.child(Text::new().text(line).variant(TextVariant::Caption));
            }
            let note = facility.note.get(&data.locale);
            if !note.trim().is_empty() {
                item = item.child(Text::new().text(note).variant(TextVariant::Caption));
            }
            rows.push((group_title(facility), Component::ListItem(item)));
        } else {
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
                    if let Some(badge) = state_badge(&state, &data.locale, exception.is_some()) {
                        item = item.trailing(badge);
                    }
                    if schedule.is_structured() {
                        item = item.details(week_rows(facility, &data.locale, today));
                    }
                    if let Some(reason) = &reason {
                        item = item.child(
                            Text::new()
                                .text(reason.clone())
                                .variant(TextVariant::Caption),
                        );
                    }
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

/// Le titre du groupe : traduit pour une clé de la liste fermée, tel quel pour un ancien texte
/// libre qu'aucun enregistrement n'a encore rangé.
fn group_title(facility: &crate::config::FacilityRow) -> Option<String> {
    let group = facility.group_label()?;
    Some(if crate::config::GROUPS.contains(&group) {
        format!("i18n:guest.group.{group}")
    } else {
        group.to_string()
    })
}
