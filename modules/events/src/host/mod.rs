//! Host dashboard surfaces — config cards embedded in the module sheet.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, FieldHint, Form, ImageUpload, Page, Select, Stack, StepList,
    Text, TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{EventRow, ModuleConfig};
use crate::nearby::has_open_agenda;

pub use crate::config::MAX_EVENTS;
use crate::config::RECURRENCES;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Calendar
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let disclaimer = config.disclaimer.host_value(&ctx);
    let open_agenda = has_open_agenda(&ctx);

    let form_children: Vec<Component> = vec![
        nearby_card(&config, open_agenda),
        events_card(&config, &ctx),
        Card::new()
            .title("i18n:host.section.disclaimer")
            .subtitle("i18n:host.section.disclaimer.help")
            .icon(IconName::InfoCircle)
            .children(vec![Field::new()
                .name("disclaimer")
                .label("i18n:host.disclaimer.label")
                .child(
                    TextArea::new()
                        .name("disclaimer")
                        .value(disclaimer)
                        .placeholder("i18n:host.disclaimer.placeholder"),
                )
                .into()])
            .into(),
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    ];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Ok(Surface::new(Page::new().child(Form::new().children(form_children))).with_id(MAIN))
}

fn nearby_card(config: &ModuleConfig, open_agenda: bool) -> Component {
    let status_key = if !config.nearby_enabled {
        "i18n:host.nearby.status.off"
    } else if open_agenda {
        "i18n:host.nearby.status.ready"
    } else {
        "i18n:host.nearby.status.missingKey"
    };
    let nearby_value = if config.nearby_enabled {
        "true"
    } else {
        "false"
    };
    let radius_value = config.normalized_radius_km().to_string();

    Card::new()
        .title("i18n:host.section.nearby")
        .subtitle("i18n:host.section.nearby.help")
        .icon(IconName::MapPin)
        .children(vec![
            Field::new()
                .name("nearby_enabled")
                .label("i18n:host.nearby.enabled")
                .child(
                    Select::new()
                        .name("nearby_enabled")
                        .options(vec![
                            ChoiceOption::new("true", "i18n:host.nearby.enabled.on"),
                            ChoiceOption::new("false", "i18n:host.nearby.enabled.off"),
                        ])
                        .value(nearby_value),
                )
                .into(),
            Field::new()
                .name("radius_km")
                .label("i18n:host.nearby.radius")
                .child(
                    Select::new()
                        .name("radius_km")
                        .options(vec![
                            ChoiceOption::new("10", "i18n:host.nearby.radius.10"),
                            ChoiceOption::new("20", "i18n:host.nearby.radius.20"),
                            ChoiceOption::new("40", "i18n:host.nearby.radius.40"),
                            ChoiceOption::new("60", "i18n:host.nearby.radius.60"),
                            ChoiceOption::new("100", "i18n:host.nearby.radius.100"),
                        ])
                        .value(radius_value),
                )
                .into(),
            Text::new()
                .text(status_key)
                .variant(TextVariant::Caption)
                .into(),
        ])
        .into()
}

/// Les événements saisis à la main, en lignes dynamiques bornées.
fn events_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.events.len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| event_row(index, config, ctx))
        .collect();

    Card::new()
        .title("i18n:host.events.title")
        .subtitle("i18n:host.events.subtitle")
        .icon(IconName::Calendar)
        .child(
            StepList::new()
                .addLabel("i18n:host.events.add")
                .removeLabel("i18n:host.events.remove")
                .emptyTitle("i18n:host.events.emptyTitle")
                .emptyDescription("i18n:host.events.emptyDescription")
                .itemKeyPrefix("events")
                .addAction(emit_input(RowCount {
                    events_count: (rows_count + 1).min(MAX_EVENTS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("events_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_EVENTS),
        None => stored.clamp(1, MAX_EVENTS),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    events_count: usize,
}

fn emit_input(payload: impl serde::Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn event_row(index: usize, config: &ModuleConfig, ctx: &HostContext) -> Component {
    let event: Option<&EventRow> = config.events.get(index);
    // Le champ, avec le message de `problems` sous lui s'il y en a un.
    let named = |key: &str| {
        let name = format!("events.{index}.{key}");
        let field = Field::new().name(name.clone());
        match config.error_of(&name) {
            Some(error) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
    let title = event.map(|e| e.title.host_value(ctx)).unwrap_or_default();
    let place = event.map(|e| e.place.host_value(ctx)).unwrap_or_default();
    let starts_at = event.map(|e| e.starts_at.as_str()).unwrap_or("");
    let url = event.and_then(|e| e.url.as_deref()).unwrap_or("");

    let ends_at = event.and_then(|e| e.ends_at.as_deref()).unwrap_or("");
    let price = event.and_then(|e| e.price.as_deref()).unwrap_or("");
    let photo = event.map(|e| e.photo.clone()).unwrap_or_default();
    let address = event.and_then(|e| e.address.as_deref()).unwrap_or("");
    let note = event
        .and_then(|e| e.note.as_ref())
        .map(|note| note.host_value(ctx))
        .unwrap_or_default()
        .to_string();
    let access = event.map(|e| e.access.host_value(ctx)).unwrap_or_default();
    let tips = event
        .map(|e| e.tips.host_value(ctx))
        .unwrap_or_default()
        .to_string();

    let mut picker = AddressMapPicker::new()
        .addressName(format!("events.{index}.address"))
        .latName(format!("events.{index}.lat"))
        .lngName(format!("events.{index}.lng"))
        .address(address)
        .label("i18n:host.event.where")
        .hint("i18n:host.event.where.hint");
    if let (Some(lat), Some(lng)) = (event.and_then(|e| e.lat), event.and_then(|e| e.lng)) {
        picker = picker.lat(lat).lng(lng);
    }

    // A filled row sends its id, so a save merges into it (and keeps its end, its note, its
    // other languages). A blank slot has nothing to keep — and an id would make it count as filled.
    let id = event
        .filter(|e| !e.is_blank())
        .map(|e| sdui::row_id("events", index, Some(&e.id)));

    let fields: Vec<Component> = vec![
        named("title")
            .label("i18n:host.event.title")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.title"))
                    .value(title),
            )
            .into(),
        Field::new()
            .name(format!("events.{index}.place"))
            .label("i18n:host.event.place")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.place"))
                    .value(place),
            )
            .into(),
        Field::new()
            .name(format!("events.{index}.starts_at"))
            .label("i18n:host.event.startsAt")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.starts_at"))
                    .value(starts_at)
                    .placeholder("i18n:host.event.startsAt.placeholder"),
            )
            .into(),
        named("url")
            .label("i18n:host.event.url")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.url"))
                    .value(url),
            )
            .into(),
        named("ends_at")
            .label("i18n:host.event.endsAt")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.ends_at"))
                    .value(ends_at)
                    .placeholder("i18n:host.event.startsAt.placeholder"),
            )
            .into(),
        FieldHint::new().text("i18n:host.event.endsAt.hint").into(),
        Field::new()
            .name(format!("events.{index}.all_day"))
            .label("i18n:host.event.allDay")
            .child(
                Toggle::new()
                    .name(format!("events.{index}.all_day"))
                    .checked(event.is_some_and(|e| e.all_day)),
            )
            .into(),
        Field::new()
            .name(format!("events.{index}.recurrence"))
            .label("i18n:host.event.recurrence")
            .child(
                Select::new()
                    .name(format!("events.{index}.recurrence"))
                    .options(
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
                    .value(event.map_or("none", EventRow::recurrence)),
            )
            .into(),
        FieldHint::new()
            .text("i18n:host.event.recurrence.hint")
            .into(),
        named("price")
            .label("i18n:host.event.price")
            .child(
                TextInput::new()
                    .name(format!("events.{index}.price"))
                    .value(price)
                    .placeholder("i18n:host.event.price.placeholder"),
            )
            .into(),
        // Le lieu par le sélecteur de carte, et non deux cases de coordonnées : un hôte ne connaît
        // pas la latitude de la place du port, et une virgule de travers posait l'événement
        // au large.
        picker.into(),
        named("note")
            .label("i18n:host.event.note")
            .child(
                TextArea::new()
                    .name(format!("events.{index}.note"))
                    .value(note)
                    .rows(3)
                    .placeholder("i18n:host.event.note.placeholder"),
            )
            .into(),
        // L'accès avant les conseils : c'est la question qu'on se pose avant d'y aller, et un
        // voyageur en fauteuil ne doit pas la chercher au milieu des bons plans de parking.
        named("access")
            .label("i18n:host.event.access")
            .child(
                TextArea::new()
                    .name(format!("events.{index}.access"))
                    .value(access)
                    .rows(2)
                    .placeholder("i18n:host.event.access.placeholder"),
            )
            .into(),
        FieldHint::new().text("i18n:host.event.access.hint").into(),
        Field::new()
            .name(format!("events.{index}.tips"))
            .label("i18n:host.event.tips")
            .child(
                TextArea::new()
                    .name(format!("events.{index}.tips"))
                    .value(tips)
                    .rows(3)
                    .placeholder("i18n:host.event.tips.placeholder"),
            )
            .into(),
        FieldHint::new().text("i18n:host.event.tips.hint").into(),
        Field::new()
            .name(format!("events.{index}.photo"))
            .label("i18n:host.event.photo")
            .child(
                ImageUpload::new()
                    .name(format!("events.{index}.photo"))
                    .value(photo),
            )
            .into(),
        Field::new()
            .name(format!("events.{index}.cancelled"))
            .label("i18n:host.event.cancelled")
            .child(
                Toggle::new()
                    .name(format!("events.{index}.cancelled"))
                    .checked(event.is_some_and(|e| e.cancelled)),
            )
            .into(),
    ];

    Stack::new()
        .id(format!("event-{index}"))
        .gap(10.0)
        .children(id.into_iter().chain(fields).collect())
        .into()
}
