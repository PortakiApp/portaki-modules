//! Shared guest SDUI body for emergency contacts.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Leading, LeadingVisual, ListItemLayout, Trailing};
use portaki_sdk::sdui::primitives::{Grid, InfoBanner, ListItem, Text};

use super::load::GuestData;

/// Le numéro comme on le lit : « 06 12 34 56 78 » pour un français en français, « +33 6 12 34 56
/// 78 » sinon. Le lien `tel:` garde la forme composable ([`tel_action`]).
///
/// ponytail: la France seulement, la seule que le module connaisse (`numbers.rs`). Un autre
/// indicatif reste tel que l'hôte l'a saisi — il l'a souvent déjà espacé.
fn display_phone(phone: &str, locale: &str) -> String {
    let compact = crate::config::compact(phone);
    let Some(national) = compact
        .strip_prefix("+33")
        .filter(|rest| rest.len() == 9 && rest.bytes().all(|b| b.is_ascii_digit()))
    else {
        return phone.trim().to_string();
    };
    let pairs = national.as_bytes()[1..]
        .chunks(2)
        .map(|pair| std::str::from_utf8(pair).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    if locale.to_ascii_lowercase().starts_with("fr") {
        format!("0{} {pairs}", &national[..1])
    } else {
        format!("+33 {} {pairs}", &national[..1])
    }
}

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
        let mut host = ListItem::new()
            .title("i18n:guest.host.label")
            .subtitle(display_phone(&data.host_phone, &data.locale))
            // L'hôte mène par son pictogramme, comme chaque rangée de la maquette : c'est la
            // ligne qu'on cherche en premier, et c'était la seule sans repère.
            .leading(Leading::Icon("users".into()))
            .trailing(Trailing::Text("i18n:guest.call".into()))
            .action(tel_action(&data.host_phone));
        // « Joignable 08:00 – 21:00 », et ce qu'on fait en dehors (§2.2).
        if let Some((from, to)) = &data.host_hours {
            let reachable = t!("guest.host.hours", from = from.clone(), to = to.clone())
                .unwrap_or_else(|_| format!("{from} – {to}"));
            host = host
                .child(Text::new().text(reachable).variant(TextVariant::Caption))
                .child(
                    Text::new()
                        .text("i18n:guest.host.outOfHours")
                        .variant(TextVariant::Caption),
                );
        }
        children.push(Component::ListItem(host));
    }

    for contact in &data.contacts {
        let label = contact.label.get(&data.locale);
        let mut item = ListItem::new()
            .title(label)
            .subtitle(display_phone(&contact.phone, &data.locale))
            .trailing(Trailing::Text("i18n:guest.call".into()));
        // Sans catégorie, le combiné : une liste où une rangée sur deux porte un repère se lit
        // de travers, et un numéro de téléphone sous une icône de téléphone ne dit rien de faux.
        let icon = contact
            .category
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .unwrap_or("phone");
        item = item.leading(Leading::Icon(icon.into()));
        let note = contact.note.get(&data.locale);
        if !note.trim().is_empty() {
            item = item.child(Text::new().text(note).variant(TextVariant::Caption));
        }
        children.push(Component::ListItem(item.action(tel_action(&contact.phone))));
    }

    // Pharmacie, hôpital, médecin : une rangée qu'on appelle, comme les contacts.
    for (key, name, phone, distance) in &data.health {
        let title = if name.is_empty() {
            format!("i18n:{key}")
        } else {
            name.clone()
        };
        let mut item = ListItem::new()
            .title(title)
            .subtitle(match distance {
                Some(distance) => format!("{} · {distance}", display_phone(phone, &data.locale)),
                None => display_phone(phone, &data.locale),
            })
            .leading(Leading::Icon("heart-handshake".into()))
            .trailing(Trailing::Text("i18n:guest.call".into()));
        // « Ouverte jusqu'à 19:30 » / « Fermée » sous la pharmacie (ses horaires, §2.3).
        if let (true, Some(state)) = (*key == "guest.health.pharmacy", &data.pharmacy_state) {
            item = item.child(
                Text::new()
                    .text(state.clone())
                    .variant(TextVariant::Caption),
            );
        }
        children.push(Component::ListItem(item.action(tel_action(phone))));
    }

    // La pharmacie de garde du pays, toujours là où elle existe (§2.3, §9 cas 4).
    if let Some(number) = data.on_duty_pharmacy {
        children.push(Component::ListItem(
            ListItem::new()
                .title("i18n:guest.health.onDuty")
                .subtitle(number)
                .leading(Leading::Icon("heart-handshake".into()))
                .trailing(Trailing::Text("i18n:guest.call".into()))
                .action(tel_action(number)),
        ));
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
pub fn country_numbers(country: Option<&str>) -> Component {
    let tiles: Vec<Component> = crate::numbers::for_country(country)
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
            // Le 114 se joint par SMS : un appel vocal n'y aboutit pas.
            let action = if number.sms {
                Action::external(format!("sms:{}", number.number))
            } else {
                tel_action(number.number)
            };
            Component::ListItem(tile.action(action))
        })
        .collect();

    Component::Grid(Grid::new().minColumnWidth(90.0).plain(true).children(tiles))
}

#[cfg(test)]
mod tests {
    use super::display_phone;

    #[test]
    fn a_french_number_reads_in_pairs() {
        assert_eq!(display_phone("+33612345678", "fr-FR"), "06 12 34 56 78");
        assert_eq!(display_phone("+33 6 12 34 56 78", "fr"), "06 12 34 56 78");
        assert_eq!(display_phone("+33612345678", "en-US"), "+33 6 12 34 56 78");
        assert_eq!(display_phone("+33493123456", "de-DE"), "+33 4 93 12 34 56");
        // Ni français ni court : tel que saisi.
        assert_eq!(
            display_phone(" +44 20 7946 0958 ", "fr-FR"),
            "+44 20 7946 0958"
        );
        assert_eq!(display_phone("3237", "fr-FR"), "3237");
    }
}
