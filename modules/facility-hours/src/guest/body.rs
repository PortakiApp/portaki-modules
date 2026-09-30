//! Shared guest SDUI body for facility hours.

use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{BadgeSpec, DetailRow, Tone, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Button, InfoBanner, KeyValue, ListItem, Text};

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
            children.push(Component::ListItem(item));
        } else {
            let schedule = facility.schedule();
            match now.and_then(|now| schedule.state_at(now, tz.as_ref())) {
                // Sur la carte, l'état remplace l'horaire : c'est ce qu'on lit d'un coup d'œil.
                Some(state) => children.push(Component::ListItem(
                    ListItem::new().title(title).trailing(state_badge(&state)),
                )),
                None => children.push(Component::KeyValue(KeyValue::new().key(title).value(hours))),
            }
        }
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
