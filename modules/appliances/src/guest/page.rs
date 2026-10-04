//! Guest explore detail — full appliance list (Booklet page body).

use portaki_sdk::sdui::primitives::Stack;
use portaki_sdk::sdui::surface::Surface;

use super::home::devices_list;
use crate::content::AppliancesPayload;

/// La liste complète : une carte par pièce (§2.4). Les cartes viennent de `devices_list`, qui sert
/// aussi l'état vide — la page ne les enveloppe plus dans une carte de plus.
pub fn build_detail_page(payload: &AppliancesPayload) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(devices_list(payload)))
        .with_id(crate::guest::EXPLORE_DETAIL)
}
