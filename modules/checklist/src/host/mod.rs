//! Host surfaces — the checklist editor (`main`) and the two stats details.
//!
//! Editor: « Vos checklists » on the left (horizontal `Stack` = master-detail, stacked when
//! narrow), the selected list on the right. The workspace header Save sends the form to
//! `updateConfig`; a new list is created from a template by `createChecklist`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Button, Card, EditableList, Eyebrow, Field, FieldHint, Form, Grid, Page, Pill, Select,
    SelectableCard, Stack, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::EditableListItem;

use crate::commands::{CreateChecklistArgs, DeleteChecklistArgs};
use crate::entities::{Checklist, ChecklistItem};
use crate::labels;
use crate::lists::{self, TEMPLATES};
use crate::storage;

mod stats;

pub use stats::{render_stats_checklist, render_stats_cleaning, stats_summary};

const SELECT_NEW: &str = "__new__";

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let fr = labels::lang_code(&ctx.locale) == "fr";
    let checklists = storage::list_checklists().unwrap_or_default();
    let items = storage::list_items().unwrap_or_default();
    let selected = ctx
        .input_str("selectedId")
        .map(str::to_string)
        .or_else(|| checklists.first().map(|list| list.id.to_string()))
        .unwrap_or_else(|| SELECT_NEW.to_string());

    let mut column: Vec<Component> = vec![Eyebrow::new().text("i18n:host.lists.title").into()];
    for list in &checklists {
        let count = items_of(&items, list).len();
        let meta =
            t!(&format!("host.lists.meta.{}", list.audience), count = count).unwrap_or_default();
        column.push(
            SelectableCard::new()
                .value(list.id.to_string())
                .label(name(list, fr))
                .description(meta)
                .icon(list.icon.clone())
                .selected(list.id.to_string() == selected)
                .action(select(&list.id.to_string()))
                .into(),
        );
    }
    column.push(
        Button::new()
            .label("i18n:host.lists.new")
            .variant(ButtonVariant::Outline)
            .action(select(SELECT_NEW))
            .into(),
    );

    let panel = match checklists
        .iter()
        .find(|list| list.id.to_string() == selected)
    {
        Some(list) => edit_panel(list, &items_of(&items, list), fr),
        None => new_panel(),
    };

    Surface::new(
        Page::new().child(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(18.0)
                .children(vec![Stack::new().gap(8.0).children(column).into(), panel]),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}

#[portaki_sdk::wire(serialize)]
struct SurfaceInputSelectedId<'a> {
    selected_id: &'a str,
}

fn select(selected_id: &str) -> Action {
    Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(json_value(SurfaceInputSelectedId { selected_id })),
    )
}

fn items_of<'a>(items: &'a [ChecklistItem], list: &Checklist) -> Vec<&'a ChecklistItem> {
    items
        .iter()
        .filter(|item| item.checklist_id == list.id)
        .collect()
}

fn name(list: &Checklist, fr: bool) -> String {
    if fr { &list.name_fr } else { &list.name_en }.clone()
}

fn select_field(label: &str, name: &str, values: &[&str], value: &str) -> Component {
    let options = values
        .iter()
        .map(|v| ChoiceOption::new(*v, format!("i18n:host.{name}.{v}")))
        .collect();
    Field::new()
        .label(label)
        .name(name)
        .child(Select::new().name(name).options(options).value(value))
        .into()
}

fn edit_panel(list: &Checklist, items: &[&ChecklistItem], fr: bool) -> Component {
    let host = list.audience == lists::HOST;
    let settings: Vec<Component> = if host {
        let assignee = match (&list.assignee_name, &list.assignee_role) {
            (Some(name), Some(role)) => format!("{name} · {role}"),
            (Some(name), None) => name.clone(),
            _ => String::new(),
        };
        vec![
            select_field(
                "i18n:host.field.hostTrigger",
                "trigger",
                lists::HOST_TRIGGERS,
                &list.trigger,
            ),
            Field::new()
                .label("i18n:host.field.assignee")
                .name("assignee")
                .child(
                    TextInput::new()
                        .name("assignee")
                        .value(assignee)
                        .placeholder("i18n:host.field.assignee.placeholder"),
                )
                .into(),
            select_field(
                "i18n:host.field.deadline",
                "deadline",
                lists::DEADLINES,
                list.deadline.as_deref().unwrap_or(lists::NEXT_ARRIVAL),
            ),
        ]
    } else {
        vec![
            select_field(
                "i18n:host.field.guestTrigger",
                "trigger",
                lists::GUEST_TRIGGERS,
                &list.trigger,
            ),
            select_field(
                "i18n:host.field.placement",
                "placement",
                lists::PLACEMENTS,
                &list.placement,
            ),
        ]
    };

    let rows = items
        .iter()
        .map(|item| {
            let label = labels::i18n_label(item);
            EditableListItem {
                id: Some(item.id.to_string()),
                label: if host && !fr {
                    label.en.clone()
                } else {
                    label.fr
                },
                label_en: (!host).then_some(label.en),
                photo: host.then_some(item.photo_required),
                checked: None,
            }
        })
        .collect();

    let audience = if host { "host" } else { "guest" };
    let mut form: Vec<Component> = vec![
        TextInput::new()
            .name("id")
            .value(list.id.to_string())
            .into(),
        TextInput::new().name("name").value(name(list, fr)).into(),
        Pill::new()
            .label(format!("i18n:host.audience.{audience}"))
            .tone(if host { Tone::Warning } else { Tone::Info })
            .into(),
        Grid::new()
            .minColumnWidth(200.0)
            .gap(12.0)
            .children(settings)
            .into(),
        Field::new()
            .label("i18n:host.tasks.title")
            .name("items")
            .child(FieldHint::new().text(format!("i18n:host.tasks.hint.{audience}")))
            .child(
                EditableList::new()
                    .name("items")
                    .items(rows)
                    .bilingual(!host)
                    .photoToggle(host)
                    .checkbox(true)
                    .addLabel("i18n:host.tasks.add"),
            )
            .into(),
    ];
    if host {
        form.push(
            ToggleRow::new()
                .name("notify_assignee")
                .label("i18n:host.toggle.notifyAssignee")
                .description("i18n:host.toggle.notifyAssignee.desc")
                .icon("bell")
                .checked(list.notify_assignee)
                .into(),
        );
        form.push(
            ToggleRow::new()
                .name("alert_host")
                .label("i18n:host.toggle.alertHost")
                .description("i18n:host.toggle.alertHost.desc")
                .icon("danger-triangle")
                .checked(list.alert_host)
                .into(),
        );
    }
    form.push(
        Text::new()
            .text(format!("i18n:host.where.{audience}"))
            .variant(TextVariant::Caption)
            .into(),
    );
    form.push(
        Button::new()
            .label("i18n:host.lists.delete")
            .variant(ButtonVariant::Ghost)
            .tone(Tone::Danger)
            .action(crate::ids::module_id().command(
                crate::ids::DELETE_CHECKLIST,
                DeleteChecklistArgs { id: list.id },
            ))
            .into(),
    );

    Card::new()
        .icon(list.icon.clone())
        .child(Form::new().child(Stack::new().gap(16.0).children(form)))
        .into()
}

/// « Nouvelle checklist » — one card per template, each creating its list.
fn new_panel() -> Component {
    let cards = TEMPLATES
        .iter()
        .map(|template| {
            SelectableCard::new()
                .value(template.id)
                .label(format!("i18n:template.{}.name", template.id))
                .description(format!("i18n:template.{}.desc", template.id))
                .icon(template.icon)
                .action(crate::ids::module_id().command(
                    crate::ids::CREATE_CHECKLIST,
                    CreateChecklistArgs {
                        template: template.id.to_string(),
                    },
                ))
                .into()
        })
        .collect();
    Card::new()
        .title("i18n:host.new.title")
        .subtitle("i18n:host.new.subtitle")
        .icon("plus")
        .child(Grid::new().minColumnWidth(220.0).gap(10.0).children(cards))
        .into()
}
