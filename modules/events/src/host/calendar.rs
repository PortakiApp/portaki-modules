//! L'onglet du logement (spec Événements §1, `property-workspace-tab`) : le calendrier des
//! événements saisis et importés, par mois, les doublons signalés (planche du 12 oct.).
//!
//! Vos événements à chaque occurrence du mois, ceux d'OpenAgenda à leur date. Filtre de source,
//! mois et événement ouvert passent par l'entrée de surface : un clic re-rend l'onglet.

use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, FilterChip, InlineNotice, KeyValue, MonthCalendar, Page, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Value};

use crate::config::{local, EventRow, ModuleConfig};
use crate::nearby::{host_now, is_duplicate, upcoming_nearby};

const MONTH: &str = "month";
const SOURCE: &str = "source";
const SELECTED: &str = "selected";
/// Tous, les vôtres, OpenAgenda — dans l'ordre des pastilles.
const SOURCES: [&str; 3] = ["all", "own", "openagenda"];
/// Un événement passé reste trente jours dans l'onglet, puis disparaît (planche).
const KEEP_PAST_DAYS: i64 = 30;

#[portaki_sdk::surface(
    host,
    id = "calendar",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.calendar",
    icon = IconName::Calendar
)]
pub fn render_host_calendar(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let (now, tz) = (host_now(), ctx.property_tz());
    let first = ctx
        .input_str(MONTH)
        .and_then(parse_month)
        .unwrap_or_else(|| local(now, tz.as_ref()).date().with_day(1).expect("day 1"));
    let next = first + Months::new(1);
    let last = next.pred_opt().expect("a month has a last day");
    let source = ctx
        .input_str(SOURCE)
        .filter(|s| SOURCES.contains(s))
        .unwrap_or(SOURCES[0]);

    let keep_from = now - Duration::days(KEEP_PAST_DAYS);
    let tz_ref = tz.as_ref();
    let own: Vec<Entry> = config
        .parse_events()
        .into_iter()
        .map(|e| crate::time_format::anchored(e, tz_ref))
        .flat_map(|event| {
            event
                .occurrences_in(first, last, tz_ref)
                .into_iter()
                .map(move |at| Entry::own(&event, at, tz_ref))
        })
        .filter(|entry| entry.ends() >= keep_from)
        .collect();
    let (imported, unreachable) = match upcoming_nearby(&ctx, &config) {
        Ok(found) => (found.unwrap_or_default(), false),
        Err(_) => (Vec::new(), true),
    };
    let imported: Vec<Entry> = imported
        .into_iter()
        .filter_map(|event| {
            let at = crate::time_format::parse_starts_at(&event.starts_at)?;
            let day = local(at, tz_ref).date();
            (first..=last).contains(&day).then(|| Entry {
                id: event.id.clone(),
                at,
                own: false,
                duplicate: own.iter().any(|o| is_duplicate(&o.event, &event)),
                event,
            })
        })
        .filter(|entry| entry.ends() >= keep_from)
        .collect();
    // Le vôtre est en doublon quand un import le double : il porte la pastille « Doublon ».
    let own: Vec<Entry> = own
        .into_iter()
        .map(|mut entry| {
            entry.duplicate = imported
                .iter()
                .any(|oa| is_duplicate(&entry.event, &oa.event));
            entry
        })
        .collect();

    let shown: Vec<&Entry> = own
        .iter()
        .filter(|_| source != "openagenda")
        .chain(
            imported
                .iter()
                .filter(|oa| source == "openagenda" || (source == "all" && !oa.duplicate)),
        )
        .collect();

    let mut children: Vec<Component> = vec![toolbar(&ctx, source)];
    if unreachable {
        children.push(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message("i18n:host.calendar.unreachable")
                .into(),
        );
    }
    if let Some(banner) = duplicates_banner(&own, &ctx) {
        children.push(banner);
    }
    children.push(
        Card::new()
            .child(calendar(first, &shown, now, tz_ref, &ctx))
            .into(),
    );
    children.push(
        Text::new()
            .text("i18n:host.calendar.hint")
            .variant(TextVariant::Caption)
            .into(),
    );
    let selected = ctx.input_str(SELECTED).unwrap_or_default();
    if let Some(entry) = own.iter().chain(&imported).find(|e| e.id == selected) {
        children.push(detail(&ctx, entry, now, tz_ref));
    }

    Ok(Surface::new(
        Page::new()
            .title("i18n:host.calendar.title")
            .child(Stack::new().gap(16.0).children(children)),
    )
    .with_id(CALENDAR))
}

/// Une case du calendrier : un événement à une occurrence.
struct Entry {
    /// `{id}@{AAAA-MM-JJ}` pour le vôtre, `oa-{uid}` (celui du connecteur) pour un import.
    id: String,
    /// L'événement porté à cette occurrence (`starts_at`, `ends_at`).
    event: EventRow,
    at: DateTime<Utc>,
    own: bool,
    duplicate: bool,
}

impl Entry {
    fn own(
        event: &EventRow,
        at: DateTime<Utc>,
        tz: Option<&portaki_sdk::host::time::PropertyTz>,
    ) -> Self {
        let mut moved = event.clone();
        let shift = crate::time_format::parse_starts_at(&event.starts_at).map(|s| at - s);
        moved.starts_at = at.to_rfc3339();
        moved.ends_at = event
            .ends_at
            .as_deref()
            .and_then(crate::time_format::parse_starts_at)
            .zip(shift)
            .map(|(ends, shift)| (ends + shift).to_rfc3339());
        Self {
            id: format!("{}@{}", event.id, local(at, tz).date()),
            event: moved,
            at,
            own: true,
            duplicate: false,
        }
    }

    /// Sa fin, sinon son début : passé quand elle est derrière nous.
    fn ends(&self) -> DateTime<Utc> {
        self.event
            .ends_at
            .as_deref()
            .and_then(crate::time_format::parse_starts_at)
            .unwrap_or(self.at)
    }

    fn title(&self, ctx: &HostContext) -> String {
        self.event.title.host_value(ctx).to_string()
    }
}

fn parse_month(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&format!("{}-01", raw.trim()), "%Y-%m-%d").ok()
}

fn input(pairs: Value) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(pairs))
}

/// Le filtre de source, puis « Réglages » : la fiche du module.
fn toolbar(ctx: &HostContext, source: &str) -> Component {
    let mut chips: Vec<Component> = SOURCES
        .iter()
        .map(|key| {
            FilterChip::new()
                .label(format!("i18n:host.calendar.source.{key}"))
                .selected(*key == source)
                .action(input(json!({ SOURCE: key, SELECTED: "" })))
                .into()
        })
        .collect();
    chips.push(
        Button::new()
            .label("i18n:host.calendar.settings")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(
                NavigateTarget::path(format!(
                    "/listings/{}/modules/{}",
                    ctx.property_id, ctx.module_id
                )),
                None,
            ))
            .into(),
    );
    Stack::new()
        .direction(StackDirection::Horizontal)
        .gap(8.0)
        .children(chips)
        .into()
}

/// « 1 doublon : « Concert sur la plage » est saisi et importé d'OpenAgenda. Le vôtre est gardé
/// dans le livret. »
fn duplicates_banner(own: &[Entry], ctx: &HostContext) -> Option<Component> {
    let mut names: Vec<String> = own
        .iter()
        .filter(|e| e.duplicate)
        .map(|e| e.title(ctx).trim().to_string())
        .collect();
    names.sort();
    names.dedup();
    if names.is_empty() {
        return None;
    }
    let quoted: Vec<String> = names
        .iter()
        .map(|name| {
            t!("host.calendar.quoted", name = name).unwrap_or_else(|_| format!("« {name} »"))
        })
        .collect();
    let key = if names.len() == 1 {
        "host.calendar.duplicates.one"
    } else {
        "host.calendar.duplicates.other"
    };
    let count = names.len().to_string();
    let names = quoted.join(", ");
    let message = t!(key, count = &count, names = &names).unwrap_or(names);
    Some(
        InlineNotice::new()
            .tone(Tone::Warning)
            .message(message)
            .into(),
    )
}

fn calendar(
    first: NaiveDate,
    shown: &[&Entry],
    now: DateTime<Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
    ctx: &HostContext,
) -> Component {
    let mut by_day: std::collections::BTreeMap<NaiveDate, Vec<&Entry>> = Default::default();
    for entry in shown {
        by_day
            .entry(local(entry.at, tz).date())
            .or_default()
            .push(entry);
    }
    let days = by_day
        .into_iter()
        .map(|(day, mut entries)| {
            entries.sort_by_key(|e| e.at);
            CalendarDay::new(
                day.format("%Y-%m-%d").to_string(),
                entries.into_iter().map(|e| chip(e, now, ctx)).collect(),
            )
        })
        .collect();
    let month = |date: NaiveDate| date.format("%Y-%m").to_string();
    MonthCalendar::new()
        .month(month(first))
        .days(days)
        .prevAction(input(
            json!({ MONTH: month(first - Months::new(1)), SELECTED: "" }),
        ))
        .nextAction(input(
            json!({ MONTH: month(first + Months::new(1)), SELECTED: "" }),
        ))
        .legend(vec![
            CalendarItem::new("", "i18n:host.calendar.legend.own"),
            CalendarItem::new("", "i18n:host.calendar.legend.imported").tone(Tone::Info),
            CalendarItem::new("", "i18n:host.calendar.legend.duplicate").tone(Tone::Warning),
            CalendarItem::new("", "i18n:host.calendar.legend.cancelled").strike(true),
            CalendarItem::new("", "i18n:host.calendar.legend.past").muted(true),
        ])
        .into()
}

fn chip(entry: &Entry, now: DateTime<Utc>, ctx: &HostContext) -> CalendarItem {
    let tone = match (entry.duplicate, entry.own) {
        (true, _) => Tone::Warning,
        (false, true) => Tone::Neutral,
        (false, false) => Tone::Info,
    };
    let label = entry.title(ctx).trim().to_string();
    let mut item = CalendarItem::new(&entry.id, label)
        .tone(tone)
        .action(input(json!({ SELECTED: entry.id })));
    if entry.event.cancelled {
        item = item.strike(true);
    }
    if entry.ends() < now {
        item = item.muted(true);
    }
    item
}

/// Ce qu'on a ouvert : le vôtre se modifie dans la fiche, un import se lit seulement.
fn detail(
    ctx: &HostContext,
    entry: &Entry,
    now: DateTime<Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
) -> Component {
    let at = local(entry.at, tz);
    let when = if entry.event.all_day {
        at.format("%d/%m/%Y").to_string()
    } else {
        format!("{} · {}", at.format("%d/%m/%Y"), at.format("%H:%M"))
    };
    let place = match entry.event.place.host_value(ctx).trim() {
        "" => entry.event.address.clone().unwrap_or_default(),
        place => place.to_string(),
    };
    let status = if entry.event.cancelled {
        Some("i18n:host.calendar.legend.cancelled")
    } else if entry.ends() < now {
        Some("i18n:host.calendar.legend.past")
    } else if entry.duplicate {
        Some("i18n:host.calendar.legend.duplicate")
    } else {
        None
    };
    let mut rows: Vec<Component> = vec![
        KeyValue::new()
            .key("i18n:host.calendar.detail.when")
            .value(when)
            .mono(true)
            .into(),
        KeyValue::new()
            .key("i18n:host.calendar.detail.source")
            .value(if entry.own {
                "i18n:host.calendar.legend.own"
            } else {
                "i18n:host.calendar.legend.imported"
            })
            .into(),
    ];
    if !place.is_empty() {
        rows.push(
            KeyValue::new()
                .key("i18n:host.calendar.detail.place")
                .value(place)
                .into(),
        );
    }
    if let Some(status) = status {
        rows.push(
            KeyValue::new()
                .key("i18n:host.calendar.detail.status")
                .value(status)
                .into(),
        );
    }
    let action = if entry.own {
        Some(
            Button::new()
                .label("i18n:host.calendar.detail.edit")
                .action(Action::navigate(
                    NavigateTarget::path(format!(
                        "/listings/{}/modules/{}",
                        ctx.property_id, ctx.module_id
                    )),
                    None,
                )),
        )
    } else {
        entry.event.url.as_deref().map(|url| {
            Button::new()
                .label("i18n:host.calendar.detail.openAgenda")
                .variant(ButtonVariant::Outline)
                .action(Action::external(url))
        })
    };
    if let Some(button) = action {
        rows.push(button.into());
    }
    Card::new().title(entry.title(ctx)).children(rows).into()
}
