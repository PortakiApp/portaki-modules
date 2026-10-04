//! Shared guest SDUI body for local guide spots.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{
    BadgeSpec, GeoPoint, Leading, LeadingVisual, ListItemLayout, StackDirection, Tone, Trailing,
    TrailingVisual,
};
use portaki_sdk::sdui::primitives::{Image, InfoBanner, Link, ListItem, Map, Pill, Stack, Text};

use crate::activities::ActivitiesView;
use crate::tiqets::{format_price, format_rating, TiqetsView, HOME_PRODUCTS};
use crate::viator::{self, format_duration, ViatorView};

use super::load::GuestData;

/// « Fermé aujourd'hui », quand on sait quel jour il est et que l'adresse ferme ce jour-là.
pub fn closed_today_label(data: &GuestData, spot: &crate::config::SpotRow) -> Option<String> {
    let today = data.today?;
    spot.closed_on(today).then(|| {
        t!("guest.spot.closedToday").unwrap_or_else(|_| "i18n:guest.spot.closedToday".into())
    })
}

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

    // Sur la carte d'accueil, les adresses défilent en tuiles ; dans la sous-page, elles se
    // déroulent en lignes avec leur note et leur lien. Même contenu, deux lectures (§2.12).
    let mut spots: Vec<Component> = Vec::new();
    for (index, spot) in data.spots.iter().enumerate() {
        let title = spot.title.get(&data.locale);
        let mut subtitle_parts = Vec::new();
        if let Some(cat) = spot.category.as_deref().filter(|c| !c.trim().is_empty()) {
            subtitle_parts.push(cat.to_string());
        }
        if let Some(dist) = spot.distance.as_deref().filter(|d| !d.trim().is_empty()) {
            subtitle_parts.push(dist.to_string());
        }
        // Fermé aujourd'hui, dans le sous-titre et non dans la pastille de fin : celle-ci porte
        // déjà l'avantage, et c'est le genre de chose qui se lit avant de s'y rendre (§2.12).
        if let Some(closed) = closed_today_label(data, spot) {
            subtitle_parts.push(closed);
        }
        let subtitle = subtitle_parts.join(" · ");

        let mut item = ListItem::new().title(title);
        if !subtitle.is_empty() {
            item = item.subtitle(subtitle);
        }
        // La vignette de plan, centrée sur l'adresse : la maquette la met devant chaque tuile, et
        // `LeadingVisual::map` l'attend — on sait où est le lieu dès que l'hôte l'a posé.
        if let (Some(lat), Some(lng)) = (spot.lat, spot.lng) {
            item = item.leading(Leading::Visual(Box::new(LeadingVisual {
                map: Some(GeoPoint { lat, lng }),
                ..LeadingVisual::default()
            })));
        }
        if let Some(tag) = spot.tag.as_deref().filter(|t| !t.trim().is_empty()) {
            // L'avantage va en fin de ligne, pas dans son corps : c'est ce que la maquette
            // montre, et un badge enfant se dessinait au milieu du texte.
            item = item.trailing(Trailing::Visual(Box::new(TrailingVisual {
                badge: Some(BadgeSpec {
                    label: tag.to_string(),
                    tone: Tone::Primary,
                    ..BadgeSpec::default()
                }),
                ..TrailingVisual::default()
            })));
        }
        // La fiche, pas le lien du commerçant : la maquette donne à chaque tuile `action: detail`,
        // et le site se rejoint depuis la fiche, après l'avantage, le plan et les horaires.
        item = item.chevron(true).action(Action::navigate(
            NavigateTarget::path(format!("local-guide/detail/{}", spot.route_id(index))),
            None,
        ));

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
            spots.push(Component::ListItem(item));
        } else {
            spots.push(Component::ListItem(item.layout(ListItemLayout::Tile)));
        }
    }

    if !spots.is_empty() {
        children.push(if enriched {
            Component::Stack(Stack::new().gap(8.0).children(spots))
        } else {
            Component::Stack(
                Stack::new()
                    .direction(StackDirection::Horizontal)
                    .scroll(true)
                    .gap(8.0)
                    .children(spots),
            )
        });
    }

    // Les activités de l'hôte avant celles des partenaires : c'est l'ordre du §2.13, et ce sont
    // les seules dont il se porte garant.
    let host_activities = super::activity::build_host_activities(data);
    if !host_activities.is_empty() {
        children.push(Component::Stack(
            Stack::new().gap(8.0).children(host_activities),
        ));
    }

    if let Some(view) = data.activities.as_ref() {
        children.push(Component::Stack(
            Stack::new()
                .gap(8.0)
                .children(build_activities(view, &data.host_name, &data.locale)),
        ));
    }

    if let Some(view) = data.tiqets.as_ref() {
        children.push(Component::Stack(
            Stack::new()
                .gap(8.0)
                .children(build_tiqets(view, enriched, &data.locale)),
        ));
    }

    if let Some(view) = data.viator.as_ref() {
        children.push(Component::Stack(
            Stack::new()
                .gap(8.0)
                .children(build_viator(view, enriched, &data.locale)),
        ));
    }

    children
}

/// Section Tiqets : les billets à proximité, mention de la source et de l'affiliation dessous.
///
/// Le détail montre image, crédit, accroche, prix et note ; la carte d'accueil n'en garde que
/// [`HOME_PRODUCTS`], en tuiles à vignette — la photo dans l'emplacement `leading`, pas en
/// bandeau plein format. Chaque lien est le `product_url` de Tiqets tel quel : c'est lui qui
/// porte le code d'affiliation.
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
    let mut tiles: Vec<Component> = Vec::new();
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
            tiles.push(partner_tile(
                item,
                product.image.as_ref().map(|image| image.url.clone()),
                product
                    .price
                    .zip(product.currency.as_deref())
                    .map(|(price, currency)| format_price(price, currency, locale)),
                action,
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

    children.extend(scrolling_tiles(tiles));
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

/// La tuile d'un lien que l'hôte a collé et que le catalogue a reconnu (§2.13, `origin: hostLink`).
///
/// Le sous-titre dit d'abord que la recommandation vient de l'hôte, puis la note et la durée du
/// fournisseur : « Recommandé par Claire · ★ 4,8 · 1 h 45 ». L'ordre compte — c'est le choix de
/// l'hôte qui fait ouvrir la tuile, la note qui rassure ensuite.
fn host_link_tile(
    link: &crate::activities::ActivityLink,
    product: &portaki_connectors::viator::ViatorProduct,
    host_name: &str,
    locale: &str,
    action: Action,
) -> Component {
    let title = if link.label.is_empty() {
        product.title.clone()
    } else {
        link.label.clone()
    };
    let mut item = ListItem::new()
        .title(title)
        .trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(BadgeSpec {
                label: "i18n:guest.viator.via".to_string(),
                ..BadgeSpec::default()
            }),
            ..TrailingVisual::default()
        })));
    let subtitle = host_link_subtitle(product, host_name, locale);
    if !subtitle.is_empty() {
        item = item.subtitle(subtitle);
    }
    // Le conseil de l'hôte sous la tuile : c'est la seule chose qu'aucun fournisseur n'écrira.
    if !link.tip.is_empty() {
        item = item.child(
            Text::new()
                .text(link.tip.clone())
                .variant(TextVariant::Caption),
        );
    }
    partner_tile(
        item,
        product.image_url.clone(),
        product
            .price
            .zip(product.currency.as_deref())
            .map(|(price, currency)| {
                let formatted = format_price(price, currency, locale);
                t!("guest.viator.priceFrom", price = &formatted).unwrap_or(formatted)
            }),
        action,
    )
}

/// « Recommandé par Claire · ★ 4,8 (1 037 avis) · 1 h 45 », ce qui en est connu.
fn host_link_subtitle(
    product: &portaki_connectors::viator::ViatorProduct,
    host_name: &str,
    locale: &str,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !host_name.is_empty() {
        if let Ok(line) = t!(
            "guest.activities.recommendedBy",
            host = host_name.to_string()
        ) {
            parts.push(line);
        }
    }
    if let Some(rating) = product.rating {
        let average = format_rating(rating, locale);
        let count = product.rating_count.to_string();
        parts.push(
            t!("guest.viator.rating", rating = &average, count = &count)
                .unwrap_or_else(|_| format!("★ {average} ({count})")),
        );
    }
    if let Some(minutes) = product.duration_minutes {
        parts.push(format_duration(minutes));
    }
    parts.join(" · ")
}

/// Une tuile de produit partenaire : la photo en tête, le prix en bout, la page au bout du doigt.
///
/// Ce que la maquette montre du §2.13 : sur la carte d'accueil, les activités défilent en tuiles
/// illustrées. En rangées sans image, elles se lisaient comme une liste de liens, et la photo —
/// la seule chose qui fait choisir une visite — n'apparaissait qu'une fois la feuille ouverte.
fn partner_tile(
    item: ListItem,
    image_url: Option<String>,
    price: Option<String>,
    action: Action,
) -> Component {
    let mut tile = item.layout(ListItemLayout::Tile).action(action);
    if let Some(url) = image_url {
        tile = tile.leading(Leading::Visual(Box::new(LeadingVisual {
            image: Some(url),
            ..LeadingVisual::default()
        })));
    }
    if let Some(price) = price {
        tile = tile.meta(price);
    }
    Component::ListItem(tile)
}

/// Les tuiles dans un défilement horizontal, ou rien quand il n'y en a pas.
fn scrolling_tiles(tiles: Vec<Component>) -> Option<Component> {
    (!tiles.is_empty()).then(|| {
        Component::Stack(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .scroll(true)
                .gap(8.0)
                .children(tiles),
        )
    })
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

/// Section Viator : les activités de la ville, mention de la source et de l'affiliation dessous.
///
/// Le détail montre image, prix, note, durée, annulation gratuite et traduction automatique ;
/// la carte d'accueil n'en garde que [`viator::HOME_PRODUCTS`], en tuiles à vignette. Chaque lien
/// est le `product_url` de Viator tel quel : c'est lui qui porte les paramètres d'affiliation.
fn build_viator(view: &ViatorView, enriched: bool, locale: &str) -> Vec<Component> {
    let mut children: Vec<Component> = vec![Text::new()
        .text("i18n:guest.viator.title")
        .variant(TextVariant::Title)
        .into()];

    let shown = if enriched {
        view.products.len()
    } else {
        viator::HOME_PRODUCTS
    };
    let mut tiles: Vec<Component> = Vec::new();
    for product in view.products.iter().take(shown) {
        let action = Action::External {
            url: product.product_url.clone(),
        };
        let mut item = ListItem::new().title(product.title.clone());
        let subtitle = viator_subtitle(product, locale);
        if !subtitle.is_empty() {
            item = item.subtitle(subtitle);
        }

        if !enriched {
            tiles.push(partner_tile(
                item,
                product.image_url.clone(),
                product
                    .price
                    .zip(product.currency.as_deref())
                    .map(|(price, currency)| format_price(price, currency, locale)),
                action,
            ));
            continue;
        }

        if let Some(url) = product.image_url.as_ref() {
            children.push(
                Image::new()
                    .url(url.clone())
                    .alt(product.title.clone())
                    .aspectRatio("16 / 9")
                    .into(),
            );
        }
        if product.free_cancellation {
            item = item.child(Pill::new().label("i18n:guest.viator.freeCancellation"));
        }
        // Viator demande de signaler un titre ou une description traduits par machine.
        if product.machine_translated {
            item = item.child(
                Text::new()
                    .text("i18n:guest.viator.machineTranslated")
                    .variant(TextVariant::Caption),
            );
        }
        item = item.child(
            Link::new()
                .label("i18n:guest.viator.book")
                .href(product.product_url.clone())
                .action(action),
        );
        children.push(Component::ListItem(item));
    }

    children.extend(scrolling_tiles(tiles));
    children.push(
        Text::new()
            .text("i18n:guest.viator.attribution")
            .variant(TextVariant::Caption)
            .into(),
    );
    children.push(
        Text::new()
            .text("i18n:guest.viator.disclosure")
            .variant(TextVariant::Caption)
            .into(),
    );
    children
}

/// « À partir de 25 € · ★ 4,8 (151 avis) · 2 h » — ce qui est connu, dans cet ordre.
fn viator_subtitle(product: &portaki_connectors::viator::ViatorProduct, locale: &str) -> String {
    let mut parts = Vec::new();
    if let (Some(price), Some(currency)) = (product.price, product.currency.as_deref()) {
        let formatted = format_price(price, currency, locale);
        parts.push(t!("guest.viator.priceFrom", price = &formatted).unwrap_or(formatted));
    }
    if let Some(rating) = product.rating {
        let average = format_rating(rating, locale);
        let count = product.rating_count.to_string();
        parts.push(
            t!("guest.viator.rating", rating = &average, count = &count)
                .unwrap_or_else(|_| format!("★ {average} ({count})")),
        );
    }
    if let Some(minutes) = product.duration_minutes {
        parts.push(format_duration(minutes));
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
fn build_activities(view: &ActivitiesView, host_name: &str, locale: &str) -> Vec<Component> {
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

    // Les liens reconnus défilent en tuiles illustrées ; les autres restent des liens. C'est
    // exactement la distinction du §2.13 : un lien enrichi se choisit sur sa photo et sa note,
    // un lien que le catalogue ne connaît pas n'a que son nom à offrir.
    let mut tiles: Vec<Component> = Vec::new();
    for link in &view.links {
        let label = if link.label.is_empty() {
            "i18n:guest.activities.openLink".to_string()
        } else {
            link.label.clone()
        };
        let action = Action::External {
            url: link.url.clone(),
        };
        match link.product.as_ref() {
            Some(product) => tiles.push(host_link_tile(link, product, host_name, locale, action)),
            None => children.push(
                Link::new()
                    .label(label)
                    .href(link.url.clone())
                    .action(action)
                    .into(),
            ),
        }
    }
    children.extend(scrolling_tiles(tiles));

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
