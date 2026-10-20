//! La page : toutes les règles, groupées par thème dans l'ordre de l'hôte.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, GroupedList};

use super::row::row;
use crate::{Rule, Rules, Theme};

#[portaki_sdk::surface(guest, id = "explore.detail", path = "rules", label_key = "nav.rules")]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let rules = Rules::load(&ctx)?;
    let group = |r: &Rule| match r.theme {
        Theme::Other => String::new(),
        theme => theme.label_key(),
    };
    Ok(GroupedList::new(rules.rows(), group, |r| row(&ctx, r))
        .icon(IconName::Scale)
        .build(EXPLORE_DETAIL))
}
