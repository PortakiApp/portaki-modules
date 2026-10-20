//! What both guest cards share: the lists once loaded, or the line that stands in for them.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Text};
use portaki_sdk::sdui::surface::Surface;

use super::home::build_home_card;
use super::load::{load_guest_checklist, GuestLoad};

pub(super) fn render_card(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    departure_only: bool,
) -> Result<Surface> {
    Ok(match load_guest_checklist(ctx, departure_only)? {
        GuestLoad::NoItems => message_card(surface_id, IconName::ListChecks, "home.card.empty"),
        // Pendant le séjour, une liste qui n'est pas encore ouverte ne dit rien au voyageur : un
        // état vide sans texte, que le livret masque (§1.9). « Pas encore » n'était que du bruit,
        // sur chaque livret. L'après-séjour garde sa phrase.
        GuestLoad::NotYet if !departure_only => {
            Surface::new(EmptyState::new().icon(IconName::ListChecks)).with_id(surface_id)
        }
        GuestLoad::NotYet => message_card(surface_id, IconName::ClockCircle, "home.card.notYet"),
        GuestLoad::Ready(data) => build_home_card(&data, surface_id),
    })
}

/// The card with one line of text instead of lists.
fn message_card(surface_id: SurfaceId, icon: IconName, key: &str) -> Surface {
    Surface::new(
        Card::new().icon(icon).title("i18n:home.card.title").child(
            Text::new()
                .text(format!("i18n:{key}"))
                .variant(TextVariant::Body),
        ),
    )
    .with_id(surface_id)
}
