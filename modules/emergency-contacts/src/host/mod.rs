//! Host dashboard surface — design `emergency-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Stack, StepList, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ContactRow, ModuleConfig};

/// Combien de contacts le formulaire accepte.
///
/// Une capacité, pas un nombre de lignes dessinées : six emplacements figés gelaient la liste à
/// six — l'hôte ne pouvait pas en saisir un septième parce que le formulaire ne le dessinait
/// jamais, et voyait quatre cartes vides quand il en avait saisi deux.
pub const MAX_CONTACTS: usize = 12;

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

    let mut cards: Vec<Component> = vec![Card::new()
        .title("i18n:host.section.hostPhone")
        .subtitle("i18n:host.section.hostPhone.help")
        .icon(IconName::InfoCircle)
        .children(vec![Field::new()
            .name("host_visible_phone")
            .label("i18n:host.phone.label")
            .child(
                TextInput::new()
                    .name("host_visible_phone")
                    .value(config.host_visible_phone.clone())
                    .placeholder("i18n:host.phone.placeholder"),
            )
            .into()])
        .into()];

    cards.push(contacts_card(&config, &ctx));

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

/// Les contacts de l'hôte, en lignes dynamiques bornées.
fn contacts_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.contacts.len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| contact_row(index, config.contacts.get(index), ctx))
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

fn contact_row(index: usize, contact: Option<&ContactRow>, ctx: &HostContext) -> Component {
    let label = contact.map(|c| c.label.host_value(ctx)).unwrap_or_default();
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
                .collect(),
        )
        .into()
}
