//! Guest explore / bottom-sheet detail surface.

use portaki_sdk::sdui::primitives::Stack;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_wifi_body;
use super::load::GuestData;

pub fn build_detail_surface(data: &GuestData) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(build_wifi_body(data, true)))
        .with_id(crate::guest::EXPLORE_DETAIL)
}
