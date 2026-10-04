//! La fiche d'un événement — la sous-page du §2.11.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, KeyValueLayout, Leading, SurfaceLevel};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Card, Grid, Image, KeyValue, ListItem, Map, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::EventRow;
use crate::time_format::format_starts_at_display;

use super::load::GuestData;

/// La largeur d'une tuile de mesure (§2.11) : trois tiennent en une rangée sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

pub fn build_event_item(data: &GuestData, event: &EventRow) -> Surface {
    let mut children: Vec<Component> = Vec::new();

    // La photo d'abord, quand l'hôte en a déposé une : c'est elle qui dit à quoi ressemble la
    // soirée. En référence `portaki-file:`, que la plateforme échange contre une URL signée.
    if let Some(reference) = event.photo_ref() {
        children.push(
            Image::new()
                .url(reference.to_string())
                .alt(event.title.get(&data.locale))
                .aspectRatio("16 / 9")
                .into(),
        );
    }

    children.push(header(data, event));

    let description = event
        .note
        .as_ref()
        .map(|note| note.get(&data.locale).trim().to_string())
        .unwrap_or_default();
    if !description.is_empty() {
        children.push(
            Text::new()
                .text(description)
                .variant(TextVariant::Body)
                .into(),
        );
    }

    if let Some(tiles) = measures(data, event) {
        children.push(tiles);
    }

    if let Some(map) = event_map(data, event) {
        children.push(map);
    }

    if let Some(card) = access(data, event) {
        children.push(card);
    }
    if let Some(card) = good_to_know(data, event) {
        children.push(card);
    }

    children.extend(actions(data, event));

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

/// Quand, quoi, où — dans cet ordre : on ouvre une fiche d'événement pour l'heure.
fn header(data: &GuestData, event: &EventRow) -> Component {
    let mut children: Vec<Component> = Vec::new();
    let when = format_starts_at_display(&event.starts_at);
    if !when.trim().is_empty() {
        children.push(Badge::new().label(when).into());
    }
    children.push(
        Text::new()
            .text(event.title.get(&data.locale))
            .variant(TextVariant::Display)
            .into(),
    );
    let place = place_line(data, event);
    if !place.is_empty() {
        children.push(
            Text::new()
                .text(place)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    Stack::new().gap(6.0).children(children).into()
}

/// « Port Vauban », ou l'adresse quand l'hôte n'a nommé aucun lieu.
fn place_line(data: &GuestData, event: &EventRow) -> String {
    let place = event.place.get(&data.locale).trim().to_string();
    if !place.is_empty() {
        return place;
    }
    event
        .address
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

/// Les tuiles : la durée, le prix, et la marche depuis le logement. Celles qu'on ne sait pas
/// calculer ne s'affichent pas — une durée inventée serait fausse à l'heure où elle compte.
fn measures(data: &GuestData, event: &EventRow) -> Option<Component> {
    let mut tiles: Vec<Component> = Vec::new();
    if let Some(minutes) = event.duration_minutes() {
        tiles.push(tile(
            IconName::Clock,
            "i18n:guest.tile.duration",
            format_duration(minutes),
        ));
    }
    if let Some(price) = event
        .price
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        tiles.push(tile(
            IconName::Star,
            "i18n:guest.tile.price",
            price.to_string(),
        ));
    }
    if let Some(metres) = walking_metres(data, event) {
        tiles.push(tile(
            IconName::Users,
            "i18n:guest.tile.walk",
            format_walking(metres),
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

fn tile(icon: IconName, label: &str, value: String) -> Component {
    KeyValue::new()
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .key(label)
        .value(value)
        .into()
}

/// « 1 h 30 », « 20 min ».
fn format_duration(minutes: i64) -> String {
    if minutes < 60 {
        return t!("guest.duration.minutes", count = minutes)
            .unwrap_or_else(|_| format!("{minutes} min"));
    }
    let hours = minutes / 60;
    let rest = minutes % 60;
    if rest == 0 {
        t!("guest.duration.hours", count = hours).unwrap_or_else(|_| format!("{hours} h"))
    } else {
        t!("guest.duration.hoursMinutes", hours = hours, minutes = rest)
            .unwrap_or_else(|_| format!("{hours} h {rest:02}"))
    }
}

/// « 15 min » à pied, comptés à 5 km/h — la vitesse d'une marche de ville.
///
/// Arrondi à la minute supérieure : annoncer 14 min pour un trajet de 14 min 40 ferait rater le
/// début. Pas d'itinéraire calculé : la distance est à vol d'oiseau, et le dire serait mentir sur
/// la précision — d'où « environ », porté par la traduction.
fn format_walking(metres: f64) -> String {
    let minutes = (metres / 1000.0 * 12.0).ceil().max(1.0) as i64;
    t!("guest.walk.minutes", count = minutes).unwrap_or_else(|_| format!("{minutes} min"))
}

/// La distance à vol d'oiseau entre le logement et l'événement, quand les deux sont posés.
fn walking_metres(data: &GuestData, event: &EventRow) -> Option<f64> {
    let (property_lat, property_lng) = data.property_coords?;
    let (lat, lng) = (event.lat?, event.lng?);
    Some(haversine_m(property_lat, property_lng, lat, lng))
}

/// La distance orthodromique en mètres.
fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_M: f64 = 6_371_000.0;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let d_phi = phi2 - phi1;
    let d_lambda = (lng2 - lng1).to_radians();
    let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);
    2.0 * EARTH_M * a.sqrt().asin()
}

/// Le plan : le logement et le lieu de l'événement.
fn event_map(data: &GuestData, event: &EventRow) -> Option<Component> {
    let (lat, lng) = (event.lat?, event.lng?);
    let mut markers = vec![MapMarker::new("event", lat, lng)
        .label(event.title.get(&data.locale))
        .kind(MapMarkerKind::Poi)];
    if let Some((property_lat, property_lng)) = data.property_coords {
        markers.push(
            MapMarker::new("property", property_lat, property_lng)
                .label("i18n:guest.map.property")
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

/// « Accès » : comment on y entre, et si on y entre (§2.11).
///
/// Sa propre carte, et avant « Bon à savoir » : c'est ce qu'on lit en partant, pas un conseil
/// qu'on lit s'il reste du temps.
fn access(data: &GuestData, event: &EventRow) -> Option<Component> {
    lines_card(
        event.access.get(&data.locale),
        "i18n:guest.access",
        // Pas d'icône d'accessibilité au vocabulaire : la boussole est ce qui s'en approche le
        // plus honnêtement — « comment on y arrive » — plutôt qu'une clé ou un bonhomme détourné.
        IconName::Compass,
    )
}

/// « Bon à savoir » : une ligne par conseil de l'hôte. Rien quand il n'en a écrit aucun — une
/// carte vide promettrait un conseil qui n'existe pas.
fn good_to_know(data: &GuestData, event: &EventRow) -> Option<Component> {
    lines_card(
        event.tips.get(&data.locale),
        "i18n:guest.goodToKnow",
        IconName::InfoCircle,
    )
}

/// Une carte dont chaque ligne du texte devient une rangée. `None` quand il n'y a pas de ligne.
fn lines_card(raw: &str, title: &str, icon: IconName) -> Option<Component> {
    let rows: Vec<Component> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            ListItem::new()
                .title(line.to_string())
                .leading(Leading::Icon(icon.to_string()))
                .into()
        })
        .collect();
    (!rows.is_empty()).then(|| {
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(icon)
            .title(title.to_string())
            .children(rows)
            .into()
    })
}

/// L'itinéraire d'abord — c'est ce qu'on ouvre en partant —, puis l'agenda, puis la page tierce.
fn actions(data: &GuestData, event: &EventRow) -> Vec<Component> {
    let mut bar: Vec<Component> = Vec::new();
    if let (Some(lat), Some(lng)) = (event.lat, event.lng) {
        bar.push(
            Button::new()
                .label("i18n:guest.route")
                .action(Action::external(google_maps_url(lat, lng)))
                .into(),
        );
    }
    if let Some(url) = calendar_url(data, event) {
        let mut button = Button::new()
            .label("i18n:guest.addToCalendar")
            .action(Action::external(url));
        if !bar.is_empty() {
            button = button.variant(ButtonVariant::Outline);
        }
        bar.push(button.into());
    }
    if let Some(url) = event
        .url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
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

/// Le lien « Ajouter à mon agenda », au format du modèle Google Agenda.
///
/// Un lien et non un fichier `.ics` : le module ne sert pas de fichiers, et un modèle d'URL ouvre
/// l'agenda du téléphone sans rien télécharger. Sans date lisible, pas de bouton : un événement
/// sans heure n'a rien à mettre dans un agenda.
fn calendar_url(data: &GuestData, event: &EventRow) -> Option<String> {
    let starts = crate::time_format::parse_starts_at(&event.starts_at)?;
    let ends = event
        .ends_at
        .as_deref()
        .and_then(crate::time_format::parse_starts_at)
        .filter(|ends| *ends > starts)
        .unwrap_or_else(|| starts + chrono::Duration::hours(1));
    let stamp = |at: portaki_sdk::prelude::DateTime<portaki_sdk::prelude::Utc>| {
        at.format("%Y%m%dT%H%M%SZ").to_string()
    };
    let mut url = format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&dates={}/{}",
        stamp(starts),
        stamp(ends)
    );
    url.push_str("&text=");
    url.push_str(&urlencode(event.title.get(&data.locale)));
    let place = place_line(data, event);
    if !place.is_empty() {
        url.push_str("&location=");
        url.push_str(&urlencode(&place));
    }
    Some(url)
}

/// Le pourcentage des caractères qu'une query string ne laisse pas passer.
///
/// Écrit ici plutôt que tiré d'une dépendance : trois lignes contre une caisse, et le jeu de
/// caractères sûrs d'une query string ne bougera pas.
fn urlencode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}
