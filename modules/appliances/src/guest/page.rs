//! Guest explore detail — full appliance list (Booklet page body).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::home::devices_list;
use crate::content::AppliancesPayload;

pub fn build_detail_page(payload: &AppliancesPayload) -> Surface {
    Surface::new(
        Stack::new().gap(json!(12)).children(vec![
            Component::Text(
                Text::new()
                    .text(json!("i18n:explore.detail.subtitle"))
                    .variant(json!("body")),
            ),
            Component::Card(
                Card::new()
                    .icon(json!("plug"))
                    .title(json!("i18n:home.card.title"))
                    .children(devices_list(payload)),
            ),
        ]),
    )
    .with_id("explore.detail")
}
