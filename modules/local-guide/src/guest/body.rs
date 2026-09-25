//! Shared guest SDUI body for local guide spots.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Image, InfoBanner, Link, ListItem, Map, Pill, Pressable, Stack, Text,
};

use crate::activities::ActivitiesView;
use crate::tiqets::{format_price, format_rating, TiqetsView, HOME_PRODUCTS};

use super::load::GuestData;

pub fn build_spots_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if enriched {
        if let Some(map) = spots_map(data) {
            children.push(map);
        }
    }

    if !data.disclaimer.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.disclaimer.title")
                .message(data.disclaimer.clone()),
        ));
    }

    for spot in &data.spots {
        let title = spot.title.get(&data.locale);
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
                let text = note.get(&data.locale);
                if !text.trim().is_empty() {
                    item = item.child(Text::new().text(text).variant(TextVariant::Caption));
                }
            }
            let detail = spot.detail.get(&data.locale);
            if !detail.trim().is_empty() {
                item = item.child(Text::new().text(detail).variant(TextVariant::Body));
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

    if let Some(view) = data.tiqets.as_ref() {
        children.push(Component::Stack(
            Stack::new()
                .gap(8.0)
                .children(build_tiqets(view, enriched, &data.locale)),
        ));
    }

    children
}

/// Section Tiqets : les billets à proximité, mention de la source et de l'affiliation dessous.
///
/// Le détail montre image, crédit, accroche, prix et note ; la carte d'accueil n'en garde que
/// [`HOME_PRODUCTS`], sans image, pour rester un aperçu. Chaque lien est le `product_url` de
/// Tiqets tel quel : c'est lui qui porte le code d'affiliation.
fn build_tiqets(view: &TiqetsView, enriched: bool, locale: &str) -> Vec<Component> {
    let mut children: Vec<Component> = vec![Text::new()
        .text("i18n:guest.tiqets.title")
        .variant(TextVariant::Title)
        .into()];

    let shown = if enriched {
        view.products.len()
    } else {
        HOME_PRODUCTS
    };
    for product in view.products.iter().take(shown) {
        let action = Action::External {
            url: product.product_url.clone(),
        };
        let mut item = ListItem::new().title(product.title.clone());
        let subtitle = product_subtitle(product, locale);
        if !subtitle.is_empty() {
            item = item.subtitle(subtitle);
        }

        if !enriched {
            children.push(Component::Pressable(
                Pressable::new().action(action).child(item),
            ));
            continue;
        }

        if let Some(image) = product.image.as_ref() {
            children.push(
                Image::new()
                    .url(image.url.clone())
                    .alt(image.alt.clone().unwrap_or_else(|| product.title.clone()))
                    .aspectRatio("16 / 9")
                    .into(),
            );
        }
        if let Some(tagline) = product.tagline.as_deref() {
            item = item.child(Text::new().text(tagline).variant(TextVariant::Body));
        }
        // Tiqets exige le crédit de l'image partout où l'image est montrée.
        if let Some(credit) = product
            .image
            .as_ref()
            .and_then(|image| image.credit.as_deref())
        {
            item = item.child(Text::new().text(credit).variant(TextVariant::Caption));
        }
        item = item.child(
            Link::new()
                .label("i18n:guest.tiqets.book")
                .href(product.product_url.clone())
                .action(action),
        );
        children.push(Component::ListItem(item));
    }

    // Toujours sous la liste, dans toutes les langues : d'où viennent billets et notes, et
    // que ces liens rapportent une commission.
    children.push(
        Text::new()
            .text("i18n:guest.tiqets.attribution")
            .variant(TextVariant::Caption)
            .into(),
    );
    children.push(
        Text::new()
            .text("i18n:guest.tiqets.disclosure")
            .variant(TextVariant::Caption)
            .into(),
    );
    children
}

/// « Dès 22 € · ★ 4,6 (18 234) » — ce qui est connu, dans cet ordre.
fn product_subtitle(product: &portaki_connectors::tiqets::TiqetsProduct, locale: &str) -> String {
    let mut parts = Vec::new();
    if let (Some(price), Some(currency)) = (product.price, product.currency.as_deref()) {
        let formatted = format_price(price, currency, locale);
        parts.push(t!("guest.tiqets.priceFrom", price = &formatted).unwrap_or(formatted));
    }
    if let Some(rating) = product.rating {
        let average = format_rating(rating, locale);
        let count = product.rating_count.to_string();
        parts.push(
            t!("guest.tiqets.rating", rating = &average, count = &count)
                .unwrap_or_else(|_| format!("★ {average} ({count})")),
        );
    }
    parts.join(" · ")
}

/// Carte des bons plans dont l'hôte a posé la position, le logement en repère.
///
/// `None` tant qu'aucun spot n'est situé : les configurations écrites avant la carte n'ont
/// pas de coordonnées, et une carte vide vaut moins que pas de carte du tout.
///
/// Statique et sans interaction, comme celle du module `events` : cette surface est une
/// feuille qui défile, où une carte pannable volerait le geste de défilement au voyageur.
fn spots_map(data: &GuestData) -> Option<Component> {
    let mut markers = Vec::new();
    let mut lat_sum = 0.0;
    let mut lng_sum = 0.0;

    for spot in &data.spots {
        let Some((lat, lng)) = spot.coords() else {
            continue;
        };
        lat_sum += lat;
        lng_sum += lng;
        markers.push(
            MapMarker::new(spot.id.clone(), lat, lng)
                .label(spot.title.get(&data.locale))
                .kind(MapMarkerKind::Poi),
        );
    }

    if markers.is_empty() {
        return None;
    }
    let mut count = markers.len() as f64;

    // Le logement ferme la carte : sans lui, le voyageur lit des points sans savoir d'où
    // il part, et le centre se décale vers la grappe des bonnes adresses.
    if let Some((lat, lng)) = data.property_coords {
        lat_sum += lat;
        lng_sum += lng;
        count += 1.0;
        markers.push(
            MapMarker::new("property".to_string(), lat, lng)
                .label(data.property_name.clone())
                .kind(MapMarkerKind::Property),
        );
    }

    Some(Component::Map(
        Map::new()
            .viewport(MapViewport::new(
                lat_sum / count,
                lng_sum / count,
                Some(13.0),
            ))
            .markers(markers)
            .isStatic(true)
            .interactionMode(MapInteractionMode::None),
    ))
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
    // qui ne sait pas y injecter de variable. Sans nom de lieu lisible — un lien court
    // collé par l'hôte sur un logement sans adresse — le libellé neutre prend le relais.
    let destination_label = if view.destination.is_empty() {
        "i18n:guest.activities.browseLabel".to_string()
    } else {
        t!(
            "guest.activities.searchLabel",
            destination = &view.destination
        )
        .unwrap_or_else(|_| view.destination.clone())
    };
    children.push(
        Link::new()
            .label(destination_label)
            .href(view.destination_url.clone())
            .action(Action::External {
                url: view.destination_url.clone(),
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
