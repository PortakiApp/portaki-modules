//! Host dashboard surface — design `emergency-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Card, Field, Form, Page, Stack, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, ContactRow, Localized};

const CONTACT_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let contacts = config.parse_contacts();

    let submit_args = crate::commands::UpdateConfigArgs {
        contacts: contacts_to_submit(&contacts, &lang),
        contacts_json: String::new(),
        host_visible_phone: config.host_visible_phone.clone(),
    };
    let save_action = crate::ids::module_id().command(crate::ids::UPDATE_CONFIG, submit_args);

    let mut cards: Vec<Component> = vec![Card::new()
        .title("i18n:host.section.hostPhone")
        .subtitle("i18n:host.section.hostPhone.help")
        .icon("info-circle")
        .children(vec![Field::new()
            .name("host_visible_phone")
            .label("i18n:host.phone.label")
            .child(
                TextInput::new()
                    .name("host_visible_phone")
                    .value(config.host_visible_phone)
                    .placeholder("i18n:host.phone.placeholder"),
            )
            .into()])
        .into()];

    for index in 0..CONTACT_SLOTS {
        cards.push(contact_card(index, contacts.get(index), &lang));
    }
    cards.push(
        Button::new()
            .label("i18n:host.save")
            .tone(Tone::Primary)
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]),
            ),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn contacts_to_submit(contacts: &[ContactRow], lang: &str) -> Vec<crate::commands::ContactInput> {
    contacts
        .iter()
        .map(|c| crate::commands::ContactInput {
            label: c.label.get(lang).to_string(),
            label_fr: String::new(),
            label_en: String::new(),
            phone: c.phone.clone(),
        })
        .collect()
}

fn contact_card(index: usize, contact: Option<&ContactRow>, lang: &str) -> Component {
    let slot = index + 1;
    let label = contact.map(|c| c.label.get(lang)).unwrap_or("");
    let phone = contact.map(|c| c.phone.as_str()).unwrap_or("");

    Card::new()
        .title(format!("i18n:host.contact.slot{slot}"))
        .icon("users")
        .children(vec![
            Field::new()
                .name(format!("contacts.{index}.label"))
                .label("i18n:host.contact.label")
                .child(
                    TextInput::new()
                        .name(format!("contacts.{index}.label"))
                        .value(label),
                )
                .into(),
            Field::new()
                .name(format!("contacts.{index}.phone"))
                .label("i18n:host.contact.phone")
                .child(
                    TextInput::new()
                        .name(format!("contacts.{index}.phone"))
                        .value(phone),
                )
                .into(),
        ])
        .into()
}
