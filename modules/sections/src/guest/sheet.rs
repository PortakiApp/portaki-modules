//! Guest explore sheet — full section bodies.

use portaki_sdk::sdui::common::Author;
use portaki_sdk::sdui::primitives::{RichText, Stack};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::Component;

use super::home::full_sections_stack;
use crate::model::SectionView;

/// Le mot complet, signé de l'hôte (§8).
///
/// Le module **ne compose pas** la signature : il marque le bloc, et la coquille y met le nom, le
/// rôle traduit, les initiales et la photo — c'est ce que dit la doc d'`Author` (« always the
/// host, filled by the platform »). Un module qui redériverait l'hôte en afficherait un autre que
/// le bandeau d'état, qui lit la même source.
///
/// La signature va sous le texte entier, pas sous chaque section : c'est un mot, pas une suite de
/// billets.
pub fn build_sheet_surface(sections: &[SectionView]) -> Surface {
    Surface::new(Component::Stack(Stack::new().gap(14.0).children(vec![
        full_sections_stack(sections),
        Component::RichText(RichText::new().signature(Author::default())),
    ])))
    .with_id(crate::guest::EXPLORE_SHEET)
}
