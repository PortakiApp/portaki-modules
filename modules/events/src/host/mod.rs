//! Host dashboard surface — the events settings page (spec Événements §2).
//!
//! `Section`s — the shell's cards — in the board's order: « Autour du logement » (the public
//! events and the radius), « Vos événements », then the guest-facing note. The events are a table
//! (règles communes §2.4 bis), sorted by date and not reorderable: each row opens its form in the
//! side panel (`panel: "sheet"`), `children[i]` of the `DataTable`. A rule broken shows under its
//! field (`Field::error`), the same one `publishReadiness` blocks on.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, ChoiceList, DataTable, DatePicker, Field, FieldHint, Form, ImageUpload,
    InlineNotice, NumberInput, Page, Section, Stack, StepList, Text, TextArea, TextInput,
    TimePicker, Toggle,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::config::{EventRow, ModuleConfig};
use crate::nearby::{has_open_agenda, host_now};

mod calendar;
pub use calendar::render_host_calendar;

pub use crate::config::MAX_EVENTS;
use crate::config::{RECURRENCES, WEEKDAYS};

/// Les événements ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_EVENTS: &str = "newEvents";
/// Au plus cinquante dates par événement : une borne de formulaire, pas un réglage.
const MAX_DATES: usize = 50;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Calendar
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let view = View::load(&ctx)?;
    let disclaimer = view.config.disclaimer.host_value(&ctx);
    let form = Form::new()
        .child(nearby_section(&view, has_open_agenda(&ctx), &ctx))
        .child(events_section(&view, &ctx))
        .child(
            Section::new()
                .title("i18n:host.section.disclaimer")
                .subtitle("i18n:host.section.disclaimer.help")
                .child(
                    Field::new()
                        .name("disclaimer")
                        .label("i18n:host.disclaimer.label")
                        .child(
                            TextArea::new()
                                .name("disclaimer")
                                .value(disclaimer)
                                .placeholder("i18n:host.disclaimer.placeholder"),
                        ),
                ),
        );
    Ok(Surface::new(Page::new().child(form)).with_id(MAIN))
}

/// Ce que la page lit : la config, le repère du logement, et les lignes du tableau.
struct View {
    config: ModuleConfig,
    home: Option<(f64, f64)>,
    /// Les lignes, dans l'ordre affiché : `(rang stocké, événement)`, `None` pour une ajoutée.
    rows: Vec<(Option<usize>, EventRow)>,
    /// Les ajoutées pas encore enregistrées, pour les garder d'un rendu à l'autre.
    added: Vec<String>,
}

impl View {
    fn load(ctx: &HostContext) -> Result<Self> {
        let config = ModuleConfig::load(ctx)?;
        let home = ctx
            .property
            .coordinates
            .as_ref()
            .map(|point| (point.lat, point.lng))
            .filter(|(lat, lng)| *lat != 0.0 || *lng != 0.0);
        let (now, tz) = (host_now(), ctx.property_tz());
        let stored = &config.events;

        // La plateforme fusionne une ligne reçue avec la ligne stockée de même id, sinon de même
        // rang. Une ligne remplie sans id ne se retrouve que par son rang : tant qu'il y en a une,
        // la liste garde l'ordre et les lignes enregistrés, vides comprises.
        // ponytail: ces lignes d'avant les id disparaissent au premier enregistrement (la ligne
        // reçoit alors son id), et le tri revient.
        let legacy = stored
            .iter()
            .any(|e| !e.is_blank() && e.id.trim().is_empty());
        let mut rows: Vec<(Option<usize>, EventRow)> = stored
            .iter()
            .enumerate()
            .filter(|(_, e)| legacy || !e.is_blank())
            .map(|(index, e)| {
                let mut e = e.clone();
                if e.id.trim().is_empty() {
                    e.id = format!("evt-{}", index + 1);
                }
                (Some(index), e)
            })
            .collect();
        if !legacy {
            // À venir d'abord, par date ; sans date ensuite ; les passés en bas, le plus récent
            // en premier.
            rows.sort_by_key(|(_, e)| {
                let at = next_start(e, now, tz.as_ref());
                match (is_past(e, now, tz.as_ref()), at) {
                    (false, Some(at)) => (0, at.timestamp()),
                    (false, None) => (1, 0),
                    (true, at) => (2, -at.map_or(0, |at| at.timestamp())),
                }
            });
        }
        let added: Vec<String> = ctx
            .input_str(NEW_EVENTS)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty() && !stored.iter().any(|e| e.id == *id))
            .map(str::to_string)
            .collect();
        for id in &added {
            if rows.len() >= MAX_EVENTS {
                break;
            }
            rows.push((
                None,
                EventRow {
                    id: id.clone(),
                    ..EventRow::default()
                },
            ));
        }
        Ok(Self {
            config,
            home,
            rows,
            added,
        })
    }

    /// Le message sous `field`, dans la langue de l'hôte.
    fn error(&self, field: &str, ctx: &HostContext) -> Option<String> {
        self.config
            .error_of(field, self.home)
            .map(|error| error.get(&ctx.locale).to_string())
    }
}

/// La prochaine occurrence à partir de `now` (la dernière, toutes passées).
fn next_start(
    event: &EventRow,
    now: chrono::DateTime<chrono::Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    crate::time_format::parse_starts_at(&event.timed().next_from(now, tz).starts_at)
}

/// Passé : sa prochaine occurrence (sa fin, sinon son début) est derrière `now`. Retiré du
/// livret par la fenêtre du séjour ; gris dans le tableau.
fn is_past(
    event: &EventRow,
    now: chrono::DateTime<chrono::Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
) -> bool {
    let next = event.timed().next_from(now, tz);
    next.ends_at
        .as_deref()
        .and_then(crate::time_format::parse_starts_at)
        .or_else(|| crate::time_format::parse_starts_at(&next.starts_at))
        .is_some_and(|at| at < now)
}

/// §2.1 Autour du logement : les événements publics d'OpenAgenda, et le rayon.
fn nearby_section(view: &View, open_agenda: bool, ctx: &HostContext) -> Component {
    let config = &view.config;
    // Le rayon tel qu'enregistré, pas borné : un 60 d'avant reste 60, avec l'erreur dessous.
    let mut radius = Field::new()
        .name("radius_km")
        .label("i18n:host.nearby.radius")
        .required(true)
        .child(
            NumberInput::new()
                .name("radius_km")
                .min(1.0)
                .max(50.0)
                .value(f64::from(config.radius_km)),
        )
        .child(FieldHint::new().text("i18n:host.nearby.radius.hint"));
    if let Some(error) = view.error("radius_km", ctx) {
        radius = radius.error(error);
    }
    let mut children: Vec<Component> = vec![
        Field::new()
            .name("nearby_enabled")
            .label("i18n:host.nearby.enabled")
            .child(
                Toggle::new()
                    .name("nearby_enabled")
                    .checked(config.nearby_enabled),
            )
            .child(FieldHint::new().text("i18n:host.section.nearby.help"))
            .into(),
        radius.into(),
    ];
    if config.nearby_enabled && !open_agenda {
        children.push(
            Text::new()
                .text("i18n:host.nearby.status.missingKey")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    Section::new()
        .title("i18n:host.section.nearby")
        .children(children)
        .into()
}

/// §2.2 Vos événements : le tableau trié par date, et le panneau de chaque ligne.
fn events_section(view: &View, ctx: &HostContext) -> Component {
    let (now, tz) = (host_now(), ctx.property_tz());
    let mut children: Vec<Component> = vec![Text::new()
        .text(format!("{} / {MAX_EVENTS}", view.rows.len()))
        .variant(TextVariant::Caption)
        .into()];
    if let Some(error) = view.error("events", ctx) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    let mut table = DataTable::new()
        .rowName("events")
        .panel("sheet")
        .columns(json!([
            { "key": "when", "label": "i18n:host.column.when", "mono": true },
            { "key": "name", "label": "i18n:host.column.event", "sub": "price" },
            { "key": "place", "label": "i18n:host.column.place" },
            { "key": "badge", "label": "i18n:host.column.status", "kind": "pill" },
        ]))
        .rows(Value::Array(
            view.rows
                .iter()
                .map(|(_, event)| row(event, now, tz.as_ref(), ctx))
                .collect(),
        ))
        .children(
            view.rows
                .iter()
                .enumerate()
                .map(|(rank, (stored, event))| event_panel(view, rank, *stored, event, ctx))
                .collect(),
        );
    if view.rows.len() < MAX_EVENTS {
        let mut added = view.added.clone();
        added.push(uuid::Uuid::new_v4().to_string());
        table = table
            .addLabel("i18n:host.events.add")
            .addAction(emit_input(new_events_input(&added)));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.events.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(table.into());

    Section::new()
        .title("i18n:host.events.title")
        .subtitle("i18n:host.events.subtitle")
        .children(children)
        .into()
}

fn new_events_input(added: &[String]) -> Map<String, Value> {
    let mut input = Map::new();
    input.insert(NEW_EVENTS.into(), Value::String(added.join(",")));
    input
}

fn emit_input(input: Map<String, Value>) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(Value::Object(input)))
}

/// Une ligne du tableau : Quand · Événement (prix dessous) · Lieu · pastille.
fn row(
    event: &EventRow,
    now: chrono::DateTime<chrono::Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
    ctx: &HostContext,
) -> Value {
    let place = match event.place.host_value(ctx).trim() {
        "" => event.address.clone().unwrap_or_default(),
        place => place.to_string(),
    };
    let badge = if event.cancelled {
        "i18n:host.event.cancelled".to_string()
    } else if is_past(event, now, tz) {
        "i18n:host.event.badge.past".to_string()
    } else {
        match event.recurrence() {
            "none" => String::new(),
            recurrence => format!("i18n:host.event.recurrence.label.{recurrence}"),
        }
    };
    json!({
        "id": event.id,
        "when": when(event, now, tz),
        "name": event.title.host_value(ctx),
        "price": event.price.clone().unwrap_or_default(),
        "place": place,
        "badge": badge,
    })
}

/// « mar., sam. · 08:00 » chaque semaine, sinon « 15/08 · 22:30 » à la prochaine date ; « — »
/// sans date lisible. L'heure du mur, comme le livret l'affiche.
fn when(
    event: &EventRow,
    now: chrono::DateTime<chrono::Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
) -> String {
    let timed = event.timed();
    let Some(at) = crate::time_format::parse_starts_at(&timed.next_from(now, tz).starts_at) else {
        return "—".into();
    };
    let day = if event.recurrence() == "weekly" {
        timed
            .effective_weekdays(tz)
            .into_iter()
            .map(|day| {
                let key = WEEKDAYS[day.num_days_from_monday() as usize];
                t!(&format!("host.event.weekday.{key}")).unwrap_or_else(|_| key.to_string())
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        at.format("%d/%m").to_string()
    };
    if event.all_day {
        day
    } else {
        format!("{day} · {}", at.format("%H:%M"))
    }
}

/// Le panneau d'une ligne : tous les champs de l'événement, nommés `events.<rang>.<clé>`. Les
/// erreurs se lisent au rang stocké, celui de `problems`.
fn event_panel(
    view: &View,
    rank: usize,
    stored: Option<usize>,
    event: &EventRow,
    ctx: &HostContext,
) -> Component {
    let name = |key: &str| format!("events.{rank}.{key}");
    let field = |key: &str, label: &str| {
        let field = Field::new().name(name(key)).label(label);
        match stored.and_then(|index| view.error(&format!("events.{index}.{key}"), ctx)) {
            Some(error) => field.error(error),
            None => field,
        }
    };
    let (date, start_time, end_time) = event.when_fields();
    let recurrence = event.recurrence();
    let note = event
        .note
        .as_ref()
        .map(|note| note.host_value(ctx))
        .unwrap_or_default()
        .to_string();

    let mut picker = AddressMapPicker::new()
        .addressName(name("address"))
        .latName(name("lat"))
        .lngName(name("lng"))
        .address(event.address.as_deref().unwrap_or(""))
        .hint("i18n:host.event.where.hint");
    if let (Some(lat), Some(lng)) = (event.lat, event.lng) {
        picker = picker.lat(lat).lng(lng);
    }

    Stack::new()
        .gap(14.0)
        .children(vec![
            field("title", "i18n:host.event.title")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("title"))
                        .value(event.title.host_value(ctx))
                        .placeholder("i18n:host.event.title.placeholder"),
                )
                .into(),
            // « Quand » : le jour, puis début – fin.
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(10.0)
                .children(vec![
                    field("date", "i18n:host.event.when")
                        .required(true)
                        .child(DatePicker::new().name(name("date")).value(date))
                        .into(),
                    field("start_time", "i18n:host.event.startsAt")
                        .child(TimePicker::new().name(name("start_time")).value(start_time))
                        .into(),
                    field("end_time", "i18n:host.event.endsAt")
                        .child(TimePicker::new().name(name("end_time")).value(end_time))
                        .into(),
                ])
                .into(),
            field("all_day", "i18n:host.event.allDay")
                .child(Toggle::new().name(name("all_day")).checked(event.all_day))
                .into(),
            field("recurrence", "i18n:host.event.recurrence")
                .child(
                    ChoiceList::new()
                        .name(name("recurrence"))
                        .layout(ChoiceListLayout::Segmented)
                        .choices(
                            RECURRENCES
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(
                                        *key,
                                        format!("i18n:host.event.recurrence.label.{key}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(recurrence),
                )
                .into(),
            field("weekdays", "i18n:host.event.weekdays")
                .visibleWhen(format!("{}=weekly", name("recurrence")))
                .child(
                    ChoiceList::new()
                        .name(name("weekdays"))
                        .multi(true)
                        .layout(ChoiceListLayout::Compact)
                        .choices(
                            WEEKDAYS
                                .iter()
                                .map(|day| {
                                    ChoiceOption::new(
                                        *day,
                                        format!("i18n:host.event.weekday.{day}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(serde_json::to_string(&event.weekdays).unwrap_or_default()),
                )
                .child(FieldHint::new().text("i18n:host.event.weekdays.hint"))
                .into(),
            dates_field(view, rank, stored, event, ctx),
            field("lat", "i18n:host.event.where")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("place"))
                        .value(event.place.host_value(ctx))
                        .placeholder("i18n:host.event.place.placeholder"),
                )
                // Le lieu par le sélecteur de carte, et non deux cases de coordonnées : un hôte
                // ne connaît pas la latitude de la place du port.
                .child(picker)
                .into(),
            field("photo", "i18n:host.event.photo")
                .child(
                    ImageUpload::new()
                        .name(name("photo"))
                        .value(event.photo.as_str()),
                )
                .into(),
            field("note", "i18n:host.event.note")
                .child(
                    TextArea::new()
                        .name(name("note"))
                        .value(note)
                        .rows(3)
                        .placeholder("i18n:host.event.note.placeholder"),
                )
                .into(),
            field("price", "i18n:host.event.price")
                .child(
                    TextInput::new()
                        .name(name("price"))
                        .value(event.price.as_deref().unwrap_or(""))
                        .placeholder("i18n:host.event.price.placeholder"),
                )
                .into(),
            field("url", "i18n:host.event.url")
                .child(
                    TextInput::new()
                        .name(name("url"))
                        .value(event.url.as_deref().unwrap_or(""))
                        .placeholder("https://"),
                )
                .into(),
            field("tips", "i18n:host.event.tips")
                .child(
                    TextArea::new()
                        .name(name("tips"))
                        .value(event.tips.host_value(ctx))
                        .rows(3)
                        .placeholder("i18n:host.event.tips.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.event.tips.hint"))
                .into(),
            // L'accès après les conseils dans le formulaire (la planche), mais avant eux dans la
            // fiche : un voyageur en fauteuil ne doit pas le chercher au milieu des bons plans.
            field("access", "i18n:host.event.access")
                .child(
                    TextInput::new()
                        .name(name("access"))
                        .value(event.access.host_value(ctx))
                        .placeholder("i18n:host.event.access.placeholder"),
                )
                .into(),
            field("cancelled", "i18n:host.event.cancelled")
                .child(
                    Toggle::new()
                        .name(name("cancelled"))
                        .checked(event.cancelled),
                )
                .child(FieldHint::new().text("i18n:host.event.cancelled.hint"))
                .into(),
        ])
        .into()
}

/// Les dates d'un événement « Dates choisies », en liste : une ligne par date. « Ajouter une
/// date » en dessine une de plus sur cet événement seul.
fn dates_field(
    view: &View,
    rank: usize,
    stored: Option<usize>,
    event: &EventRow,
    ctx: &HostContext,
) -> Component {
    let count_key = format!("dates.{}", event.id);
    let count = ctx
        .input_u64(&count_key)
        .map_or(0, |n| n as usize)
        .max(event.dates.len())
        .min(MAX_DATES);
    let rows: Vec<Component> = (0..count)
        .map(|slot| {
            DatePicker::new()
                .name(format!("events.{rank}.dates.{slot}.date"))
                .value(event.dates.get(slot).map_or("", |d| d.date.as_str()))
                .into()
        })
        .collect();
    let mut input = new_events_input(&view.added);
    input.insert(count_key, json!((count + 1).min(MAX_DATES)));
    let name = format!("events.{rank}.dates");
    let mut field = Field::new()
        .name(name.clone())
        .label("i18n:host.event.dates")
        .visibleWhen(format!("events.{rank}.recurrence=dates"));
    if let Some(error) = stored.and_then(|index| view.error(&format!("events.{index}.dates"), ctx))
    {
        field = field.error(error);
    }
    field
        .child(
            StepList::new()
                .addLabel("i18n:host.event.dates.add")
                .removeLabel("i18n:host.events.remove")
                .emptyTitle("i18n:host.event.dates.empty")
                .itemKeyPrefix(name)
                .addAction(emit_input(input))
                .children(rows),
        )
        .into()
}
