//! Shared guest SDUI body for emergency contacts.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{InfoBanner, Link, ListItem, Pressable, Text};

use super::load::GuestData;

fn tel_action(phone: &str) -> Action {
    let tel = phone.replace(|c: char| c.is_whitespace(), "");
    Action::external(format!("tel:{tel}"))
}

pub fn build_contacts_body(data: &GuestData, show_emergency_banner: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if show_emergency_banner {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.emergency.title")
                .message("i18n:guest.emergency.message"),
        ));
    }

    if !data.host_phone.is_empty() {
        let action = tel_action(&data.host_phone);
        children.push(Component::Pressable(
            Pressable::new().action(action.clone()).child(
                ListItem::new()
                    .title("i18n:guest.host.label")
                    .subtitle(data.host_phone.clone())
                    .trailing("i18n:guest.call"),
            ),
        ));
    }

    for contact in &data.contacts {
        let label = contact.label.get(&data.locale);
        let mut item = ListItem::new()
            .title(label)
            .subtitle(contact.phone.clone())
            .trailing("i18n:guest.call");
        if let Some(cat) = contact.category.as_deref().filter(|c| !c.trim().is_empty()) {
            item = item.leading(cat);
        }
        let note = contact.note.get(&data.locale);
        if !note.trim().is_empty() {
            item = item.child(Text::new().text(note).variant(TextVariant::Caption));
        }
        children.push(Component::Pressable(
            Pressable::new()
                .action(tel_action(&contact.phone))
                .child(item),
        ));
    }

    if show_emergency_banner {
        children.push(Component::Link(
            Link::new()
                .label("i18n:guest.dial112")
                .href("tel:112")
                .action(tel_action("112")),
        ));
    }

    children
}
