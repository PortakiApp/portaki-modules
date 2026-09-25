//! Host dashboard surface — design `emergency-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Stack, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ContactRow, Localized, ModuleConfig};

const CONTACT_SLOTS: usize = 6;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::EmergencyEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Phone
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let lang = Localized::lang_code(&ctx.locale);
    let config = ModuleConfig::read(&ctx)?;
    let contacts = config.parse_contacts();

    let mut cards: Vec<Component> = vec![Card::new()
        .title("i18n:host.section.hostPhone")
        .subtitle("i18n:host.section.hostPhone.help")
        .icon(IconName::InfoCircle)
        .children(vec![Field::new()
            .name("host_visible_phone")
            .label("i18n:host.phone.label")
            .required(true)
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

    // No Save button — the modules drawer owns the footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(crate::ids::HOST_MAIN))
}

fn contact_card(index: usize, contact: Option<&ContactRow>, lang: &str) -> Component {
    let slot = index + 1;
    let label = contact.map(|c| c.label.pick(lang)).unwrap_or_default();
    let phone = contact.map(|c| c.phone.as_str()).unwrap_or("");

    Card::new()
        .title(format!("i18n:host.contact.slot{slot}"))
        .icon(IconName::Users)
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
