//! Le bloc « Randonnées » de la page publique du logement — ce qu'un visiteur sans séjour voit.
//!
//! Un arbre statique, construit ici de zéro plutôt qu'avec les rangées du livret : celles-ci
//! portent une navigation, et rien d'autre que la photo, le titre, le niveau et les mesures ne
//! doit en sortir. Jamais la trace, le lien tiers, l'adresse ni la position du départ.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Badge, Grid, Image, Section, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::rows::stats_line;
use crate::config::{ModuleConfig, TrailRow};
use crate::format;

/// La largeur d'une carte d'itinéraire : deux côte à côte sur un écran large, une sur un téléphone.
const CARD_WIDTH: f64 = 240.0;

/// La `Section` du bloc. Sans enfant quand le bloc est éteint ou qu'il manque des itinéraires :
/// la plateforme masque alors le bloc.
pub fn build_public_section(config: &ModuleConfig, locale: &str) -> Surface {
    let mut section = Section::new()
        .title("i18n:public.title")
        .subtitle("i18n:public.eyebrow");
    let trails = config.public_trails();
    if !trails.is_empty() {
        section = section.child(
            Grid::new()
                .minColumnWidth(CARD_WIDTH)
                .plain(true)
                .children(trails.iter().map(|t| trail_card(t, locale)).collect()),
        );
    }
    Surface::new(section).with_id(crate::guest::PROPERTY_PUBLIC)
}

fn trail_card(trail: &TrailRow, locale: &str) -> Component {
    let title = trail.title.get(locale).trim().to_string();
    let mut children: Vec<Component> = Vec::new();
    if let Some(reference) = trail.photo_ref() {
        children.push(
            Image::new()
                .url(reference.to_string())
                .alt(title.clone())
                .aspectRatio("16 / 9")
                .into(),
        );
    }
    if let Some(level) = trail.level_key() {
        children.push(Badge::new().label(format::level(level)).into());
    }
    children.push(Text::new().text(title).variant(TextVariant::Title).into());
    if let Some(line) = stats_line(trail, false) {
        children.push(
            Text::new()
                .text(line)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    if trail.starts_at_property == Some(true) {
        children.push(
            Text::new()
                .text("i18n:public.startAtProperty")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    Stack::new().gap(6.0).children(children).into()
}
