//! Shared guest SDUI body for emergency contacts.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Leading, LeadingVisual, ListItemLayout, Trailing};
use portaki_sdk::sdui::primitives::{Grid, InfoBanner, ListItem, Text};

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
        // L'action sur la ligne, pas autour : c'est ce que la maquette donne à chaque rangée, et
        // un `Pressable` enveloppant ajoutait un niveau que rien ne lit.
        children.push(Component::ListItem(
            ListItem::new()
                .title("i18n:guest.host.label")
                .subtitle(data.host_phone.clone())
                .trailing(Trailing::Text("i18n:guest.call".into()))
                .action(tel_action(&data.host_phone)),
        ));
    }

    for contact in &data.contacts {
        let label = contact.label.get(&data.locale);
        let mut item = ListItem::new()
            .title(label)
            .subtitle(contact.phone.clone())
            .trailing(Trailing::Text("i18n:guest.call".into()));
        if let Some(cat) = contact.category.as_deref().filter(|c| !c.trim().is_empty()) {
            item = item.leading(Leading::Icon(cat.into()));
        }
        let note = contact.note.get(&data.locale);
        if !note.trim().is_empty() {
            item = item.child(Text::new().text(note).variant(TextVariant::Caption));
        }
        children.push(Component::ListItem(item.action(tel_action(&contact.phone))));
    }

    // La phrase utile ferme la carte : on la lit quand on n'a pas trouvé son numéro au-dessus.
    if !data.useful_line.is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(data.useful_line.clone())
                .variant(TextVariant::Caption),
        ));
    }

    children
}

/// Les tuiles des numéros du pays, en tête de carte (§2.16).
///
/// Elles passent avant les contacts de l'hôte : en urgence, on compose avant de lire. Le premier
/// porte le ton danger — c'est celui qu'on fait sans savoir lequel faire.
pub fn country_numbers(locale: &str) -> Component {
    let tiles: Vec<Component> = crate::numbers::for_locale(locale)
        .iter()
        .map(|number| {
            let mut tile = ListItem::new()
                .layout(ListItemLayout::Tile)
                .leading(Leading::Visual(Box::new(LeadingVisual {
                    icon: Some(IconName::Phone),
                    ..LeadingVisual::default()
                })))
                .title(number.number)
                .subtitle(format!("i18n:{}", number.label_key));
            if number.primary {
                // La maquette teinte l'icône seule ; `LeadingVisual` ne porte pas de ton, donc
                // c'est la tuile entière qui le prend. Plus appuyé que le dessin, jamais faux.
                tile = tile.tone(Tone::Danger);
            }
            Component::ListItem(tile.action(tel_action(number.number)))
        })
        .collect();

    Component::Grid(Grid::new().minColumnWidth(90.0).plain(true).children(tiles))
}
