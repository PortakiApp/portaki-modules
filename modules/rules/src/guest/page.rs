//! Guest explore detail — full rules list (page body).
//!
//! Design: `pageModules` → fullscreen page; body is the elevated card of rows
//! (same block as the Séjour glance, without the card header — shell supplies it).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::SurfaceLevel;
use portaki_sdk::sdui::primitives::{Card, Stack};
use portaki_sdk::sdui::surface::Surface;

use super::home::rules_stack;
use crate::content::RulesPayload;

pub fn build_detail_page(payload: &RulesPayload) -> Surface {
    Surface::new(Stack::new().gap(0.0).child(Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .child(rules_stack(&payload.items)),
    )))
    .with_id(crate::ids::EXPLORE_DETAIL)
}
