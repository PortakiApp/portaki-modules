//! Guest home booklet card — teaser + first sections.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Divider, Markdown, RichText, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::content::is_tiptap_doc;
use crate::model::SectionView;

const CARD_SECTION_LIMIT: usize = 2;

pub fn build_home_card(sections: &[SectionView]) -> Surface {
    Surface::new(
        Card::new()
            .icon("home")
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::ids::EXPLORE_SHEET,
                OverlayArgs::new()
                    .icon("home")
                    .title("i18n:home.card.title"),
            ))
            .children(section_blocks(sections, CARD_SECTION_LIMIT)),
    )
    .with_id(crate::ids::HOME_CARD)
}

pub fn section_blocks(sections: &[SectionView], limit: usize) -> Vec<Component> {
    let mut children = Vec::new();
    for (index, section) in sections.iter().take(limit).enumerate() {
        if index > 0 {
            children.push(Component::Divider(Divider::new()));
        }
        if !section.title.trim().is_empty() {
            children.push(Component::Text(
                Text::new()
                    .text(section.title.clone())
                    .variant(TextVariant::Caption),
            ));
        }
        if let Some(body) = body_component(&section.body_markdown) {
            children.push(body);
        }
    }
    if children.is_empty() {
        children.push(Component::Text(
            Text::new()
                .text("i18n:home.card.empty.description")
                .variant(TextVariant::Body),
        ));
    }
    children
}

pub fn full_sections_stack(sections: &[SectionView]) -> Component {
    Component::Stack(
        Stack::new()
            .gap(12.0)
            .children(section_blocks(sections, sections.len())),
    )
}

fn body_component(body: &str) -> Option<Component> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    if is_tiptap_doc(trimmed) {
        Some(Component::RichText(RichText::new().content(trimmed)))
    } else {
        Some(Component::Markdown(Markdown::new().content(trimmed)))
    }
}
