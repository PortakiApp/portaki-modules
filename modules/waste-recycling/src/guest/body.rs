//! Shared guest SDUI body for waste bins.

use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Leading, LeadingVisual};
use portaki_sdk::sdui::primitives::{Highlight, InfoBanner, ListItem, Text};

use crate::collection::{next_collection, Departure, NextCollection};
use crate::config::{bin_swatch, DropoffRow};

use super::load::GuestData;

/// Le bandeau de collecte seul (§2.7) : c'est la seule chose de cette carte qui change d'un jour
/// à l'autre, et donc la seule qu'on relit.
///
/// Rendu à part pour que la feuille puisse glisser « Dans le logement » et « Le local poubelles »
/// entre lui et les bacs, comme la maquette les ordonne.
pub fn build_collection_banner(data: &GuestData) -> Vec<Component> {
    collection_banner(data, next_up(data))
}

/// Les bacs, une rangée par bac : le nom en titre, ce qui y va en sous-titre, et la pastille de
/// couleur en tête quand l'hôte a choisi une teinte que le livret connaît.
///
/// Une rangée et non un `ColorDotItem` : la maquette donne `ListItem leading:{swatch}`, et le
/// titre y reste un titre. Le point de couleur mettait « Bac jaune — emballages, plastique,
/// carton » sur une seule ligne, en un seul poids.
pub fn bin_rows(data: &GuestData) -> Vec<Component> {
    data.bins
        .iter()
        .map(|bin| {
            let mut item = ListItem::new().title(bin.title.get(&data.locale));
            let subtitle = bin.items(&data.locale).join(", ");
            if !subtitle.is_empty() {
                item = item.subtitle(subtitle);
            }
            // « Collecte : mardi, vendredi » — ses propres jours, quand l'hôte les a cochés.
            let days: Vec<&str> = bin
                .days
                .iter()
                .filter_map(|day| crate::collection::parse_day(day))
                .map(|day| time::weekday_name(day, &data.locale))
                .collect();
            if !days.is_empty() {
                let line = t!("guest.bin.days", days = days.join(", "))
                    .unwrap_or_else(|_| days.join(", "));
                item = item.child(Text::new().text(line).variant(TextVariant::Caption));
            }
            if let Some(swatch) = bin_swatch(bin.color.as_deref()) {
                item = item.leading(Leading::Visual(Box::new(LeadingVisual {
                    swatch: Some(swatch),
                    ..LeadingVisual::default()
                })));
            }
            Component::ListItem(item)
        })
        .collect()
}

/// La consigne de sortie, en clair dans la feuille — sauf le jour où elle est déjà remontée en
/// tête du bandeau, où la répéter la banaliserait.
pub fn takeout_note(data: &GuestData) -> Option<Component> {
    let takeout = data.takeout_note.trim();
    if takeout.is_empty() || highlighted(next_up(data)) {
        return None;
    }
    Some(Component::Text(
        Text::new().text(takeout).variant(TextVariant::Caption),
    ))
}

/// Une rangée de point d'apport : ce qu'il accepte, puis la distance quand on sait la mesurer.
pub fn dropoff_row(data: &GuestData, point: &DropoffRow) -> Component {
    let accepts: Vec<String> = point
        .accepted_keys()
        .into_iter()
        .map(|key| t!(key).unwrap_or_else(|_| key.to_string()))
        .collect();
    let distance = point
        .coordinates()
        .zip(data.property)
        .map(|(point, home)| walking_distance(home, point));
    let subtitle = [Some(accepts.join(", ")).filter(|s| !s.is_empty()), distance]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");

    let mut item = ListItem::new()
        .title(point.title.get(&data.locale))
        .leading(Leading::Icon("map-pin".into()));
    if !subtitle.is_empty() {
        item = item.subtitle(subtitle);
    }
    let note = point.note.get(&data.locale);
    if !note.trim().is_empty() {
        item = item.meta(note);
    }
    Component::ListItem(item)
}

/// La rangée du composteur, quand l'hôte en a un et dit où.
pub fn compost_row(data: &GuestData) -> Option<Component> {
    let location = data.compost_location.trim();
    if location.is_empty() {
        return None;
    }
    Some(Component::ListItem(
        ListItem::new()
            .title(t!("guest.compost.title").unwrap_or_else(|_| "i18n:guest.compost.title".into()))
            .subtitle(location)
            .leading(Leading::Icon("recycle".into())),
    ))
}

/// La distance à vol d'oiseau, écrite en mètres sous le kilomètre et en kilomètres au-delà.
///
/// ponytail: à vol d'oiseau, pas par la route — un itinéraire coûterait un appel réseau par point.
/// Pas de durée de marche non plus : « 8 min à pied » serait une estimation déguisée en fait, et
/// l'hôte peut la mettre dans sa note s'il la connaît.
fn walking_distance(from: (f64, f64), to: (f64, f64)) -> String {
    let metres = haversine_metres(from, to).round() as i64;
    if metres < 1000 {
        // Arrondi à 50 m : annoncer « 643 m » sur une ligne droite serait une fausse précision.
        let rounded = ((metres + 25) / 50) * 50;
        t!("guest.distance.metres", value = rounded.max(50))
            .unwrap_or_else(|_| format!("{} m", rounded.max(50)))
    } else {
        let km = (metres as f64) / 1000.0;
        t!("guest.distance.km", value = format!("{km:.1}"))
            .unwrap_or_else(|_| format!("{km:.1} km"))
    }
}

/// Haversine, rayon moyen de la Terre.
fn haversine_metres((lat1, lng1): (f64, f64), (lat2, lng2): (f64, f64)) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let delta_phi = phi2 - phi1;
    let delta_lambda = (lng2 - lng1).to_radians();
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

/// La collecte qui vient, si l'hôte a coché des jours et si l'hôte a une horloge.
///
/// Sans horloge, rien n'est calculé : la phrase de l'hôte reste, ce qui vaut mieux qu'un jour tiré
/// d'une date inventée.
fn next_up(data: &GuestData) -> Option<NextCollection> {
    let now = time::now().ok()?;
    let tz = PropertyTz::parse(&data.timezone);
    next_collection(&data.collection_days, now, tz.as_ref(), data.checkout_at)
}

/// La consigne de sortie est déjà remontée en tête du bandeau.
fn highlighted(next: Option<NextCollection>) -> bool {
    next.is_some_and(|next| next.departure == Some(Departure::Eve))
}

/// Le bandeau de collecte : le jour calculé quand on l'a, la phrase de l'hôte sinon.
fn collection_banner(data: &GuestData, next: Option<NextCollection>) -> Vec<Component> {
    let schedule = data.collection_schedule.trim();
    let takeout = data.takeout_note.trim();

    let Some(next) = next else {
        // Aucun jour coché : exactement le bandeau d'avant, à la lettre.
        if schedule.is_empty() {
            return Vec::new();
        }
        return vec![Component::InfoBanner(
            InfoBanner::new()
                .tone(Tone::Info)
                .title("i18n:guest.collection.title")
                .message(schedule),
        )];
    };

    let put_out_key = format!("i18n:guest.putOut.{}", data.put_out);
    let title = if next.today {
        "i18n:guest.collection.today".to_string()
    } else {
        // Le jour est glissé ici, pas par le livret : une clé `i18n:` traverse telle quelle et n'a
        // pas de quoi recevoir un nom de jour. Si le service de traduction ne répond pas, on
        // retombe sur une clé sans jour, que le livret résout lui-même.
        t!(
            "guest.collection.next",
            day = time::weekday_name(next.day, &data.locale)
        )
        .unwrap_or_else(|_| "i18n:guest.collection.nextPlain".to_string())
    };

    // Le message dit d'abord ce que le départ change ; sinon il garde la phrase de l'hôte, qui
    // précise souvent ce qu'une case à cocher ne dit pas (« avant 7 h », « bac vert une semaine
    // sur deux »).
    let message = match next.departure {
        Some(Departure::SameDay) => "i18n:guest.collection.departureDay",
        Some(Departure::Eve) => "i18n:guest.collection.departureEve",
        // Sans phrase de l'hôte, quand sortir les bacs (§2.1).
        None if schedule.is_empty() => &put_out_key,
        None => schedule,
    };

    // `info` et non le neutre par défaut : la maquette dessine ce bandeau en bleu, et un gris
    // le rangerait avec les fonds de carte.
    let mut banner = InfoBanner::new().tone(Tone::Info).title(title);
    if !message.is_empty() {
        banner = banner.message(message);
    }
    let mut out = vec![Component::InfoBanner(banner)];

    // « Départ la veille d'une collecte → consigne de sortie mise en avant » (§2.7) : c'est le seul
    // cas où le voyageur doit agir avant de fermer la porte, et où lire la consigne plus bas serait
    // la lire trop tard.
    if next.departure == Some(Departure::Eve) && !takeout.is_empty() {
        out.push(Component::Highlight(Highlight::new().text(takeout)));
    }

    out
}
