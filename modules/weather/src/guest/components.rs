//! Shared guest SDUI building blocks.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, Tone};
use portaki_sdk::sdui::primitives::{Icon, Stack, Text};

pub fn metric_label(icon: IconName, label: &str) -> Component {
    Component::Stack(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(6.0)
            .child(Icon::new().name(icon).size(14.0))
            .child(
                Text::new()
                    .text(label)
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Strong),
            ),
    )
}

/// One metric tile: icon + label on top, value below — two tiles per grid row.
pub fn metric_tile(
    icon: IconName,
    label: &str,
    value: &str,
    value_tone: Option<Tone>,
) -> Component {
    let mut value_text = Text::new().text(value).variant(TextVariant::Caption);
    if let Some(tone) = value_tone {
        value_text = value_text.tone(tone);
    }
    Component::Stack(
        Stack::new()
            .gap(2.0)
            .child(metric_label(icon, label))
            .child(Component::Text(value_text)),
    )
}
