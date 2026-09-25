//! Guest explore / bottom-sheet detail surface.

use portaki_sdk::sdui::primitives::Stack;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_bins_body;
use super::load::GuestData;

/// Body-only tree for the bottom sheet (shell supplies header chrome).
pub fn build_detail_surface(data: &GuestData) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(build_bins_body(data, true)))
        .with_id(crate::guest::EXPLORE_DETAIL)
}
