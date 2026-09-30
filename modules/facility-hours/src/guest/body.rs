//! Shared guest SDUI body for facility hours.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, InfoBanner, KeyValue, ListItem, Text};

use super::load::GuestData;

/// Le nombre de lignes que la carte d'accueil montre avant de renvoyer à la liste (§2.6).
const HOME_ROWS: usize = 3;

pub fn build_hours_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

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
            for line in lines {
                item = item.child(Text::new().text(line).variant(TextVariant::Caption));
            }
            let note = facility.note.get(&data.locale);
            if !note.trim().is_empty() {
                item = item.child(Text::new().text(note).variant(TextVariant::Caption));
            }
            children.push(Component::ListItem(item));
        } else {
            children.push(Component::KeyValue(KeyValue::new().key(title).value(hours)));
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
