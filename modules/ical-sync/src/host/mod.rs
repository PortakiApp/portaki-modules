//! Host dashboard surfaces — config sheet + property stats card.

use portaki_sdk::contracts::booking_channel::BookingChannel;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, InfoBanner, Page, Select, Stack, StepList, Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, CalendarFeed, CalendarFormat, ModuleConfig, CALENDAR_SLOTS};

mod stats;

pub use stats::render_host_stats;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let calendars_count = draft_calendars_count(&ctx, &config);

    let last_sync = config
        .last_sync_at
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:host.status.never".to_string());
    let summary = config
        .sync_summary
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:host.status.emptySummary".to_string());

    let mut calendar_rows: Vec<Component> = Vec::new();
    for index in 0..calendars_count {
        let feed = config.calendars.get(index);
        calendar_rows.push(calendar_row(index, feed));
    }

    let form_children: Vec<Component> = vec![
        InfoBanner::new()
            .title("i18n:host.alert.title")
            .message("i18n:host.alert.message")
            .into(),
        Card::new()
            .title("i18n:host.section.feeds")
            .subtitle("i18n:host.section.feeds.help")
            .icon("calendar")
            .children(vec![StepList::new()
                .label("i18n:host.calendars.label")
                .hint("i18n:host.calendars.hint")
                .emptyTitle("i18n:host.calendars.emptyTitle")
                .emptyDescription("i18n:host.calendars.emptyDescription")
                .addLabel("i18n:host.calendars.add")
                .removeLabel("i18n:host.calendars.remove")
                .itemKeyPrefix("calendars")
                .addAction(emit_input(CalendarsCountInput {
                    calendars_count: (calendars_count + 1).min(CALENDAR_SLOTS),
                }))
                .children(calendar_rows)
                .into()])
            .into(),
        Card::new()
            .title("i18n:host.section.status")
            .subtitle("i18n:host.section.status.help")
            .icon("refresh")
            .children(vec![
                Field::new()
                    .name("last_sync_at")
                    .label("i18n:host.status.lastSync")
                    .child(Text::new().text(last_sync).variant(TextVariant::Body))
                    .into(),
                Field::new()
                    .name("sync_summary")
                    .label("i18n:host.status.summary")
                    .child(Text::new().text(summary).variant(TextVariant::Caption))
                    .into(),
            ])
            .into(),
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    ];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Surface::new(Page::new().child(Form::new().children(form_children)))
        .with_id(crate::ids::HOST_MAIN)
}

fn draft_calendars_count(ctx: &HostContext, config: &ModuleConfig) -> usize {
    if let Some(n) = ctx.input_u64("calendars_count") {
        return (n as usize).min(CALENDAR_SLOTS);
    }
    let existing = config.calendars.len();
    if existing == 0 {
        1
    } else {
        existing.min(CALENDAR_SLOTS)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct CalendarsCountInput {
    calendars_count: usize,
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn calendar_row(index: usize, feed: Option<&CalendarFeed>) -> Component {
    let id = feed
        .map(|f| f.id.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("cal-{}", index + 1));
    let url = feed.map(|f| f.url.as_str()).unwrap_or("");
    let label = feed.and_then(|f| f.label.as_deref()).unwrap_or("");
    let format = feed.map(|f| f.format).unwrap_or(CalendarFormat::Generic);
    let format_help = format_help_key(format);
    let channel = feed.map(|f| f.channel).unwrap_or_default();

    // Hidden id keeps stable feed ids across edits (StepList remove clears visible fields only).
    Stack::new()
        .id(format!("calendar-{index}"))
        .gap(10.0)
        .children(vec![
            TextInput::new()
                .name(format!("calendars.{index}.id"))
                .value(id)
                .into(),
            Field::new()
                .name(format!("calendars.{index}.format"))
                .label("i18n:host.calendar.format")
                .child(
                    Select::new()
                        .name(format!("calendars.{index}.format"))
                        .options(format_options())
                        .value(format.as_str()),
                )
                .into(),
            Text::new()
                .text(format_help)
                .variant(TextVariant::Caption)
                .into(),
            Field::new()
                .name(format!("calendars.{index}.channel"))
                .label("i18n:host.calendar.channel")
                .child(
                    Select::new()
                        .name(format!("calendars.{index}.channel"))
                        .options(channel_options())
                        .value(channel.as_str()),
                )
                .into(),
            Text::new()
                .text("i18n:host.calendar.channel.help")
                .variant(TextVariant::Caption)
                .into(),
            Field::new()
                .name(format!("calendars.{index}.label"))
                .label("i18n:host.calendar.label")
                .child(
                    TextInput::new()
                        .name(format!("calendars.{index}.label"))
                        .value(label)
                        .placeholder("i18n:host.calendar.label.placeholder"),
                )
                .into(),
            Field::new()
                .name(format!("calendars.{index}.url"))
                .label("i18n:host.calendar.url")
                .child(
                    TextInput::new()
                        .name(format!("calendars.{index}.url"))
                        .value(url)
                        .placeholder("i18n:host.calendar.url.placeholder"),
                )
                .into(),
        ])
        .into()
}

fn format_options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("airbnb", "i18n:host.format.airbnb")
            .description("i18n:host.format.airbnb.help"),
        ChoiceOption::new("booking", "i18n:host.format.booking")
            .description("i18n:host.format.booking.help"),
        ChoiceOption::new("abritel_vrbo", "i18n:host.format.abritel_vrbo")
            .description("i18n:host.format.abritel_vrbo.help"),
        ChoiceOption::new("google", "i18n:host.format.google")
            .description("i18n:host.format.google.help"),
        ChoiceOption::new("generic", "i18n:host.format.generic")
            .description("i18n:host.format.generic.help"),
    ]
}

/// Platform list comes straight from [`BookingChannel::ALL`] — the SDK owns the
/// vocabulary so this selector cannot drift from what the gateway parses.
fn channel_options() -> Vec<ChoiceOption> {
    BookingChannel::ALL
        .iter()
        .map(|channel| {
            let code = channel.as_str();
            ChoiceOption::new(code, format!("i18n:host.channel.{code}"))
                .description(format!("i18n:host.channel.{code}.help"))
        })
        .collect()
}

fn format_help_key(format: CalendarFormat) -> &'static str {
    match format {
        CalendarFormat::Airbnb => "i18n:host.format.airbnb.help",
        CalendarFormat::Booking => "i18n:host.format.booking.help",
        CalendarFormat::AbritelVrbo => "i18n:host.format.abritel_vrbo.help",
        CalendarFormat::Google => "i18n:host.format.google.help",
        CalendarFormat::Generic => "i18n:host.format.generic.help",
    }
}
