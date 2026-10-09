//! Host dashboard surface — design `emergency-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, FieldHint, Form, Page, Select, Stack, StepList, Text, TextInput,
    Toggle,
};
use portaki_sdk::sdui::surface::Surface;

pub use crate::config::MAX_CONTACTS;
use crate::config::{ModuleConfig, AVAILABILITY, DEFAULT_HOURS};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::EmergencyEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Phone
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // L'ordre de la spec (§2) : les numéros du pays, vos contacts, la santé.
    let mut cards: Vec<Component> = vec![Card::new()
        .title("i18n:host.section.country")
        .icon(IconName::Phone)
        .child(FieldHint::new().text("i18n:host.section.country.help"))
        .into()];
    cards.push(host_card(&config, &ctx));
    cards.push(contacts_card(&config, &ctx));
    cards.push(useful_card(&config, &ctx));

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
    .with_id(MAIN))
}

/// §2.2 L'hôte : son numéro affiché ou non, et quand on peut l'appeler.
fn host_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![
        Field::new()
            .name("show_host")
            .label("i18n:host.showHost.label")
            .child(Toggle::new().name("show_host").checked(config.show_host()))
            .into(),
        FieldHint::new().text("i18n:host.showHost.hint").into(),
    ];
    // Masqué, le numéro ne demande ni disponibilité ni surcharge (règles communes).
    if config.show_host() {
        let availability = if config.host_hours().is_some() {
            "hours"
        } else {
            "always"
        };
        children.push(
            Field::new()
                .name("host_availability")
                .label("i18n:host.availability.label")
                .child(
                    Select::new()
                        .name("host_availability")
                        .options(
                            AVAILABILITY
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(
                                        *key,
                                        format!("i18n:host.availability.label.{key}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(availability),
                )
                .into(),
        );
        if let Some((from, to)) = config.host_hours() {
            for (name, label, value, placeholder) in [
                (
                    "host_hours_from",
                    "i18n:host.hours.from",
                    from,
                    DEFAULT_HOURS.0,
                ),
                ("host_hours_to", "i18n:host.hours.to", to, DEFAULT_HOURS.1),
            ] {
                children.push(
                    named(config, ctx, name)
                        .label(label)
                        .child(
                            TextInput::new()
                                .name(name)
                                .value(value)
                                .placeholder(placeholder),
                        )
                        .into(),
                );
            }
        }
        children.push(
            named(config, ctx, "host_visible_phone")
                .label("i18n:host.phone.label")
                .child(
                    TextInput::new()
                        .name("host_visible_phone")
                        .value(config.host_visible_phone.clone())
                        .placeholder("i18n:host.phone.placeholder"),
                )
                .into(),
        );
        children.push(
            FieldHint::new()
                .text("i18n:host.section.hostPhone.help")
                .into(),
        );
    }
    Card::new()
        .title("i18n:host.section.hostPhone")
        .icon(IconName::Users)
        .children(children)
        .into()
}

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: impl Into<String>) -> Field {
    let name = name.into();
    let field = Field::new().name(name.clone());
    match config.error_of(&name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// Les contacts de l'hôte, en lignes dynamiques bornées.
fn contacts_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.contacts.len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| contact_row(index, config, ctx))
        .collect();

    Card::new()
        .title("i18n:host.contacts.title")
        .subtitle("i18n:host.contacts.subtitle")
        .icon(IconName::Users)
        .child(
            StepList::new()
                .addLabel("i18n:host.contacts.add")
                .removeLabel("i18n:host.contacts.remove")
                .emptyTitle("i18n:host.contacts.emptyTitle")
                .emptyDescription("i18n:host.contacts.emptyDescription")
                .itemKeyPrefix("contacts")
                .addAction(emit_input(RowCount {
                    contacts_count: (rows_count + 1).min(MAX_CONTACTS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("contacts_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_CONTACTS),
        None => stored.clamp(1, MAX_CONTACTS),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    contacts_count: usize,
}

fn emit_input(payload: impl serde::Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn contact_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let contact = config.contacts.get(index);
    let label = contact.map(|c| c.label.host_value(ctx)).unwrap_or_default();
    let note = contact.map(|c| c.note.host_value(ctx)).unwrap_or_default();
    let phone = contact.map(|c| c.phone.as_str()).unwrap_or("");
    // A filled row sends its id, so a save merges into it (and keeps its note, its category, its
    // other languages). A blank slot has nothing to keep — and an id would make it count as filled.
    let id = contact
        .filter(|c| !c.is_blank())
        .map(|c| sdui::row_id("contacts", index, Some(&c.id)));

    Stack::new()
        .id(format!("contact-{index}"))
        .gap(10.0)
        .children(
            id.into_iter()
                .chain([
                    named(config, ctx, format!("contacts.{index}.label"))
                        .label("i18n:host.contact.label")
                        .child(
                            TextInput::new()
                                .name(format!("contacts.{index}.label"))
                                .value(label)
                                .placeholder("i18n:host.contact.label.placeholder"),
                        )
                        .into(),
                    named(config, ctx, format!("contacts.{index}.phone"))
                        .label("i18n:host.contact.phone")
                        .child(
                            TextInput::new()
                                .name(format!("contacts.{index}.phone"))
                                .value(phone)
                                .placeholder("+33 6 12 34 56 78"),
                        )
                        .into(),
                    named(config, ctx, format!("contacts.{index}.note"))
                        .label("i18n:host.contact.note")
                        .child(
                            TextInput::new()
                                .name(format!("contacts.{index}.note"))
                                .value(note)
                                .placeholder("i18n:host.contact.note.placeholder"),
                        )
                        .into(),
                ])
                .collect(),
        )
        .into()
}

/// §2.3 Santé : la pharmacie, l'hôpital et le médecin — un nom et un numéro chacun.
fn useful_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = Vec::new();
    for (name_key, name, phone_key, phone) in [
        (
            "pharmacy",
            &config.pharmacy,
            "pharmacy_phone",
            &config.pharmacy_phone,
        ),
        (
            "hospital",
            &config.hospital,
            "hospital_phone",
            &config.hospital_phone,
        ),
        (
            "doctor",
            &config.doctor,
            "doctor_phone",
            &config.doctor_phone,
        ),
    ] {
        let camel = |key: &str| key.replace("_phone", "Phone");
        children.push(
            Field::new()
                .name(name_key)
                .label(format!("i18n:host.{name_key}.label"))
                .child(
                    TextInput::new()
                        .name(name_key)
                        .value(name.clone())
                        .placeholder(format!("i18n:host.{name_key}.placeholder")),
                )
                .into(),
        );
        children.push(
            named(config, ctx, phone_key)
                .label(format!("i18n:host.{}.label", camel(phone_key)))
                .child(TextInput::new().name(phone_key).value(phone.clone()))
                .into(),
        );
        // La pharmacie et l'hôpital se placent sur la carte : un repère sur la Carte du livret.
        if let Some((address, lat, lng)) = match name_key {
            "pharmacy" => Some((
                &config.pharmacy_address,
                config.pharmacy_lat,
                config.pharmacy_lng,
            )),
            "hospital" => Some((
                &config.hospital_address,
                config.hospital_lat,
                config.hospital_lng,
            )),
            _ => None,
        } {
            let mut picker = AddressMapPicker::new()
                .addressName(format!("{name_key}_address"))
                .latName(format!("{name_key}_lat"))
                .lngName(format!("{name_key}_lng"))
                .address(address.clone())
                .label(format!("i18n:host.{name_key}.position"));
            if let (Some(lat), Some(lng)) = (lat, lng) {
                picker = picker.lat(lat).lng(lng);
            }
            children.push(picker.into());
        }
    }
    Card::new()
        .title("i18n:host.section.useful")
        .subtitle("i18n:host.section.useful.help")
        .icon(IconName::InfoCircle)
        .children(children)
        .into()
}
