//! The guest surfaces' shared empty state.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

/// Nothing in the catalog: nothing the guest can report.
pub(super) fn empty_catalog_card(surface_id: SurfaceId) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Package)
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.empty")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}
