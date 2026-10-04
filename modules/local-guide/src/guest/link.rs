//! La fiche d'un lien collé par l'hôte (§2.13, `origin: hostLink`).
//!
//! Le lien a été reconnu comme une adresse de produit Viator, et enrichi : le voyageur a donc une
//! galerie, une note, un prix et une durée. Ce que la réponse de recherche ne porte pas — Inclus,
//! point de rendez-vous, langues — n'est **pas** deviné : ces champs ne vivent que sur
//! `GET /products/{code}`, dont Portaki n'a aucune réponse réelle, et une section vide inventée
//! vaut moins qu'une section absente.
//!
//! Le conseil de l'hôte reste la seule chose qu'aucun fournisseur n'écrira.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, KeyValueLayout, StackDirection, SurfaceLevel, Tone};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Card, Grid, Image, InfoBanner, KeyValue, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::activities::ActivityLink;
use crate::tiqets::{format_price, format_rating};
use crate::viator::format_duration;
use crate::viator_api::ViatorProduct;

use super::load::GuestData;

/// La largeur d'une tuile de mesure (§2.13) : deux tiennent côte à côte sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

/// Images de la galerie. Au-delà, on fait défiler un album plutôt que de choisir une activité.
const MAX_GALLERY: usize = 5;

pub fn build_link_item(data: &GuestData, link: &ActivityLink, product: &ViatorProduct) -> Surface {
    let mut children: Vec<Component> = Vec::new();

    if let Some(gallery) = gallery(product, link) {
        children.push(gallery);
    }

    children.push(header(data, link, product));

    if let Some(tiles) = measures(product, &data.locale) {
        children.push(tiles);
    }

    // L'annulation en bandeau : c'est ce qu'on cherche avant de s'engager, et une ligne parmi les
    // infos pratiques se lisait après le reste.
    if product.free_cancellation {
        children.push(
            InfoBanner::new()
                .tone(Tone::Success)
                .title("i18n:guest.viator.freeCancellation")
                .into(),
        );
    }

    if let Some(card) = host_tip(data, link) {
        children.push(card);
    }
    if let Some(card) = about(product) {
        children.push(card);
    }
    if let Some(card) = practical(product, &data.locale) {
        children.push(card);
    }

    children.push(
        Button::new()
            .label(book_label())
            .action(Action::External {
                url: product.product_url.clone(),
            })
            .into(),
    );
    // Obligatoire, dans toutes les langues : ce lien rapporte à Portaki, le voyageur doit le lire
    // avant de cliquer et non le découvrir après.
    children.push(
        Text::new()
            .text("i18n:guest.activities.disclosure")
            .variant(TextVariant::Caption)
            .into(),
    );

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_LINK)
}

/// La galerie : une image seule en tête, plusieurs qui défilent.
///
/// Les photos sont ce sur quoi on choisit une excursion, avant d'en lire le prix.
fn gallery(product: &ViatorProduct, link: &ActivityLink) -> Option<Component> {
    let alt = title(link, product);
    let urls: Vec<&String> = product.gallery.iter().take(MAX_GALLERY).collect();
    match urls.as_slice() {
        [] => None,
        [only] => Some(
            Image::new()
                .url((*only).clone())
                .alt(alt)
                .aspectRatio("4 / 3")
                .into(),
        ),
        many => Some(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .scroll(true)
                .gap(8.0)
                .itemWidth("78%")
                .children(
                    many.iter()
                        .map(|url| {
                            Image::new()
                                .url((*url).clone())
                                .alt(alt.clone())
                                .aspectRatio("4 / 3")
                                .into()
                        })
                        .collect(),
                )
                .into(),
        ),
    }
}

/// « via Viator », le nom, puis « Recommandé par Claire · ★ 4,7 (812) ».
fn header(data: &GuestData, link: &ActivityLink, product: &ViatorProduct) -> Component {
    let mut children: Vec<Component> = vec![
        Badge::new().label("i18n:guest.viator.via").into(),
        Text::new()
            .text(title(link, product))
            .variant(TextVariant::Display)
            .into(),
    ];
    let subtitle = super::body::host_link_subtitle(product, &data.host_name, &data.locale);
    if !subtitle.is_empty() {
        children.push(
            Text::new()
                .text(subtitle)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    // Un texte traduit par une machine se lit autrement : le dire évite de prendre une maladresse
    // pour une imprécision du prestataire.
    if product.machine_translated {
        children.push(
            Text::new()
                .text("i18n:guest.viator.machineTranslated")
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    Stack::new().gap(6.0).children(children).into()
}

/// Le titre de l'hôte s'il en a écrit un, celui de Viator sinon.
fn title(link: &ActivityLink, product: &ViatorProduct) -> String {
    if link.label.trim().is_empty() {
        product.title.clone()
    } else {
        link.label.clone()
    }
}

/// Les tuiles : le prix « dès » et la durée. « Dès » et non un prix fixe — c'est un partenaire, et
/// le prix dépend de la date et du nombre de personnes.
fn measures(product: &ViatorProduct, locale: &str) -> Option<Component> {
    let mut tiles: Vec<Component> = Vec::new();
    if let (Some(price), Some(currency)) = (product.price, product.currency.as_deref()) {
        tiles.push(measure(
            IconName::Ticket,
            "i18n:guest.viator.priceFromLabel",
            format_price(price, currency, locale),
        ));
    }
    if let Some(minutes) = product.duration_minutes {
        tiles.push(measure(
            IconName::ClockCircle,
            "i18n:guest.activity.duration",
            format_duration(minutes),
        ));
    }
    (!tiles.is_empty()).then(|| {
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(tiles)
            .into()
    })
}

fn measure(icon: IconName, label: &str, value: String) -> Component {
    KeyValue::new()
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .key(label)
        .value(value)
        .into()
}

/// Le texte de présentation du produit.
fn about(product: &ViatorProduct) -> Option<Component> {
    let description = product.description.as_deref()?.trim();
    (!description.is_empty()).then(|| {
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::InfoCircle)
            .title("i18n:guest.activity.about")
            .child(Text::new().text(description.to_string()))
            .into()
    })
}

/// « Infos pratiques » : ce que la recherche sait dire, et rien de plus.
///
/// Inclus, point de rendez-vous et langues n'y sont pas : ils ne vivent que sur la fiche produit,
/// dont nous n'avons aucune réponse réelle. Les deviner remplirait la carte de champs vides.
fn practical(product: &ViatorProduct, locale: &str) -> Option<Component> {
    let mut rows: Vec<Component> = Vec::new();
    if let Some(rating) = product.rating {
        rows.push(row(
            "i18n:guest.activity.rating",
            &t!(
                "guest.viator.rating",
                rating = &format_rating(rating, locale),
                count = &product.rating_count.to_string()
            )
            .unwrap_or_else(|_| format_rating(rating, locale)),
        ));
    }
    if let Some(minutes) = product.duration_minutes {
        rows.push(row(
            "i18n:guest.activity.duration",
            &format_duration(minutes),
        ));
    }
    rows.push(row(
        "i18n:guest.activity.provider",
        &t!("guest.viator.providerName").unwrap_or_else(|_| "Viator".to_string()),
    ));
    (!rows.is_empty()).then(|| {
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::InfoCircle)
            .title("i18n:guest.practical.title")
            .children(rows)
            .into()
    })
}

fn row(key: &str, value: &str) -> Component {
    KeyValue::new().key(key).value(value.to_string()).into()
}

/// « Le conseil de Claire » : la seule chose qu'aucun fournisseur n'écrira.
fn host_tip(data: &GuestData, link: &ActivityLink) -> Option<Component> {
    let tip = link.tip.trim();
    (!tip.is_empty()).then(|| {
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::Sparkles)
            .title(tip_title(&data.host_name))
            .child(Text::new().text(tip.to_string()))
            .into()
    })
}

fn tip_title(host_name: &str) -> String {
    if host_name.is_empty() {
        return "i18n:guest.activity.tip".to_string();
    }
    t!("guest.activity.tip.named", host = host_name.to_string())
        .unwrap_or_else(|_| "i18n:guest.activity.tip".into())
}

/// « Réserver sur Viator ».
fn book_label() -> String {
    t!(
        "guest.activity.bookOn",
        provider = &t!("guest.viator.providerName").unwrap_or_else(|_| "Viator".to_string())
    )
    .unwrap_or_else(|_| "i18n:guest.viator.book".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product() -> ViatorProduct {
        ViatorProduct {
            code: "273628P2".into(),
            title: "Le Petit Train d'Antibes".into(),
            image_url: Some("https://cdn.example/cover.jpg".into()),
            gallery: vec![
                "https://cdn.example/cover.jpg".into(),
                "https://cdn.example/b.jpg".into(),
            ],
            description: Some("Quarante-cinq minutes de visite.".into()),
            price: Some(13.0),
            currency: Some("EUR".into()),
            rating: Some(4.1),
            rating_count: 48,
            duration_minutes: Some(45),
            free_cancellation: true,
            machine_translated: false,
            product_url: "https://www.viator.com/tours/x?pid=P1".into(),
        }
    }

    #[test]
    fn the_host_title_wins_over_the_providers() {
        let mut link = ActivityLink {
            label: "  ".into(),
            url: "https://www.viator.com/tours/x".into(),
            tip: String::new(),
            product: None,
        };
        assert_eq!(title(&link, &product()), "Le Petit Train d'Antibes");
        link.label = "Le petit train".into();
        assert_eq!(title(&link, &product()), "Le petit train");
    }
}
