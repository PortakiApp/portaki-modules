//! Le bloc « Adresses » de la page publique du logement — ce qu'un visiteur sans séjour voit.
//!
//! Un arbre statique, construit ici de zéro plutôt qu'avec les rangées du livret : celles-ci
//! portent une navigation et la position de l'adresse (la vignette de plan). Seuls la photo, le
//! nom, la catégorie et une distance approximative en sortent. Jamais l'avantage, le prix, le
//! téléphone, les horaires, l'adresse ni le conseil de l'hôte : ils restent dans le livret.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Grid, Image, Section, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::item::{haversine_m, DRIVE_KMH, WALK_KMH};
use crate::config::{valid_coords, ModuleConfig, SpotRow};

/// La largeur d'une carte d'adresse : trois côte à côte sur un écran large, une sur un téléphone.
const CARD_WIDTH: f64 = 200.0;

/// Au-delà, on n'y va plus à pied : la distance s'annonce en voiture.
const WALK_LIMIT_M: f64 = 2_000.0;

/// La `Section` du bloc. Sans enfant quand le bloc est éteint ou qu'il manque des adresses : la
/// plateforme masque alors le bloc.
///
/// `property` est le centre flouté que la plateforme envoie au visiteur public : toute distance
/// en est approximative, et se dit comme telle.
pub fn build_public_section(
    config: &ModuleConfig,
    locale: &str,
    property: Option<(f64, f64)>,
) -> Surface {
    let mut section = Section::new()
        .title("i18n:public.title")
        .subtitle("i18n:public.eyebrow");
    let spots = config.public_spots();
    if !spots.is_empty() {
        section = section
            .child(
                Grid::new().minColumnWidth(CARD_WIDTH).plain(true).children(
                    spots
                        .iter()
                        .map(|spot| spot_card(spot, locale, property))
                        .collect(),
                ),
            )
            .child(
                Text::new()
                    .text("i18n:public.more")
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            );
    }
    Surface::new(section).with_id(crate::guest::PROPERTY_PUBLIC)
}

fn spot_card(spot: &SpotRow, locale: &str, property: Option<(f64, f64)>) -> Component {
    let title = spot.title.get(locale).trim().to_string();
    let mut children: Vec<Component> = Vec::new();
    if let Some(reference) = spot.photo_ref() {
        children.push(
            Image::new()
                .url(reference.to_string())
                .alt(title.clone())
                .aspectRatio("4 / 3")
                .into(),
        );
    }
    children.push(Text::new().text(title).variant(TextVariant::Title).into());
    if let Some(category) = spot
        .category
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        children.push(
            Text::new()
                .text(category.to_string())
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    // La distance calculée, jamais le texte libre `distance` de l'hôte : il peut nommer une rue.
    if let Some(line) = approximate_distance(spot, property) {
        children.push(Text::new().text(line).variant(TextVariant::Caption).into());
    }
    Stack::new().gap(6.0).children(children).into()
}

/// « ~15 min à pied » ou « ~10 min en voiture », depuis le centre flouté du logement.
///
/// Arrondie aux cinq minutes supérieures : le centre est flouté sur 500 m, soit six minutes de
/// marche ; annoncer « 13 min » dirait une précision que la page n'a pas.
fn approximate_distance(spot: &SpotRow, property: Option<(f64, f64)>) -> Option<String> {
    let (property_lat, property_lng) = property.and_then(|(lat, lng)| valid_coords(lat, lng))?;
    let (lat, lng) = spot.coords()?;
    let metres = haversine_m(property_lat, property_lng, lat, lng);
    Some(if metres < WALK_LIMIT_M {
        let minutes = rounded_minutes(metres, WALK_KMH);
        t!("public.walk", count = minutes).unwrap_or_else(|_| format!("~{minutes} min"))
    } else {
        let minutes = rounded_minutes(metres, DRIVE_KMH);
        t!("public.drive", count = minutes).unwrap_or_else(|_| format!("~{minutes} min"))
    })
}

fn rounded_minutes(metres: f64, kmh: f64) -> i64 {
    let minutes = (metres / 1000.0 / kmh * 60.0).ceil() as i64;
    ((minutes + 4) / 5 * 5).max(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minutes_round_up_to_five() {
        // 1 km à pied = 12 min → 15 ; 100 m = 2 min → 5 ; 2,5 km en voiture = 5 min → 5.
        assert_eq!(rounded_minutes(1_000.0, WALK_KMH), 15);
        assert_eq!(rounded_minutes(100.0, WALK_KMH), 5);
        assert_eq!(rounded_minutes(2_500.0, DRIVE_KMH), 5);
        assert_eq!(rounded_minutes(1_250.0, WALK_KMH), 15);
    }
}
