//! Guest home booklet card — design Portaki Guest `rules` section (Séjour).
//!
//! Card: scale + « Règlement intérieur » + Ouvrir → fullscreen page.
//! Body: icon rows (title + optional subtitle), glance of first rules.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{BadgeSpec, Leading, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Button, Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{RuleItem, RuleStatus, RulesPayload};

/// Design glance shows four rules on the Séjour card.
const CARD_GLANCE_LIMIT: usize = 4;

pub fn build_home_card(payload: &RulesPayload) -> Surface {
    // Les essentielles d'abord : important → autorisé → neutres (§2.8). Le tri décide donc *quelles*
    // quatre règles le voyageur voit, pas seulement dans quel ordre.
    let ranked = payload.by_weight();
    let total = ranked.len();
    let shown: Vec<&RuleItem> = ranked.into_iter().take(CARD_GLANCE_LIMIT).collect();

    // Aucune règle : la carte disparaît (§2.8). Un logement sans règlement n'a pas de règlement à
    // annoncer, et une carte qui dit « rien pour l'instant » occupe l'accueil pour ne rien dire.
    if shown.is_empty() {
        return Surface::new(Stack::new()).with_id(crate::guest::HOME_CARD);
    }

    let mut children: Vec<Component> = shown.into_iter().map(rule_list_item).collect();

    // « Voir les N règles », et seulement s'il en reste : à quatre ou moins, le bouton promettrait
    // une liste identique à celle qu'on lit déjà (§2.8).
    if total > CARD_GLANCE_LIMIT {
        let label = t!("home.card.seeAll", count = total)
            .unwrap_or_else(|_| "i18n:home.card.seeAllPlain".to_string());
        children.push(Component::Button(
            Button::new()
                .label(label)
                .variant(ButtonVariant::Outline)
                .action(Action::open_overlay(
                    OverlayPresentation::Fullscreen,
                    crate::guest::EXPLORE_DETAIL,
                    OverlayArgs::new()
                        .icon(IconName::Scale)
                        .title("i18n:nav.rules"),
                )),
        ));
    }

    // Prefer nav.* — shell ships `nav.rules`; avoids colliding home.card titles.
    Surface::new(
        Card::new()
            .icon(IconName::Scale)
            .title("i18n:nav.rules")
            .subtitle(count_line(
                "home.card.subtitle",
                "home.card.subtitle.one",
                total,
            ))
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Scale)
                    .title("i18n:nav.rules"),
            ))
            .children(children),
    )
    .with_id(crate::guest::HOME_CARD)
}

pub fn rule_list_item(item: &RuleItem) -> Component {
    let icon_name = if item.icon.trim().is_empty() {
        "check-circle".to_string()
    } else {
        normalize_guest_icon(&item.icon)
    };
    let mut list = ListItem::new()
        .title(item.title.clone())
        .leading(Leading::Icon(icon_name));
    if !item.subtitle.trim().is_empty() {
        list = list.subtitle(item.subtitle.clone());
    }
    if let Some(badge) = status_badge(item.status) {
        list = list.trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(badge),
            ..TrailingVisual::default()
        })));
    }
    Component::ListItem(list)
}

/// Une ligne qui compte des règles, au singulier quand il n'y en a qu'une.
pub fn count_line(plural_key: &str, one_key: &str, count: usize) -> String {
    let key = if count == 1 { one_key } else { plural_key };
    t!(key, count = count).unwrap_or_else(|_| format!("i18n:{key}"))
}

/// L'étiquette d'un statut (§2.8) : warning « Important », success « Autorisé ».
///
/// Neutre ne porte rien — une règle sur trois serait étiquetée « Normal », ce qui ne dit rien et
/// affaiblit les deux autres.
fn status_badge(status: RuleStatus) -> Option<BadgeSpec> {
    let (key, tone) = match status {
        RuleStatus::Important => ("rule.status.important", Tone::Warning),
        RuleStatus::Allowed => ("rule.status.allowed", Tone::Success),
        RuleStatus::Neutral => return None,
    };
    Some(BadgeSpec::new(
        t!(key).unwrap_or_else(|_| format!("i18n:{key}")),
        tone,
    ))
}

pub fn rules_stack(items: &[RuleItem]) -> Component {
    rules_rows(
        items
            .iter()
            .filter(|item| !item.title.trim().is_empty())
            .collect(),
    )
}

/// Un bloc de rangées, ou la phrase du vide quand il n'y a rien à montrer.
pub fn rules_rows(items: Vec<&RuleItem>) -> Component {
    let children: Vec<Component> = items.into_iter().map(rule_list_item).collect();
    if children.is_empty() {
        return Component::Text(
            Text::new()
                .text("i18n:home.card.empty.description")
                .variant(TextVariant::Body),
        );
    }
    Component::Stack(Stack::new().gap(0.0).children(children))
}

fn normalize_guest_icon(icon: &str) -> String {
    match icon.trim() {
        "paw-print" | "pets" => "gift".into(),
        "volume-x" | "volume-2" | "noise" => "minus".into(),
        other => other.to_string(),
    }
}
