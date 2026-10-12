//! Host surfaces — the checklist editor (`main`) and the two stats details.
//!
//! Editor (validated board « Checklists ») : the property's lists on the left, grouped
//! « Voyageur » / « Hôte et équipe » (horizontal `Stack` = master-detail, stacked when narrow), the
//! selected list on the right — « Affichage » for a guest list or « Tâche » for a team list, then
//! « Étapes », a sortable table (drag = booklet order) whose rows open in a dialog. « Nouvelle
//! checklist » opens the templates in a `Modal`. Every edit autosaves through `updateConfig`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, DataTable, Eyebrow, Field, FieldHint, Form, Grid, InlineNotice, Modal,
    NumberInput, Page, Pill, Section, Select, SelectableCard, Stack, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::commands::{CreateChecklistArgs, DeleteChecklistArgs};
use crate::entities::{Checklist, ChecklistItem};
use crate::labels;
use crate::lists::{self, TEMPLATES};
use crate::queries::{looks_like_code, step_problem, MAX_STEPS};
use crate::storage;

mod stats;
mod stay;

pub use stats::{render_stats_checklist, render_stats_cleaning, stats_summary};
pub use stay::render_host_stay;

/// Les étapes ajoutées pas encore enregistrées : `<listId>:<stepId>`, séparés par des virgules.
const NEW_STEPS: &str = "newSteps";

/// « Créer la tâche » et « À terminer avant » dans l'ordre de la maquette. Les constantes de
/// [`lists`] gardent le leur : `pick` retombe sur leur premier élément.
const HOST_TRIGGERS: &[&str] = &[
    lists::AFTER_EACH_DEPARTURE,
    lists::BEFORE_EACH_ARRIVAL,
    lists::AT_DEPARTURE_BEFORE_CLEANING,
    lists::ONLY_IF_NEXT_ARRIVAL,
];
const DEADLINES: &[&str] = &[
    lists::NEXT_ARRIVAL_MINUS_2H,
    lists::NEXT_ARRIVAL,
    lists::DEPARTURE_EVENING,
    lists::ARRIVAL,
];
/// « Afficher » : trois moments ; `beforeArrival`, hors maquette, seulement pour une liste qui l'a.
const GUEST_TRIGGERS: &[&str] = &[
    lists::DURING_STAY,
    lists::AT_DEPARTURE,
    lists::DEPARTURE_DAY,
];

#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Sparkles
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::ChecklistEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::CheckCircle
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = labels::lang_code(&ctx.locale);
    let checklists = storage::list_checklists().unwrap_or_default();
    // Blank steps included: the host must see the step « Écrivez l'étape. » is about.
    let items = storage::all_items().unwrap_or_default();
    let in_booklet: Vec<Uuid> = storage::booklet_lists()
        .unwrap_or_default()
        .iter()
        .map(|list| list.id)
        .collect();
    let wanted = ctx.input_str("selectedId");
    let found = wanted.and_then(|id| checklists.iter().find(|list| list.id.to_string() == id));
    // « Nouvelle checklist » sends an id no list has yet: the templates open, and the list created
    // with that id is the one selected on the next render.
    let new_id = match (found, wanted.and_then(|id| Uuid::parse_str(id).ok())) {
        (None, Some(id)) => Some(id),
        (None, None) if checklists.is_empty() => Some(Uuid::new_v4()),
        _ => None,
    };
    let shown = found.or_else(|| checklists.first());

    let mut column: Vec<Component> = Vec::new();
    for audience in [lists::GUEST, lists::HOST] {
        let group: Vec<&Checklist> = checklists
            .iter()
            .filter(|list| list.audience == audience)
            .collect();
        if group.is_empty() {
            continue;
        }
        column.push(
            Eyebrow::new()
                .text(format!("i18n:host.audience.{audience}"))
                .into(),
        );
        for list in group {
            let count = items_of(&items, list).len();
            let key = if in_booklet.contains(&list.id) {
                "host.lists.meta.live"
            } else {
                "host.lists.meta.steps"
            };
            column.push(
                SelectableCard::new()
                    .value(list.id.to_string())
                    .label(name(list, lang == "fr"))
                    .description(t!(key, count = count).unwrap_or_default())
                    .icon(stored_icon(&list.icon))
                    .selected(shown.is_some_and(|s| s.id == list.id))
                    .action(select(&list.id.to_string()))
                    .into(),
            );
        }
    }
    column.push(
        Button::new()
            .label("i18n:host.lists.new")
            .variant(ButtonVariant::Outline)
            .icon(IconName::Plus)
            .action(select(&Uuid::new_v4().to_string()))
            .into(),
    );

    let new_steps: Vec<Uuid> = ctx
        .input_str(NEW_STEPS)
        .unwrap_or_default()
        .split(',')
        .filter_map(|entry| entry.trim().split_once(':'))
        .filter(|(list, _)| shown.is_some_and(|s| s.id.to_string() == *list))
        .filter_map(|(_, step)| Uuid::parse_str(step).ok())
        .collect();
    let mut right: Vec<Component> = match shown {
        Some(list) => edit_panel(list, &items_of(&items, list), &new_steps, &lang),
        None => Vec::new(),
    };
    match (new_id, shown) {
        // Rien à éditer derrière : les modèles en place, sans fenêtre.
        (Some(id), None) => right.push(
            Section::new()
                .title("i18n:host.new.title")
                .child(templates(id))
                .child(FieldHint::new().text("i18n:host.new.hint"))
                .into(),
        ),
        (Some(id), Some(list)) => right.push(new_dialog(id, list)),
        _ => {}
    }

    Surface::new(
        Page::new().child(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(28.0)
                .children(vec![
                    Stack::new().gap(6.0).children(column).into(),
                    Stack::new().gap(16.0).children(right).into(),
                ]),
        ),
    )
    .with_id(MAIN)
}

#[portaki_sdk::wire(serialize)]
struct SurfaceInputSelectedId<'a> {
    selected_id: &'a str,
    /// Vidé en changeant de liste : une ligne ajoutée n'appartient qu'à la sienne.
    new_steps: &'a str,
}

fn select(selected_id: &str) -> Action {
    Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(json_value(SurfaceInputSelectedId {
            selected_id,
            new_steps: "",
        })),
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

fn options(name: &str, values: &[&str]) -> Vec<ChoiceOption> {
    values
        .iter()
        .map(|v| ChoiceOption::new(*v, format!("i18n:host.{name}.{v}")))
        .collect()
}

fn select_field(label: &str, name: &str, values: &[&str], value: &str) -> Component {
    Field::new()
        .label(label)
        .name(name)
        .child(
            Select::new()
                .name(name)
                .options(options(name, values))
                .value(value),
        )
        .into()
}

/// The right side for one list: its settings card, then its steps.
fn edit_panel(
    list: &Checklist,
    items: &[&ChecklistItem],
    new_steps: &[Uuid],
    lang: &str,
) -> Vec<Component> {
    let host = list.audience == lists::HOST;
    let problems = crate::queries::list_problems(list, items);
    let error_of = |field: &str| {
        problems
            .iter()
            .find(|(name, _)| *name == field)
            .map(|(_, error)| error.get(lang).to_string())
    };
    let with_error = |field: Field, name: &str| match error_of(name) {
        Some(error) => field.error(error),
        None => field,
    };
    let name_field: Component = Field::new()
        .label("i18n:host.field.name")
        .name("name")
        .required(true)
        .child(
            TextInput::new()
                .name("name")
                .value(name(list, lang == "fr")),
        )
        .into();
    let id_field: Component = TextInput::new()
        .name("id")
        .value(list.id.to_string())
        .into();

    let settings = if host {
        let assignee = match (&list.assignee_name, &list.assignee_role) {
            (Some(name), Some(role)) => format!("{name} · {role}"),
            (Some(name), None) => name.clone(),
            _ => String::new(),
        };
        Section::new()
            .title("i18n:host.task.title")
            .child(id_field)
            .child(name_field)
            .child(select_field(
                "i18n:host.field.hostTrigger",
                "trigger",
                HOST_TRIGGERS,
                &list.trigger,
            ))
            .child(
                Grid::new()
                    .minColumnWidth(220.0)
                    .gap(14.0)
                    .child(
                        // ponytail: texte libre — aucun accès du module aux membres de l'équipe du
                        // compte ; une liste déroulante quand la plateforme les exposera.
                        Field::new()
                            .label("i18n:host.field.assignee")
                            .name("assignee")
                            .child(
                                TextInput::new()
                                    .name("assignee")
                                    .value(assignee)
                                    .placeholder("i18n:host.field.assignee.placeholder"),
                            )
                            .child(FieldHint::new().text("i18n:host.field.assignee.hint")),
                    )
                    .child(select_field(
                        "i18n:host.field.deadline",
                        "deadline",
                        DEADLINES,
                        list.deadline.as_deref().unwrap_or(lists::NEXT_ARRIVAL),
                    )),
            )
            // Enregistrés mais lus par personne : pas de canal pour les tenir. Annoncés, pas
            // proposés — le formulaire ne les envoie pas, la valeur enregistrée reste.
            .child(soon("notifyAssignee"))
            .child(soon("alertHost"))
    } else {
        let display = storage::display::read(list.id);
        let mut moments = GUEST_TRIGGERS.to_vec();
        if list.trigger == lists::BEFORE_ARRIVAL {
            moments.push(lists::BEFORE_ARRIVAL);
        }
        Section::new()
            .title("i18n:host.display.title")
            .child(id_field)
            .child(name_field)
            .child(
                Field::new()
                    .label("i18n:host.field.guestTrigger")
                    .name("trigger")
                    .child(
                        ChoiceList::new()
                            .name("trigger")
                            .layout(ChoiceListLayout::Segmented)
                            .choices(options("trigger", &moments))
                            .value(lists::pick(&list.trigger, lists::GUEST_TRIGGERS)),
                    ),
            )
            // The value as stored, out of bounds too: the error under it must point at it.
            .child(with_error(
                Field::new()
                    .label("i18n:host.field.visibleLimit")
                    .name("visible_limit")
                    .child(
                        NumberInput::new()
                            .name("visible_limit")
                            .min(f64::from(storage::display::MIN_VISIBLE_LIMIT))
                            .max(f64::from(storage::display::MAX_VISIBLE_LIMIT))
                            .value(f64::from(
                                display.visible_limit.unwrap_or(display.visible_limit()),
                            )),
                    )
                    .child(FieldHint::new().text("i18n:host.field.visibleLimit.hint")),
                "visible_limit",
            ))
            .child(with_error(
                Field::new()
                    .label("i18n:host.field.doneMessage")
                    .name("done_message")
                    .child(
                        TextInput::new()
                            .name("done_message")
                            .value(
                                labels::decode_map(&display.done_message)
                                    .get(lang)
                                    .cloned()
                                    .unwrap_or_default(),
                            )
                            .placeholder("i18n:guest.done.message"),
                    )
                    .child(FieldHint::new().text("i18n:host.field.doneMessage.hint")),
                "done_message",
            ))
            .child(
                ToggleRow::new()
                    .name("remind")
                    .label("i18n:host.toggle.remind")
                    .description("i18n:host.toggle.remind.desc")
                    .icon(IconName::Bell)
                    .checked(display.remind()),
            )
    };

    vec![
        Form::new()
            .child(settings)
            .child(steps_section(list, items, new_steps, lang, &error_of))
            .into(),
        Text::new()
            .text(format!(
                "i18n:host.where.{}",
                if host { "host" } else { "guest" }
            ))
            .variant(TextVariant::Caption)
            .into(),
        Button::new()
            .label("i18n:host.lists.delete")
            .variant(ButtonVariant::Ghost)
            .tone(Tone::Danger)
            .action(crate::ids::module_id().command(
                crate::commands::DELETE_CHECKLIST,
                DeleteChecklistArgs { id: list.id },
            ))
            .into(),
    ]
}

/// A setting the board shows but no platform channel delivers yet: its label, and « Bientôt ».
fn soon(key: &str) -> Component {
    Stack::new()
        .gap(4.0)
        .child(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(8.0)
                .child(
                    Text::new()
                        .text(format!("i18n:host.toggle.{key}"))
                        .variant(TextVariant::Body),
                )
                .child(Pill::new().label("i18n:host.soon").tone(Tone::Neutral)),
        )
        .child(
            Text::new()
                .text(format!("i18n:host.toggle.{key}.desc"))
                .variant(TextVariant::Caption),
        )
        .into()
}

/// « Étapes N / 40 » : the sortable table, one dialog per row, the code warning.
fn steps_section(
    list: &Checklist,
    items: &[&ChecklistItem],
    new_steps: &[Uuid],
    lang: &str,
    error_of: &dyn Fn(&str) -> Option<String>,
) -> Component {
    let host = list.audience == lists::HOST;
    let mut rows: Vec<Step> = items.iter().map(|item| Step::of(item, lang)).collect();
    for id in new_steps {
        if !items.iter().any(|item| item.id == *id) {
            rows.push(Step::blank(*id));
        }
    }

    let mut children: Vec<Component> = vec![FieldHint::new()
        .text(format!(
            "i18n:host.steps.hint.{}",
            if host { "host" } else { "guest" }
        ))
        .into()];
    if let Some(error) = error_of("steps") {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    let mut columns = vec![
        json!({ "key": "label", "label": "i18n:host.step.label", "sub": "detail" }),
        json!({ "key": "zone", "label": "i18n:host.step.zone" }),
    ];
    if host {
        columns.push(json!({ "key": "photo", "label": "i18n:host.step.photo", "kind": "pill" }));
    }
    let mut table = DataTable::new()
        .rowName("steps")
        .sortable(true)
        .panel("dialog")
        .columns(Value::Array(columns))
        .rows(Value::Array(
            rows.iter().map(|step| step.row(host)).collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, step)| step.panel(index, host))
                .collect(),
        );
    if rows.len() < MAX_STEPS {
        let mut added: Vec<String> = new_steps
            .iter()
            .map(|step| format!("{}:{step}", list.id))
            .collect();
        added.push(format!("{}:{}", list.id, Uuid::new_v4()));
        let mut input = Map::new();
        input.insert(NEW_STEPS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.steps.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());

    // Un code écrit dans une étape du livret se lit sans calendrier de révélation (§3).
    if let Some(step) = rows.iter().find(|step| !host && step.code) {
        if let Ok(warning) = t!("host.steps.code", step = step.label.as_str()) {
            children.push(
                InlineNotice::new()
                    .tone(Tone::Warning)
                    .message(warning)
                    .into(),
            );
        }
    }

    Section::new()
        .title("i18n:host.steps.title")
        .subtitle(format!("{} / {MAX_STEPS}", rows.len()))
        .children(children)
        .into()
}

/// One step as the table shows it and its dialog edits it (in the host's language).
struct Step {
    id: Uuid,
    /// Shown in the table: the host's language, else another.
    label: String,
    detail: String,
    zone: String,
    /// Edited in the dialog: the host's language only — a fallback saved there would be written
    /// into it.
    own: (String, String, String),
    photo: bool,
    code: bool,
    error: Option<String>,
}

impl Step {
    fn of(item: &ChecklistItem, lang: &str) -> Self {
        let label = labels::labels_from_item(item);
        let detail = labels::decode_map(&item.description_i18n);
        let zone = labels::decode_map(&item.group_i18n);
        let own = |map: &labels::Labels| map.get(lang).cloned().unwrap_or_default();
        Self {
            id: item.id,
            label: labels::pick_label(&label, lang, "fr"),
            detail: labels::pick_label(&detail, lang, "fr"),
            zone: labels::pick_label(&zone, lang, "fr"),
            own: (own(&label), own(&detail), own(&zone)),
            photo: item.photo_required,
            code: label
                .values()
                .chain(detail.values())
                .any(|text| looks_like_code(text)),
            error: step_problem(item).map(|error| error.get(lang).to_string()),
        }
    }

    fn blank(id: Uuid) -> Self {
        Self {
            id,
            label: String::new(),
            detail: String::new(),
            zone: String::new(),
            own: Default::default(),
            photo: false,
            code: false,
            error: None,
        }
    }

    fn row(&self, host: bool) -> Value {
        json!({
            "id": self.id.to_string(),
            "label": self.label,
            "detail": self.detail,
            "zone": if self.zone.is_empty() { "—" } else { self.zone.as_str() },
            "photo": if host && self.photo { "i18n:host.step.photo" } else { "" },
        })
    }

    /// Étape, Précision, Zone ; « Photo obligatoire » pour une liste d'équipe.
    fn panel(&self, index: usize, host: bool) -> Component {
        let name = |key: &str| format!("steps.{index}.{key}");
        let (label, detail, zone) = &self.own;
        // Vide dans la langue de l'hôte : ce qu'une autre langue dit, en indice.
        let input = |key: &str, own: &str, shown: &str| {
            let input = TextInput::new().name(name(key)).value(own);
            if own.is_empty() && !shown.is_empty() {
                input.placeholder(shown)
            } else {
                input
            }
        };
        let mut label_field = Field::new()
            .name(name("label"))
            .label("i18n:host.step.label")
            .required(true)
            .child(input("label", label, &self.label));
        if let Some(error) = &self.error {
            label_field = label_field.error(error.clone());
        } else if self.code && !host {
            label_field = label_field.child(FieldHint::new().text("i18n:host.step.code"));
        }
        let mut fields: Vec<Component> = vec![
            label_field.into(),
            Field::new()
                .name(name("detail"))
                .label("i18n:host.step.detail")
                .child(input("detail", detail, &self.detail))
                .into(),
            Field::new()
                .name(name("zone"))
                .label("i18n:host.step.zone")
                .child(input(
                    "zone",
                    zone,
                    if self.zone.is_empty() {
                        "i18n:host.step.zone.placeholder"
                    } else {
                        &self.zone
                    },
                ))
                .into(),
        ];
        if host {
            fields.push(
                ToggleRow::new()
                    .name(name("photo"))
                    .label("i18n:host.step.photo")
                    .description("i18n:host.step.photo.desc")
                    .icon(IconName::Image)
                    .checked(self.photo)
                    .into(),
            );
        }
        Stack::new().gap(14.0).children(fields).into()
    }
}

/// The six cards of « Nouvelle checklist », each creating its list under `id`.
fn templates(id: Uuid) -> Component {
    let cards = TEMPLATES
        .iter()
        .map(|template| {
            SelectableCard::new()
                .value(template.id)
                .label(format!("i18n:template.{}.name", template.id))
                .description(format!("i18n:template.{}.desc", template.id))
                .icon(template.icon)
                .action(crate::ids::module_id().command(
                    crate::commands::CREATE_CHECKLIST,
                    CreateChecklistArgs {
                        template: template.id.to_string(),
                        id: Some(id),
                    },
                ))
                .into()
        })
        .collect();
    Grid::new()
        .minColumnWidth(180.0)
        .gap(10.0)
        .children(cards)
        .into()
}

/// « Nouvelle checklist » over the list being edited. Keyed by `id`: each click is a new id, so a
/// dialog closed by its cross opens again (the dashboard keeps a closed `Modal` closed).
fn new_dialog(id: Uuid, back_to: &Checklist) -> Component {
    Modal::new()
        .id(format!("new-{id}"))
        .title("i18n:host.new.title")
        .child(
            Stack::new()
                .gap(14.0)
                .child(templates(id))
                .child(FieldHint::new().text("i18n:host.new.hint"))
                .child(
                    Button::new()
                        .label("i18n:host.new.cancel")
                        .variant(ButtonVariant::Outline)
                        .action(select(&back_to.id.to_string())),
                ),
        )
        .into()
}

/// The icon a list was saved with. Stored as text, so a row written before the vocabulary was
/// typed may name a token no shell draws: it falls back to the checklist's own icon.
fn stored_icon(wire: &str) -> IconName {
    IconName::from_wire(wire).unwrap_or(IconName::CheckCircle)
}
