//! Guest home booklet card — design Portaki Guest `rules` section (Séjour).
//!
//! Card: scale + « Règlement intérieur » + Ouvrir → fullscreen page.
//! Body: icon rows (title + optional subtitle), glance of first rules.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{RuleItem, RulesPayload};

/// Design glance shows four rules on the Séjour card.
const CARD_GLANCE_LIMIT: usize = 4;

pub fn build_home_card(payload: &RulesPayload) -> Surface {
    let items: Vec<&RuleItem> = payload
        .items
        .iter()
        .filter(|item| !item.title.trim().is_empty())
        .take(CARD_GLANCE_LIMIT)
        .collect();

    let children: Vec<Component> = if items.is_empty() {
        vec![Component::Text(
            Text::new()
                .text("i18n:home.card.empty.description")
                .variant(TextVariant::Body),
        )]
    } else {
        items.into_iter().map(rule_list_item).collect()
    };

    // Prefer nav.* — shell ships `nav.rules`; avoids colliding home.card titles.
    Surface::new(
        Card::new()
            .icon("scale")
            .title("i18n:nav.rules")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new().icon("scale").title("i18n:nav.rules"),
            ))
            .children(children),
    )
    .with_id(crate::ids::HOME_CARD)
}

pub fn rule_list_item(item: &RuleItem) -> Component {
    let icon_name = if item.icon.trim().is_empty() {
        "check-circle".to_string()
    } else {
        normalize_guest_icon(&item.icon)
    };
    let mut list = ListItem::new().title(item.title.clone()).leading(icon_name);
    if !item.subtitle.trim().is_empty() {
        list = list.subtitle(item.subtitle.clone());
    }
    Component::ListItem(list)
}

pub fn rules_stack(items: &[RuleItem]) -> Component {
    let children: Vec<Component> = items
        .iter()
        .filter(|item| !item.title.trim().is_empty())
        .map(rule_list_item)
        .collect();
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
