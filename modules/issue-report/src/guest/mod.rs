//! Livret : une surface par fichier ; les deux styles de légende sont communs.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::Text;

pub mod form;
pub mod home_card;

pub(crate) fn caption(text: &str) -> Component {
    Text::new().text(text).variant(TextVariant::Caption).into()
}

pub(crate) fn note(text: &str) -> Component {
    Text::new()
        .text(text)
        .variant(TextVariant::Caption)
        .emphasis(Emphasis::Subtle)
        .into()
}
