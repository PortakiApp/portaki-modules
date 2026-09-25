//! Shared guest SDUI body for facility hours.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{InfoBanner, KeyValue, ListItem, Text};

use super::load::GuestData;

pub fn build_hours_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if !data.general_note.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.note.title")
                .message(data.general_note.clone()),
        ));
    }

    for facility in &data.facilities {
        let title = facility.title.get(&data.locale);
        let lines = facility.lines(&data.locale);
        let hours = facility
            .hours
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| (!lines.is_empty()).then(|| lines.join(" · ")))
            .unwrap_or_default();

        if enriched {
            let mut item = ListItem::new().title(title);
            if !hours.is_empty() {
                item = item.subtitle(hours.clone());
            }
            for line in lines {
                item = item.child(Text::new().text(line).variant(TextVariant::Caption));
            }
            let note = facility.note.get(&data.locale);
            if !note.trim().is_empty() {
                item = item.child(Text::new().text(note).variant(TextVariant::Caption));
            }
            children.push(Component::ListItem(item));
        } else {
            children.push(Component::KeyValue(KeyValue::new().key(title).value(hours)));
        }
    }

    children
}
