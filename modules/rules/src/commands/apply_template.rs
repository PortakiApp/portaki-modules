//! « Partir d'un modèle » : les cinq règles les plus courantes, sur un règlement vide.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;

use crate::{Rule, RuleStatus, Rules, Theme};

/// « Partir d'un modèle » (§4) : silence, animaux, tabac, fêtes, visiteurs, dans la langue de
/// l'hôte ; seulement sur un règlement vide, un second clic n'écrase rien.
#[portaki_sdk::command(
    name = "applyTemplate",
    example(label = "Partir d'un modèle", input = "{}")
)]
pub fn apply_template(ctx: Context, _args: EmptyArgs) -> Result<()> {
    let mut rules = Rules::load(&ctx)?;
    if !rules.is_empty() {
        return Ok(());
    }
    let lang = ctx.lang();
    let text = |key: &str| I18nText::default().with(lang.clone(), t!(key).unwrap_or_default());
    let rule = |key: &str, status, theme, hours: &str| Rule {
        id: format!("t-{key}"),
        title: text(&format!("host.template.{key}.title")),
        detail: text(&format!("host.template.{key}.detail")),
        status,
        theme,
        hours: hours.into(),
    };
    rules.rows_mut().extend([
        rule(
            "silence",
            RuleStatus::Important,
            Theme::Noise,
            "22:00-08:00",
        ),
        rule("pets", RuleStatus::Allowed, Theme::Pets, ""),
        rule("smoking", RuleStatus::Forbidden, Theme::Smoking, ""),
        rule("parties", RuleStatus::Forbidden, Theme::Parties, ""),
        rule("visitors", RuleStatus::Neutral, Theme::Visitors, ""),
    ]);
    Rules::save(&rules)
}
