//! Guest explore detail — full rules list (page body).
//!
//! Design: `pageModules` → fullscreen page ; une phrase d'intro, puis un bloc par thème (§2.8).
//! Sans thème — ou avec un seul — c'est un bloc unique sans titre : un en-tête qui ne distingue
//! rien n'est qu'une ligne de plus à lire.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::SurfaceLevel;
use portaki_sdk::sdui::primitives::{Card, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::home::{count_line, rules_rows, rules_stack};
use crate::content::RulesPayload;

pub fn build_detail_page(payload: &RulesPayload) -> Surface {
    let groups = payload.by_theme();
    let count = payload.named().count();

    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(count_line(
                "guest.detail.intro",
                "guest.detail.intro.one",
                count,
            ))
            .variant(TextVariant::Caption),
    )];

    // Le détail garde l'ordre de l'hôte, thèmes compris : c'est la référence complète, pas un tri.
    // Seule la carte d'accueil hiérarchise (§2.8).
    let grouped = groups.len() > 1;
    if grouped {
        for (theme, rules) in groups {
            let mut card = Card::new().surface(SurfaceLevel::Elevated);
            if !theme.is_empty() {
                card = card.icon(IconName::Scale).title(theme);
            }
            children.push(Component::Card(card.child(rules_rows(rules))));
        }
    } else {
        children.push(Component::Card(
            Card::new()
                .surface(SurfaceLevel::Elevated)
                .child(rules_stack(&payload.items)),
        ));
    }

    Surface::new(Stack::new().gap(0.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}
