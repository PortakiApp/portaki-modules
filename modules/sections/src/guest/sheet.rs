//! Guest explore sheet — full section bodies.

use portaki_sdk::sdui::surface::Surface;

use super::home::full_sections_stack;
use crate::model::SectionView;

pub fn build_sheet_surface(sections: &[SectionView]) -> Surface {
    Surface::new(full_sections_stack(sections)).with_id(crate::guest::EXPLORE_SHEET)
}
