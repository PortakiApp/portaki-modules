//! Shared guest SDUI body for local guide spots.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{InfoBanner, Link, ListItem, Pill, Pressable, Stack, Text};

use crate::activities::ActivitiesView;

use super::load::GuestData;

pub fn build_spots_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if !data.disclaimer.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.disclaimer.title")
                .message(data.disclaimer.clone()),
        ));
    }

    for spot in &data.spots {
        let title = spot
            .title
            .pick_with_fallback(&data.locale, &data.property_locale);
        let mut subtitle_parts = Vec::new();
        if let Some(cat) = spot.category.as_deref().filter(|c| !c.trim().is_empty()) {
            subtitle_parts.push(cat.to_string());
        }
        if let Some(dist) = spot.distance.as_deref().filter(|d| !d.trim().is_empty()) {
            subtitle_parts.push(dist.to_string());
        }
        let subtitle = subtitle_parts.join(" · ");

        let mut item = ListItem::new().title(title);
        if !subtitle.is_empty() {
            item = item.subtitle(subtitle);
        }
        if let Some(tag) = spot.tag.as_deref().filter(|t| !t.trim().is_empty()) {
            item = item.child(Pill::new().label(tag));
        }
        if enriched {
            if let Some(note) = spot.note.as_ref() {
                let text = note.pick_with_fallback(&data.locale, &data.property_locale);
                if !text.trim().is_empty() {
                    item = item.child(Text::new().text(text).variant(TextVariant::Caption));
                }
            }
            if let Some(detail) = spot.detail.as_ref() {
                let text = detail.pick_with_fallback(&data.locale, &data.property_locale);
                if !text.trim().is_empty() {
                    item = item.child(Text::new().text(text).variant(TextVariant::Body));
                }
            }
            if let Some(url) = spot.url.as_deref().map(str::trim).filter(|u| !u.is_empty()) {
                let action = Action::External {
                    url: url.to_string(),
                };
                item = item.child(
                    Link::new()
                        .label("i18n:guest.openLink")
                        .href(url)
                        .action(action),
                );
            }
            children.push(Component::ListItem(item));
        } else if let Some(url) = spot.url.as_deref().map(str::trim).filter(|u| !u.is_empty()) {
            let action = Action::External {
                url: url.to_string(),
            };
            children.push(Component::Pressable(
                Pressable::new().action(action).child(item),
            ));
        } else {
            children.push(Component::ListItem(item));
        }
    }

    if let Some(view) = data.activities.as_ref() {
        children.push(Component::Stack(
            Stack::new().gap(8.0).children(build_activities(view)),
        ));
    }

    children
}

/// Section « Activités & billets » : recherche d'abord, sélection de l'hôte ensuite,
/// mention d'affiliation dessous.
///
/// Rien que des liens — pas de script, pas d'iframe, pas une image chargée chez
/// GetYourGuide. Le livret n'appelle personne pour afficher cette section.
fn build_activities(view: &ActivitiesView) -> Vec<Component> {
    let mut children: Vec<Component> = vec![Text::new()
        .text("i18n:guest.activities.title")
        .variant(TextVariant::Title)
        .into()];

    if !view.intro.is_empty() {
        children.push(
            Text::new()
                .text(view.intro.clone())
                .variant(TextVariant::Body)
                .into(),
        );
    }

    // La destination est interpolée ici : une clé `i18n:` part telle quelle vers le shell,
    // qui ne sait pas y injecter de variable.
    let search_label = t!(
        "guest.activities.searchLabel",
        destination = &view.destination
    )
    .unwrap_or_else(|_| view.destination.clone());
    children.push(
        Link::new()
            .label(search_label)
            .href(view.search_url.clone())
            .action(Action::External {
                url: view.search_url.clone(),
            })
            .into(),
    );

    for link in &view.links {
        let label = if link.label.is_empty() {
            "i18n:guest.activities.openLink".to_string()
        } else {
            link.label.clone()
        };
        children.push(
            Link::new()
                .label(label)
                .href(link.url.clone())
                .action(Action::External {
                    url: link.url.clone(),
                })
                .into(),
        );
    }

    // Obligatoire, dans toutes les langues, sous la liste : ces liens rapportent à
    // Portaki, le voyageur doit le lire avant de cliquer et non le découvrir après.
    children.push(
        Text::new()
            .text("i18n:guest.activities.disclosure")
            .variant(TextVariant::Caption)
            .into(),
    );

    children
}
