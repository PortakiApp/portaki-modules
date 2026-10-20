//! Host dashboard surface — les réglages « Coupures et sécurité » (spec Sécurité §2), mode page.
//!
//! Deux `Section`s : « Consigne générale », puis « Emplacements », un tableau ordonnable (l'ordre
//! du livret) dont chaque ligne s'ouvre dans un dialogue (`panel: "dialog"`), ses champs nommés
//! `shutoffs.<i>.<clé>`. Chaque saisie s'enregistre par `updateConfig` ; une erreur s'affiche sous
//! son champ et bloque « Publier » (`publishReadiness`), jamais l'enregistrement.

use portaki_sdk::bundle_text;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    ChoiceList, DataTable, Field, FieldHint, Form, ImageUpload, InlineNotice, Page, Section, Stack,
    TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::config::{ModuleConfig, ShutoffRow, KINDS, KIND_OTHER, MAX_SHUTOFFS, PROPOSED};

/// Les lignes ajoutées pas encore enregistrées : leurs id, séparés par des virgules.
const NEW_SHUTOFFS: &str = "newShutoffs";

/// La consigne générale : 280 caractères (§2.1).
const NOTE_MAX: usize = 280;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::Shield
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let rows = rows(&config, &ctx);

    // No Save button — the dashboard autosaves and owns « Publier ».
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(note_section(&config, &ctx))
                .child(shutoffs_section(&config, &rows, &ctx)),
        ),
    )
    .with_id(MAIN))
}

/// Les lignes du tableau : celles enregistrées — à la première ouverture, les cinq proposées
/// (§2.2) —, puis celles qu'« Ajouter » a ouvertes.
fn rows(config: &ModuleConfig, ctx: &HostContext) -> Vec<ShutoffRow> {
    let mut rows: Vec<ShutoffRow> = match &config.shutoffs {
        Some(rows) => rows.clone(),
        None => PROPOSED
            .iter()
            .map(|kind| ShutoffRow {
                id: (*kind).to_string(),
                ..ShutoffRow::default()
            })
            .collect(),
    };
    for id in ctx.input_str(NEW_SHUTOFFS).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty() && !rows.iter().any(|row| row.id == id) {
            rows.push(ShutoffRow {
                id: id.to_string(),
                ..ShutoffRow::default()
            });
        }
    }
    rows
}

/// §2.1 La consigne générale : le bandeau « Le mot de {hôte} » en tête du détail.
fn note_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let value = config.general_note.host_value(ctx);
    let mut field = Field::new()
        .name("general_note")
        .label("i18n:host.note.label");
    if let Some(error) = config.error_of("general_note") {
        field = field.error(error.get(&ctx.locale).to_string());
    }
    let hint = ctx
        .host
        .as_ref()
        .map(|host| host.name.trim())
        .filter(|name| !name.is_empty())
        .and_then(|name| t!("host.note.hint.named", host = name).ok())
        .unwrap_or_else(|| "i18n:host.note.hint".to_string());
    Section::new()
        .title("i18n:host.note.title")
        .child(
            field
                .child(
                    TextArea::new()
                        .name("general_note")
                        .value(value)
                        .rows(3)
                        .placeholder("i18n:host.note.placeholder"),
                )
                .child(FieldHint::new().text(hint))
                // ponytail: compté au rendu, pas à la frappe — `TextArea` n'a pas de compteur.
                .child(FieldHint::new().text(format!("{} / {NOTE_MAX}", value.chars().count()))),
        )
        .into()
}

/// §2.2 Les emplacements : le tableau ordonnable et le dialogue de chaque ligne.
fn shutoffs_section(config: &ModuleConfig, rows: &[ShutoffRow], ctx: &HostContext) -> Component {
    let mut children: Vec<Component> =
        vec![FieldHint::new().text("i18n:host.shutoffs.hint").into()];

    // « Ajoutez au moins un emplacement. » tant qu'aucune ligne n'a d'emplacement ; « 20 au
    // maximum » vient de `problems`.
    let error = config.error_of("shutoffs").or_else(|| {
        (!rows.iter().any(ShutoffRow::is_complete)).then(|| bundle_text!("host.shutoffs.required"))
    });
    if let Some(error) = error {
        children.push(
            InlineNotice::new()
                .tone(Tone::Danger)
                .message(error.get(&ctx.locale))
                .into(),
        );
    }

    let mut table = DataTable::new()
        .rowName("shutoffs")
        .sortable(true)
        .panel("dialog")
        .columns(json!([
            { "key": "kind", "label": "i18n:host.kind.label", "kind": "pill" },
            { "key": "title", "label": "i18n:host.shutoffs.rowTitle", "sub": "location" },
            { "key": "status", "label": "i18n:host.column.status", "kind": "pill" },
            { "key": "photo", "label": "i18n:host.shutoffs.photo.pill", "kind": "pill" },
        ]))
        .rows(Value::Array(
            rows.iter().map(|shutoff| row(shutoff, ctx)).collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, shutoff)| shutoff_panel(index, shutoff, config, ctx))
                .collect(),
        );
    if rows.len() < MAX_SHUTOFFS {
        let mut added: Vec<String> = rows
            .iter()
            .skip(config.rows().len())
            .filter(|row| !PROPOSED.contains(&row.id.as_str()))
            .map(|row| row.id.clone())
            .collect();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_SHUTOFFS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.shutoffs.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());

    // §3 : les lignes sans emplacement sont retirées à la publication — on le dit, sans bloquer.
    let empty = rows.iter().filter(|row| !row.is_complete()).count();
    if empty > 0 {
        children.push(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message(crate::queries::empty_rows(empty).get(&ctx.locale))
                .into(),
        );
    }

    Section::new()
        .title("i18n:host.shutoffs.title")
        .subtitle(format!("{} / {MAX_SHUTOFFS}", rows.len()))
        .children(children)
        .into()
}

/// Une ligne du tableau : type, nom (l'emplacement dessous), « À compléter », « Photo ».
fn row(shutoff: &ShutoffRow, ctx: &HostContext) -> Value {
    let location = shutoff.location.host_value(ctx);
    json!({
        "id": shutoff.id,
        "kind": if shutoff.kind_raw().is_empty() {
            String::new()
        } else {
            format!("i18n:host.kind.label.{}", shutoff.kind_key())
        },
        "title": shutoff.title_for(&ctx.locale),
        "location": if location.trim().is_empty() {
            "i18n:host.shutoffs.location.proposed".to_string()
        } else {
            location.to_string()
        },
        "status": if shutoff.is_complete() { "" } else { "i18n:host.shutoffs.todo" },
        "photo": if shutoff.photo_ref().is_some() { "i18n:host.shutoffs.photo.pill" } else { "" },
    })
}

/// Le dialogue d'une ligne : Type, Nom, Où, Comment couper, Photo (§2.2).
fn shutoff_panel(
    index: usize,
    shutoff: &ShutoffRow,
    config: &ModuleConfig,
    ctx: &HostContext,
) -> Component {
    let name = |key: &str| format!("shutoffs.{index}.{key}");
    let field = |key: &str, label: &str| {
        let field = Field::new().name(name(key)).label(label);
        match config.error_of(&name(key)) {
            Some(error) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
    // Le nom suit le type : c'est lui que le voyageur lit quand le champ reste vide (§4).
    let kind = shutoff.kind_key();
    let title_placeholder = if shutoff.kind_raw().is_empty() || kind == KIND_OTHER {
        "i18n:host.shutoffs.rowTitle.placeholder".to_string()
    } else {
        format!("i18n:host.kind.label.{kind}")
    };

    Stack::new()
        .gap(14.0)
        .children(vec![
            field("kind", "i18n:host.kind.label")
                .child(
                    ChoiceList::new()
                        .name(name("kind"))
                        .layout(ChoiceListLayout::Cards)
                        .value(shutoff.kind_raw())
                        .choices(
                            KINDS
                                .iter()
                                .map(|(wire, icon)| {
                                    ChoiceOption::new(*wire, format!("i18n:host.kind.label.{wire}"))
                                        .icon(*icon)
                                })
                                .collect(),
                        ),
                )
                .into(),
            field("title", "i18n:host.shutoffs.rowTitle")
                .child(
                    TextInput::new()
                        .name(name("title"))
                        .value(shutoff.title.host_value(ctx))
                        .placeholder(title_placeholder),
                )
                .into(),
            field("location", "i18n:host.shutoffs.location")
                .child(
                    TextInput::new()
                        .name(name("location"))
                        .value(shutoff.location.host_value(ctx))
                        .placeholder("i18n:host.shutoffs.location.placeholder"),
                )
                .into(),
            field("instruction", "i18n:host.shutoffs.instruction")
                .child(
                    TextArea::new()
                        .name(name("instruction"))
                        .value(shutoff.instruction.host_value(ctx))
                        .rows(2)
                        .placeholder("i18n:host.shutoffs.instruction.placeholder"),
                )
                .into(),
            field("photo", "i18n:host.shutoffs.photo")
                .child(
                    ImageUpload::new()
                        .name(name("photo"))
                        .value(shutoff.photo.as_str()),
                )
                .child(FieldHint::new().text("i18n:host.shutoffs.photo.hint"))
                .into(),
        ])
        .into()
}
