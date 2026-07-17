//! Guest explore detail — full appliance list (Booklet page body).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Stack};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::home::devices_list;
use crate::content::AppliancesPayload;

/// List page — elevated card of rows (design: Portaki Guest `appliances` block).
pub fn build_detail_page(payload: &AppliancesPayload) -> Surface {
    Surface::new(Stack::new().gap(json!(12)).children(vec![Component::Card(
            Card::new()
                .surface(portaki_sdk::sdui::common::SurfaceLevel::Elevated)
                .children(devices_list(payload)),
        )]))
    .with_id("explore.detail")
}
