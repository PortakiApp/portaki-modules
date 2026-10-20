//! Livret : la feuille montre le mot entier.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Author;
use portaki_sdk::sdui::primitives::{RichText, Stack};

use super::block::{blocks, empty, visible};

/// Le mot entier ; la signature va sous le texte, pas sous chaque section, et c'est la coquille
/// qui la remplit (`Author`, « always the host, filled by the platform »).
#[portaki_sdk::surface(guest, id = "explore.sheet")]
pub fn render_explore_sheet(ctx: GuestContext) -> Result<Surface> {
    let sections = visible(&ctx)?;
    if sections.is_empty() {
        return Ok(empty(EXPLORE_SHEET));
    }
    let all = Stack::new()
        .gap(12.0)
        .children(blocks(&ctx, &sections, sections.len()));
    let sheet = Stack::new().gap(14.0).children(vec![
        all.into(),
        RichText::new().signature(Author::default()).into(),
    ]);
    Ok(Surface::new(sheet).with_id(EXPLORE_SHEET))
}
