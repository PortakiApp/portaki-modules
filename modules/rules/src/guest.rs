//! Livret (spec Règlement §2.8) : la carte d'accueil liste les premières règles, la page les
//! groupe par thème. Les gabarits du SDK font la mise en page, le vide et « voir tout ».

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, GroupedList, ListCard, Row};
use portaki_sdk::sdui::common::BadgeSpec;

use crate::{Rule, RuleStatus, Rules, Theme};

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let rules = Rules::load(&ctx)?;
    Ok(ListCard::new(rows(&ctx, rules.rows()), EXPLORE_DETAIL)
        .titled(IconName::Scale, "nav.rules")
        .limit(rules.meta().limit())
        .build(HOME_CARD))
}

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

fn rows(ctx: &Context, rules: &[Rule]) -> Vec<Row> {
    rules.iter().map(|r| row(ctx, r)).collect()
}

/// Titre, précision · horaires, et l'étiquette du statut — jamais verte pour « Autorisé » : un
/// feu vert se lit comme une invitation ; rien pour une règle neutre.
fn row(ctx: &Context, rule: &Rule) -> Row {
    let badge = match rule.status {
        RuleStatus::Important => Some(Tone::Warning),
        RuleStatus::Allowed | RuleStatus::Forbidden => Some(Tone::Neutral),
        RuleStatus::Neutral => None,
    }
    .map(|tone| {
        BadgeSpec::new(
            t!(&format!("rule.status.{}", rule.status.wire())).unwrap_or_default(),
            tone,
        )
    });
    Row::new(rule.title.for_ctx(ctx))
        .icon(rule.icon())
        .detail(rule.detail.for_ctx(ctx))
        .detail(rule.hours_label())
        .badge(badge)
}
