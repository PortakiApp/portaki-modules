//! La carte d'accueil : les premières règles, « voir tout » au-delà de la limite de l'hôte.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::ListCard;

use super::explore_detail::EXPLORE_DETAIL;
use super::row::rows;
use crate::Rules;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let rules = Rules::load(&ctx)?;
    Ok(ListCard::new(rows(&ctx, rules.rows()), EXPLORE_DETAIL)
        .titled(IconName::Scale, "nav.rules")
        .limit(rules.meta().limit())
        .build(HOME_CARD))
}
