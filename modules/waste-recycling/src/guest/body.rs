//! Shared guest SDUI body for waste bins.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{ColorDotItem, InfoBanner, ListItem, Text};

use crate::config::bin_swatch;

use super::load::GuestData;

/// Glance / detail shared body: bin rows + optional collection banner.
pub fn build_bins_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    for bin in &data.bins {
        let title = bin
            .title
            .pick_with_fallback(&data.locale, &data.property_locale);
        let subtitle = bin
            .items
            .iter()
            .map(|item| item.pick_with_fallback(&data.locale, &data.property_locale))
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        if let Some(swatch) = bin_swatch(bin.color.as_deref()) {
            children.push(Component::ColorDotItem(
                ColorDotItem::new()
                    .label(format!("{title} — {subtitle}"))
                    .swatch(swatch),
            ));
        } else {
            let mut item = ListItem::new().title(title);
            if !subtitle.is_empty() {
                item = item.subtitle(subtitle);
            }
            if enriched {
                for line in &bin.items {
                    let text = line.pick_with_fallback(&data.locale, &data.property_locale);
                    if text.trim().is_empty() {
                        continue;
                    }
                    item = item.child(Text::new().text(text).variant(TextVariant::Caption));
                }
            }
            children.push(Component::ListItem(item));
        }
    }

    if !data.collection_schedule.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.collection.title")
                .message(data.collection_schedule.clone()),
        ));
    }

    children
}
