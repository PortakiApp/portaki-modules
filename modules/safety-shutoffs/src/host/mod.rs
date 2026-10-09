//! Host dashboard surface — l'onglet « Coupures & sécurité » de l'espace du logement.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, ImageUpload, Page, Select, Stack, StepList, Text, TextArea,
    TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::config::{ModuleConfig, ShutoffRow, KINDS, MAX_SHUTOFFS};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::Shield
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // No Save button — the modules drawer owns the footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:host.intro")
                        .variant(TextVariant::Caption)
                        .into(),
                    shutoffs_card(&config, &ctx),
                    note_card(&config, &ctx),
                ]))),
    )
    .with_id(MAIN))
}

/// Les organes, en lignes dynamiques bornées — le motif de `rules` et `ical-sync`.
///
/// Pas de six emplacements figés : la plupart des logements n'ont ni extincteur ni détecteur à
/// déclarer, et six cartes vides font passer un formulaire de deux minutes pour un formulaire de
/// dix (§2.22).
fn shutoffs_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.parse_shutoffs().len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| shutoff_row(index, config, ctx))
        .collect();

    Card::new()
        .title("i18n:host.shutoffs.title")
        .subtitle("i18n:host.shutoffs.subtitle")
        .icon(IconName::Shield)
        .child(
            StepList::new()
                .addLabel("i18n:host.shutoffs.add")
                .removeLabel("i18n:host.shutoffs.remove")
                .emptyTitle("i18n:host.shutoffs.emptyTitle")
                .emptyDescription("i18n:host.shutoffs.emptyDescription")
                .itemKeyPrefix("shutoffs")
                .addAction(emit_input(RowCount {
                    shutoffs_count: (rows_count + 1).min(MAX_SHUTOFFS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("shutoffs_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_SHUTOFFS),
        None => stored.clamp(1, MAX_SHUTOFFS),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    shutoffs_count: usize,
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn shutoff_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let row: Option<&ShutoffRow> = config.shutoffs.get(index);
    // Le champ, avec le message de `problems` sous lui s'il y en a un.
    let named = |key: &str| {
        let name = format!("shutoffs.{index}.{key}");
        let field = Field::new().name(name.clone());
        match config.error_of(&name) {
            Some(error) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
    let kind = row.map(ShutoffRow::kind_key).unwrap_or("other");
    let title = row.map(|r| r.title.host_value(ctx)).unwrap_or_default();
    let location = row.map(|r| r.location.host_value(ctx)).unwrap_or_default();
    let instruction = row
        .map(|r| r.instruction.host_value(ctx))
        .unwrap_or_default();
    let photo = row.map(|r| r.photo.clone()).unwrap_or_default();
    let id = row
        .filter(|r| !r.is_blank())
        .map(|r| sdui::row_id("shutoffs", index, Some(&r.id)));

    let mut children: Vec<Component> = id.into_iter().collect();
    children.push(
        Field::new()
            .name(format!("shutoffs.{index}.kind"))
            .label("i18n:host.kind.label")
            .child(
                Select::new()
                    .name(format!("shutoffs.{index}.kind"))
                    .options(
                        KINDS
                            .iter()
                            .map(|(wire, _)| {
                                ChoiceOption::new(*wire, format!("i18n:host.kind.label.{wire}"))
                            })
                            .collect(),
                    )
                    .value(kind),
            )
            .into(),
    );
    children.push(
        named("title")
            .label("i18n:host.shutoffs.rowTitle")
            .child(
                TextInput::new()
                    .name(format!("shutoffs.{index}.title"))
                    .value(title)
                    .placeholder("i18n:host.shutoffs.rowTitle.placeholder"),
            )
            .into(),
    );
    children.push(
        named("location")
            .label("i18n:host.shutoffs.location")
            .child(
                TextInput::new()
                    .name(format!("shutoffs.{index}.location"))
                    .value(location)
                    .placeholder("i18n:host.shutoffs.location.placeholder"),
            )
            .into(),
    );
    children.push(
        FieldHint::new()
            .text("i18n:host.shutoffs.location.hint")
            .into(),
    );
    children.push(
        named("instruction")
            .label("i18n:host.shutoffs.instruction")
            .child(
                TextArea::new()
                    .name(format!("shutoffs.{index}.instruction"))
                    .value(instruction)
                    .rows(2)
                    .placeholder("i18n:host.shutoffs.instruction.placeholder"),
            )
            .into(),
    );

    children.push(
        Field::new()
            .name(format!("shutoffs.{index}.photo"))
            .label("i18n:host.shutoffs.photo")
            .child(
                ImageUpload::new()
                    .name(format!("shutoffs.{index}.photo"))
                    .value(photo),
            )
            .into(),
    );
    children.push(
        FieldHint::new()
            .text("i18n:host.shutoffs.photo.hint")
            .into(),
    );

    Stack::new()
        .id(format!("shutoff-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

/// La consigne générale : ce que l'hôte veut dire avant la liste.
fn note_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut field = Field::new().name("general_note");
    if let Some(error) = config.error_of("general_note") {
        field = field.error(error.get(&ctx.locale).to_string());
    }
    Card::new()
        .title("i18n:host.note.title")
        .icon(IconName::MessageCircle)
        .child(
            field.label("i18n:host.note.label").child(
                TextArea::new()
                    .name("general_note")
                    .value(config.general_note.host_value(ctx))
                    .rows(3)
                    .placeholder("i18n:host.note.placeholder"),
            ),
        )
        .child(FieldHint::new().text("i18n:host.note.hint"))
        .into()
}
