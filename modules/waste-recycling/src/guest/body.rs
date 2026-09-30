//! Shared guest SDUI body for waste bins.

use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{ColorDotItem, Highlight, InfoBanner, ListItem, Text};

use crate::collection::{next_collection, Departure, NextCollection};
use crate::config::bin_swatch;

use super::load::GuestData;

/// Glance / detail shared body: collection banner, bin rows, takeout note.
pub fn build_bins_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let next = next_up(data);
    // Le bandeau vient en tête (§2.7) : c'est la seule chose de cette carte qui change d'un jour à
    // l'autre, et donc la seule qu'on relit.
    let mut children = collection_banner(data, next);

    for bin in &data.bins {
        let title = bin.title.get(&data.locale);
        let items = bin.items(&data.locale);
        let subtitle = items.join(", ");

        if let Some(swatch) = bin_swatch(bin.color.as_deref()) {
            children.push(Component::ColorDotItem(
                ColorDotItem::new()
                    .label(format!("{title} — {subtitle}"))
                    .swatch(swatch),
            ));
        } else {
            let mut item = ListItem::new().title(title);
            if !subtitle.is_empty() {
                item = item.subtitle(subtitle);
            }
            if enriched {
                for line in items {
                    item = item.child(Text::new().text(line).variant(TextVariant::Caption));
                }
            }
            children.push(Component::ListItem(item));
        }
    }

    // La consigne de sortie, en clair dans la feuille — sauf le jour où elle est déjà remontée en
    // tête, où la répéter la banaliserait.
    let takeout = data.takeout_note.trim();
    if enriched && !takeout.is_empty() && !highlighted(next) {
        children.push(Component::Text(
            Text::new().text(takeout).variant(TextVariant::Caption),
        ));
    }

    children
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
                .title("i18n:guest.collection.title")
                .message(schedule),
        )];
    };

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
        None => schedule,
    };

    let mut banner = InfoBanner::new().title(title);
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
