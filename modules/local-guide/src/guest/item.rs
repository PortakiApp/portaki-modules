//! La fiche d'une adresse — la sous-page du §2.12.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{
    Author, Emphasis, KeyValueLayout, RichTextVariant, StackDirection, SurfaceLevel, Tone,
};
use portaki_sdk::sdui::primitives::{
    Button, Card, Grid, Image, InfoBanner, KeyValue, Map, RichText, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::SpotRow;

use super::load::GuestData;

/// La largeur d'une tuile de trajet (§2.12) : deux tiennent côte à côte sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

/// Les vitesses de trajet, en km/h : la marche d'une ville, et une voiture qui s'arrête aux feux.
///
/// ponytail: deux constantes contre un calcul d'itinéraire. Elles tiennent sur une distance à vol
/// d'oiseau ; un vrai temps de trajet demande un appel réseau par adresse, à chaque ouverture du
/// livret. D'où « environ », porté par la traduction.
pub(crate) const WALK_KMH: f64 = 5.0;
pub(crate) const DRIVE_KMH: f64 = 30.0;

pub fn build_spot_item(data: &GuestData, spot: &SpotRow) -> Surface {
    let mut children: Vec<Component> = Vec::new();

    // Les photos d'abord : on choisit un restaurant sur ce qu'on voit avant de lire ses horaires.
    if let Some(gallery) = gallery(data, spot) {
        children.push(gallery);
    }

    children.push(header(data, spot));

    // Fermé aujourd'hui, tout de suite sous le titre : la fiche sert à décider d'y aller
    // maintenant, et l'information arrive avant l'avantage et les horaires (§2.12).
    if let Some(closed) = super::body::closed_today_label(data, spot) {
        children.push(InfoBanner::new().tone(Tone::Warning).title(closed).into());
    }

    // « Réservation indispensable en août » : ce que l'hôte veut qu'on lise avant d'y aller.
    let warning = spot.warning.get(&data.locale).trim().to_string();
    if !warning.is_empty() {
        children.push(InfoBanner::new().tone(Tone::Warning).title(warning).into());
    }

    // L'avantage en bandeau de marque : c'est la raison d'être d'un bon plan, et une ligne de
    // texte parmi d'autres le faisait passer pour un détail d'horaires.
    let perk = spot.perk.get(&data.locale).trim().to_string();
    if !perk.is_empty() {
        children.push(
            InfoBanner::new()
                .tone(Tone::Primary)
                .title("i18n:guest.perk.title")
                .message(perk)
                .into(),
        );
    }

    if let Some(tiles) = travel_tiles(data, spot) {
        children.push(tiles);
    }

    if let Some(map) = spot_map(data, spot) {
        children.push(map);
    }

    if let Some(card) = practical(data, spot) {
        children.push(card);
    }

    if let Some(card) = host_tip(data, spot) {
        children.push(card);
    }

    children.extend(actions(data, spot));

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

/// Le nom, puis « Restaurant · €€€ » : ce qu'on cherche à savoir avant d'y aller.
fn header(data: &GuestData, spot: &SpotRow) -> Component {
    let mut children: Vec<Component> = vec![Text::new()
        .text(spot.title.get(&data.locale))
        .variant(TextVariant::Display)
        .into()];
    let mut parts: Vec<String> = Vec::new();
    if let Some(category) = trimmed(spot.category.as_deref()) {
        parts.push(category.to_string());
    }
    if let Some(price) = trimmed(spot.price.as_deref()) {
        parts.push(price.to_string());
    }
    if !parts.is_empty() {
        children.push(
            Text::new()
                .text(parts.join(" · "))
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    let detail = spot.detail.get(&data.locale).trim().to_string();
    if !detail.is_empty() {
        children.push(Text::new().text(detail).variant(TextVariant::Body).into());
    }
    Stack::new().gap(6.0).children(children).into()
}

/// « À pied 15 min · En voiture 4 min », depuis la distance entre le logement et l'adresse.
///
/// Rien sans les deux positions : une durée tirée d'une seule serait inventée. La tuile voiture
/// disparaît sous 400 m — on ne prend pas la voiture pour traverser la rue.
fn travel_tiles(data: &GuestData, spot: &SpotRow) -> Option<Component> {
    let (property_lat, property_lng) = data.property_coords?;
    let (lat, lng) = (spot.lat?, spot.lng?);
    let metres = haversine_m(property_lat, property_lng, lat, lng);

    let mut tiles: Vec<Component> = vec![tile(
        IconName::Users,
        "i18n:guest.tile.walk",
        minutes_label(metres, WALK_KMH),
    )];
    if metres >= 400.0 {
        tiles.push(tile(
            IconName::Car,
            "i18n:guest.tile.drive",
            minutes_label(metres, DRIVE_KMH),
        ));
    }
    Some(
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(tiles)
            .into(),
    )
}

fn tile(icon: IconName, label: &str, value: String) -> Component {
    KeyValue::new()
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .key(label)
        .value(value)
        .into()
}

/// « environ 15 min », arrondi à la minute supérieure : annoncer 14 pour 14 min 40 fait arriver
/// après la fermeture.
fn minutes_label(metres: f64, kmh: f64) -> String {
    let minutes = (metres / 1000.0 / kmh * 60.0).ceil().max(1.0) as i64;
    t!("guest.travel.minutes", count = minutes).unwrap_or_else(|_| format!("{minutes} min"))
}

/// La distance orthodromique en mètres.
pub(crate) fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_M: f64 = 6_371_000.0;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let d_phi = phi2 - phi1;
    let d_lambda = (lng2 - lng1).to_radians();
    let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);
    2.0 * EARTH_M * a.sqrt().asin()
}

/// Le plan : le logement et l'adresse, pour voir de quel côté c'est.
fn spot_map(data: &GuestData, spot: &SpotRow) -> Option<Component> {
    let (lat, lng) = (spot.lat?, spot.lng?);
    let mut markers = vec![MapMarker::new("spot", lat, lng)
        .label(spot.title.get(&data.locale))
        .kind(MapMarkerKind::Poi)];
    if let Some((property_lat, property_lng)) = data.property_coords {
        markers.push(
            MapMarker::new("property", property_lat, property_lng)
                .label(data.property_name.clone())
                .kind(MapMarkerKind::Property),
        );
    }
    Some(
        Map::new()
            .viewport(MapViewport::new(lat, lng, Some(14.0)))
            .markers(markers)
            .label("i18n:guest.map.label")
            .isStatic(true)
            .interactionMode(MapInteractionMode::None)
            .into(),
    )
}

/// La galerie : une image seule en tête, plusieurs qui défilent (§2.12).
///
/// Le même geste que la fiche d'une activité : une photo remplit la largeur, deux ou plus se
/// touchent du pouce sans quitter la page.
fn gallery(data: &GuestData, spot: &SpotRow) -> Option<Component> {
    let alt = spot.title.get(&data.locale);
    let refs = spot.photo_refs();
    match refs.as_slice() {
        [] => None,
        [only] => Some(
            Image::new()
                .url((*only).to_string())
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
                        .map(|reference| {
                            Image::new()
                                .url((*reference).to_string())
                                .alt(alt)
                                .aspectRatio("4 / 3")
                                .into()
                        })
                        .collect(),
                )
                .into(),
        ),
    }
}

/// « Infos pratiques » : horaires, ouverture, parking, téléphone. Seules les lignes que l'hôte a
/// remplies ; la carte ne sort pas si aucune ne l'est.
fn practical(data: &GuestData, spot: &SpotRow) -> Option<Component> {
    let _ = data;
    let mut rows: Vec<Component> = Vec::new();
    if let Some(hours) = trimmed(spot.hours.as_deref()) {
        rows.push(row("i18n:guest.practical.hours", hours, true));
    }
    if let Some(opening) = trimmed(spot.opening.as_deref()) {
        rows.push(row("i18n:guest.practical.opening", opening, false));
    }
    if let Some(parking) = trimmed(spot.parking.as_deref()) {
        rows.push(row("i18n:guest.practical.parking", parking, false));
    }
    if let Some(phone) = trimmed(spot.phone.as_deref()) {
        rows.push(row("i18n:guest.practical.phone", phone, true));
    }
    (!rows.is_empty()).then(|| {
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::InfoCircle)
            .title("i18n:guest.practical.title")
            .children(rows)
            .into()
    })
}

fn row(key: &str, value: &str, mono: bool) -> Component {
    let mut node = KeyValue::new().key(key).value(value.to_string());
    if mono {
        node = node.mono(true);
    }
    node.into()
}

/// « Le conseil de votre hôte » : la note qu'il a écrite, et seulement si elle existe.
///
/// Citation signée de §8 : `author` pose la ligne au bas d'un encart teinté, ce que veut un
/// conseil — là où `signature` la pose sous un filet, au bout d'un texte long comme le mot de
/// bienvenue. Le module **marque** le bloc et rien de plus : nom, rôle, initiales et photo
/// viennent du profil de l'hôte, et c'est la coquille qui les remplit. Un module qui les
/// redériverait afficherait un hôte différent du bandeau d'état, qui lit la même source.
fn host_tip(data: &GuestData, spot: &SpotRow) -> Option<Component> {
    let tip = spot.note.as_ref()?.get(&data.locale).trim().to_string();
    if tip.is_empty() {
        return None;
    }
    Some(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::Message)
            .title(tip_title(&data.host_name))
            .child(
                RichText::new()
                    .content(tip)
                    .variant(RichTextVariant::Lead)
                    .author(Author::default()),
            )
            .into(),
    )
}

/// « Le conseil de Claire », ou « Le conseil de votre hôte » quand la plateforme ne le nomme pas.
fn tip_title(host_name: &str) -> String {
    if host_name.is_empty() {
        return t!("guest.tip.title").unwrap_or_else(|_| "i18n:guest.tip.title".into());
    }
    t!("guest.tip.title.named", host = host_name.to_string())
        .unwrap_or_else(|_| "i18n:guest.tip.title".into())
}

/// L'itinéraire d'abord, puis l'appel, puis le site : l'ordre de ce qu'on fait en partant.
fn actions(data: &GuestData, spot: &SpotRow) -> Vec<Component> {
    let _ = data;
    let mut bar: Vec<Component> = Vec::new();
    if let (Some(lat), Some(lng)) = (spot.lat, spot.lng) {
        bar.push(
            Button::new()
                .label("i18n:guest.route")
                .action(Action::external(google_maps_url(lat, lng)))
                .into(),
        );
    }
    if let Some(phone) = trimmed(spot.phone.as_deref()) {
        let mut button = Button::new()
            .label("i18n:guest.call")
            .action(Action::external(tel_url(phone)));
        if !bar.is_empty() {
            button = button.variant(ButtonVariant::Outline);
        }
        bar.push(button.into());
    }
    if let Some(url) = trimmed(spot.url.as_deref()) {
        let mut button = Button::new()
            .label("i18n:guest.openLink")
            .action(Action::external(url.to_string()));
        if !bar.is_empty() {
            button = button.variant(ButtonVariant::Outline);
        }
        bar.push(button.into());
    }
    bar
}

fn google_maps_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")
}

/// `tel:` sans espace ni séparateur : un numéro écrit « +33 4 93 00 00 01 » ne se compose pas.
fn tel_url(phone: &str) -> String {
    let digits: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();
    format!("tel:{digits}")
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_dials_without_its_spaces() {
        assert_eq!(tel_url("+33 4 93 00 00 01"), "tel:+33493000001");
        assert_eq!(tel_url("04.93.00.00.01"), "tel:0493000001");
    }

    #[test]
    fn walking_rounds_up_to_the_next_minute() {
        // 1 km à 5 km/h = 12 min pile ; 1,05 km passe à 13 et non à 12. Hors livret, la
        // traduction n'est pas chargée : la valeur de secours est le nombre seul.
        assert!(minutes_label(1000.0, WALK_KMH).ends_with("12 min"));
        assert!(minutes_label(1050.0, WALK_KMH).ends_with("13 min"));
    }

    #[test]
    fn a_distance_is_the_same_both_ways() {
        let there = haversine_m(43.55, 7.01, 43.56, 7.02);
        let back = haversine_m(43.56, 7.02, 43.55, 7.01);
        assert!((there - back).abs() < 0.001);
        // Un dixième de degré de latitude fait une poignée de kilomètres, pas des centaines.
        assert!((1_300.0..1_400.0).contains(&there), "{there}");
    }
}
