//! Stay detail encart (spec Votre avis §1) : « Note : 5 / 5 · lien public proposé », ou « Pas
//! encore noté ». Le lien se dit proposé d'après ce que le voyageur a vu en notant
//! (`StoredReview::link_offered`) : l'hôte ne reçoit pas la plateforme du séjour, et un lien
//! configuré n'est pas montré à un voyageur venu d'une autre plateforme (`shows_link`). Le
//! commentaire privé reste sur la page de statistiques.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Star
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let review = match stay_id {
        Some(stay_id) => crate::commands::review_for_stay(stay_id)?,
        None => None,
    };
    let line = match review {
        None => "i18n:host.stay.notYet".to_string(),
        Some(review) => {
            let rating = review.rating.to_string();
            let key = if review.link_offered {
                "host.stay.ratedWithLink"
            } else {
                "host.stay.rated"
            };
            t!(key, rating = &rating)
                .ok()
                .filter(|text| text.contains(&rating))
                .unwrap_or_else(|| format!("Note : {rating} / 5"))
        }
    };
    let card = Card::new()
        .icon(IconName::Star)
        .title("i18n:host.stay.title")
        .child(Text::new().text(line).variant(TextVariant::Body));
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}
