//! Ce qu'une règle affiche dans le livret.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Row};
use portaki_sdk::sdui::common::BadgeSpec;

use crate::{Rule, RuleStatus};

pub fn rows(ctx: &Context, rules: &[Rule]) -> Vec<Row> {
    rules.iter().map(|r| row(ctx, r)).collect()
}

/// Titre, précision · horaires, et l'étiquette du statut — jamais verte pour « Autorisé » : un
/// feu vert se lit comme une invitation ; rien pour une règle neutre.
pub fn row(ctx: &Context, rule: &Rule) -> Row {
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
        .detail(hours_label(rule))
        .badge(badge)
}

/// « 22:00 – 08:00 », vide sans horaires ; le livret et le tableau de bord l'affichent pareil.
pub(crate) fn hours_label(rule: &Rule) -> String {
    portaki_sdk::sdui::hours::parse_range(&rule.hours)
        .map(|(start, end)| format!("{start} – {end}"))
        .unwrap_or_default()
}
