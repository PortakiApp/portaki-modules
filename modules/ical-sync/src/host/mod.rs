//! Host dashboard surfaces — the settings page (spec Calendriers §2) + stats, stay, tasks.
//!
//! Two `Section`s, the shell's cards: the calendars — a `DataTable` (1 to 10) whose row opens in a
//! dialog (`panel: "dialog"`), fields named `calendars.<i>.<key>` — then the import. The link is a
//! row secret: the table shows it masked, the dialog never sends it back (blank keeps it). A rule
//! broken shows under its field, the same one `publishReadiness` blocks on; no Save button — the
//! shell autosaves through `updateConfig`.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::booking_channel::{BookingChannel, ChannelSignal};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    ChoiceList, DataTable, Field, FieldHint, Form, InlineNotice, Page, Pill, SecretInput, Section,
    Stack, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::config::{CalendarFeed, MissingEmail, ModuleConfig, CALENDAR_SLOTS};
use crate::sync_state::{load_sync_state, SyncState, FEED_ERROR_RUNS};

mod stats;
mod stay;

pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;

/// Les calendriers ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_CALENDARS: &str = "newCalendars";

// « Conflit de dates », « Flux … injoignable depuis 2 j » dans À venir : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::DangerTriangle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Calendar
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let state = load_sync_state()?;
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    // Sous chaque champ : ce que bloque la publication, puis un flux que la synchro ne lit plus.
    let mut problems = config.problems();
    problems.extend(config.sync_problems(&state));
    let error_of = |name: &str| {
        problems
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, error)| error.get(&ctx.locale).to_string())
    };
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(calendars_section(&config, &state, &ctx, now, &error_of)?)
                .child(import_section(&config, &state, &ctx, now)?),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 Calendriers ──────────────────────────────────────────────────────────

fn calendars_section(
    config: &ModuleConfig,
    state: &SyncState,
    ctx: &HostContext,
    now: DateTime<Utc>,
    error_of: &dyn Fn(&str) -> Option<String>,
) -> Result<Component> {
    let rows = rows(config, ctx);
    let mut children: Vec<Component> =
        vec![FieldHint::new().text("i18n:host.calendars.help").into()];
    if let Some(error) = error_of("calendars") {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }
    let mut table_rows = Vec::new();
    for feed in &rows {
        table_rows.push(row(feed, state, now, &ctx.locale)?);
    }
    let mut table = DataTable::new()
        .rowName("calendars")
        .panel("dialog")
        .columns(json!([
            { "key": "name", "label": "i18n:host.calendar.unnamed", "sub": "link" },
            { "key": "state", "label": "i18n:host.calendar.state" },
            { "key": "platform", "label": "i18n:host.calendar.channel", "kind": "pill" },
        ]))
        .rows(Value::Array(table_rows))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, feed)| calendar_panel(index, feed, error_of))
                .collect(),
        );
    if rows.len() < CALENDAR_SLOTS {
        let mut added: Vec<String> = rows
            .iter()
            .filter(|feed| feed.url.is_empty())
            .map(|feed| feed.id.clone())
            .collect();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_CALENDARS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.calendars.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());
    Ok(Section::new()
        .title("i18n:host.calendars.label")
        .subtitle(format!("{} / {CALENDAR_SLOTS}", rows.len()))
        .children(children)
        .into())
}

/// Les lignes : les calendriers enregistrés, puis ceux qu'« Ajouter » a ouverts (sans lien).
fn rows(config: &ModuleConfig, ctx: &HostContext) -> Vec<CalendarFeed> {
    let mut rows = config.calendars.clone();
    for id in ctx.input_str(NEW_CALENDARS).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty() && rows.len() < CALENDAR_SLOTS && !rows.iter().any(|f| f.id == id) {
            rows.push(CalendarFeed {
                id: id.to_string(),
                url: String::new(),
                label: None,
                format: Default::default(),
                channel: BookingChannel::Unknown,
                channel_signal: ChannelSignal::None,
                active: true,
            });
        }
    }
    rows
}

/// Une ligne du tableau : nom (lien masqué dessous), état du flux, plateforme.
fn row(feed: &CalendarFeed, state: &SyncState, now: DateTime<Utc>, lang: &str) -> Result<Value> {
    let platform = feed
        .channel
        .is_identified()
        .then(|| format!("i18n:host.channel.{}", feed.channel.as_str()));
    let name = match (&feed.label, &platform) {
        (Some(label), _) => label.clone(),
        (None, Some(platform)) => platform.clone(),
        (None, None) => "i18n:host.calendar.unnamed".to_string(),
    };
    Ok(json!({
        "id": feed.id,
        "name": name,
        "link": if feed.url.is_empty() { String::new() } else { feed.masked_url() },
        "state": feed_state(feed, state, now, lang)?,
        "platform": platform.unwrap_or_default(),
    }))
}

/// « À jour · il y a 12 min », « Ne répond plus depuis 2 j », « Suspendu », « Pas encore relevé ».
fn feed_state(
    feed: &CalendarFeed,
    state: &SyncState,
    now: DateTime<Utc>,
    lang: &str,
) -> Result<String> {
    let parse = |raw: &str| {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|at| at.with_timezone(&Utc))
    };
    if !feed.active {
        return Ok("i18n:host.calendar.state.paused".into());
    }
    if let Some(failure) = state
        .feed_failures
        .get(&feed.id)
        .filter(|failure| failure.count >= FEED_ERROR_RUNS)
    {
        if let Some(since) = parse(&failure.since) {
            return t!(
                "host.calendar.state.down",
                since = time::elapsed(since, now, lang)
            );
        }
    }
    // Un relevé d'avant le suivi par calendrier : la dernière synchro réussie vaut pour tous.
    let read_at = state.feed_read_at.get(&feed.id).or_else(|| {
        state
            .feed_read_at
            .is_empty()
            .then_some(state.last_success_at.as_ref())
            .flatten()
    });
    match read_at.and_then(|raw| parse(raw)) {
        Some(at) => t!("host.calendar.state.ok", ago = time::ago(at, now, lang)),
        None => Ok("i18n:host.calendar.state.never".into()),
    }
}

/// La fenêtre d'une ligne : Lien du calendrier, Plateforme (lue), Nom, Actif (§2.1).
fn calendar_panel(
    index: usize,
    feed: &CalendarFeed,
    error_of: &dyn Fn(&str) -> Option<String>,
) -> Component {
    let name = |key: &str| format!("calendars.{index}.{key}");
    let field = |key: &str| {
        let field = Field::new().name(name(key));
        match error_of(&name(key)) {
            Some(error) => field.error(error),
            None => field,
        }
    };
    // Le lien enregistré n'est jamais renvoyé : son masque sert d'indication, vide le garde.
    let placeholder = if feed.url.is_empty() {
        "i18n:host.calendar.url.placeholder".to_string()
    } else {
        feed.masked_url()
    };
    let platform: Component = if feed.channel.is_identified() {
        let how = if feed.channel_signal == ChannelSignal::HostOverride {
            "i18n:host.calendar.channel.chosen"
        } else {
            "i18n:host.calendar.channel.detected"
        };
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(6.0)
            .children(vec![
                Pill::new()
                    .label(format!("i18n:host.channel.{}", feed.channel.as_str()))
                    .into(),
                Text::new().text(how).variant(TextVariant::Caption).into(),
            ])
            .into()
    } else {
        FieldHint::new()
            .text("i18n:host.calendar.channel.undetected")
            .into()
    };
    Stack::new()
        .gap(14.0)
        .children(vec![
            field("url")
                .label("i18n:host.calendar.url")
                .required(true)
                .child(
                    SecretInput::new()
                        .name(name("url"))
                        .value(String::new())
                        .placeholder(placeholder),
                )
                .child(FieldHint::new().text("i18n:host.calendar.url.help"))
                .into(),
            // Lue, pas saisie : pas de `name`, la ligne garde ce qu'elle avait.
            Field::new()
                .label("i18n:host.calendar.channel")
                .child(platform)
                .into(),
            // `required(false)` : le tableau de bord marque le champ « facultatif ».
            field("label")
                .label("i18n:host.calendar.label")
                .required(false)
                .child(
                    TextInput::new()
                        .name(name("label"))
                        .value(feed.label.clone().unwrap_or_default())
                        .placeholder("i18n:host.calendar.label.placeholder"),
                )
                .into(),
            ToggleRow::new()
                .name(name("active"))
                .label("i18n:host.calendar.active")
                .description("i18n:host.calendar.active.help")
                .checked(feed.active)
                .into(),
        ])
        .into()
}

// ── 2.2 Import ───────────────────────────────────────────────────────────────

fn import_section(
    config: &ModuleConfig,
    state: &SyncState,
    ctx: &HostContext,
    now: DateTime<Utc>,
) -> Result<Component> {
    // Le plan fixe la cadence ; le module n'en reçoit que le nom, jamais le nombre de passages.
    let plan = ctx.plan.display_name.trim();
    let frequency = if plan.is_empty() {
        "i18n:host.frequency.plan".to_string()
    } else {
        t!("host.frequency.named", plan = plan)?
    };
    let last = state
        .last_success_at
        .as_deref()
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|at| time::ago(at.with_timezone(&Utc), now, &ctx.locale));
    let last_line = match last {
        Some(ago) => t!(
            "host.import.last",
            ago = ago,
            count = state.uids.len().to_string()
        )?,
        None => "i18n:host.import.never".to_string(),
    };
    Ok(Section::new()
        .title("i18n:host.import.title")
        .child(
            Field::new()
                .label("i18n:host.frequency.label")
                .child(Text::new().text(frequency).variant(TextVariant::Body))
                .child(FieldHint::new().text(last_line)),
        )
        .child(
            Field::new()
                .name("missing_email")
                .label("i18n:host.missingEmail.label")
                .child(
                    ChoiceList::new()
                        .name("missing_email")
                        .layout(ChoiceListLayout::Segmented)
                        .value(config.missing_email.as_str())
                        .choices(
                            MissingEmail::ALL
                                .iter()
                                .map(|choice| {
                                    ChoiceOption::new(
                                        choice.as_str(),
                                        format!("i18n:host.missingEmail.label.{}", choice.as_str()),
                                    )
                                })
                                .collect(),
                        ),
                )
                .child(FieldHint::new().text("i18n:host.missingEmail.help")),
        )
        // Fermer les dates ailleurs demande un flux iCal sortant de Portaki : rien n'est enregistré.
        .child(
            Field::new()
                .label("i18n:host.blockDates.label")
                .child(Pill::new().label("i18n:host.soon"))
                .child(FieldHint::new().text("i18n:host.blockDates.help")),
        )
        .into())
}

#[cfg(test)]
mod tests {
    use portaki_test_utils::MockContext;

    use super::*;
    use crate::config::{feeds, CalendarRow};

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// À jour, ne répond plus, suspendu, pas encore relevé — et un relevé d'avant le suivi par
    /// calendrier vaut pour tous.
    #[test]
    #[serial_test::serial]
    fn each_calendar_says_how_it_is_read() {
        let row = |id: &str, active: bool| CalendarRow {
            id: id.into(),
            url: format!("https://www.airbnb.com/calendar/ical/{id}.ics"),
            active: Some(active.into()),
            ..CalendarRow::default()
        };
        let calendars = feeds(&[
            row("ok", true),
            row("down", true),
            row("off", false),
            row("new", true),
        ]);
        let mut state = SyncState::default();
        state.record_feed("ok", true, "2026-08-01T11:48:00Z");
        for _ in 0..FEED_ERROR_RUNS {
            state.record_feed("down", false, "2026-07-30T12:00:00Z");
        }
        let now = at("2026-08-01T12:00:00Z");
        MockContext::host()
            .with_translation("host.calendar.state.ok", "À jour · {ago}")
            .with_translation("host.calendar.state.down", "Ne répond plus depuis {since}")
            .run(|_| {
                let states: Vec<String> = calendars
                    .iter()
                    .map(|feed| feed_state(feed, &state, now, "fr").unwrap())
                    .collect();
                assert_eq!(
                    states,
                    [
                        "À jour · il y a 12 min",
                        "Ne répond plus depuis 2 j",
                        "i18n:host.calendar.state.paused",
                        "i18n:host.calendar.state.never",
                    ]
                );
                let legacy = SyncState {
                    last_success_at: Some("2026-08-01T11:48:00Z".into()),
                    ..SyncState::default()
                };
                assert_eq!(
                    feed_state(&calendars[3], &legacy, now, "fr").unwrap(),
                    "À jour · il y a 12 min"
                );
            });
    }
}
