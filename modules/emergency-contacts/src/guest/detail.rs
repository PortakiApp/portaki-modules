//! Guest explore / bottom-sheet detail surface.

use portaki_sdk::sdui::primitives::Stack;
use portaki_sdk::sdui::surface::Surface;

use super::body::{build_contacts_body, country_numbers};
use super::load::GuestData;

pub fn build_detail_surface(data: &GuestData) -> Surface {
    // Les mêmes tuiles qu'à l'accueil : la sous-page ne doit pas être le seul endroit où l'on
    // trouve le 112, ni l'accueil (§2.16). `children` remplace la liste, d'où la construction
    // en une fois plutôt qu'un `child` suivi d'un `children`.
    let mut body = vec![country_numbers(&data.locale)];
    body.extend(build_contacts_body(data, true));

    Surface::new(Stack::new().gap(12.0).children(body)).with_id(crate::guest::EXPLORE_DETAIL)
}
