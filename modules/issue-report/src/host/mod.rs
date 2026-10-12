//! Host surfaces — the settings drawer, the stats, the stay encart and its « Ajouter » action;
//! the recent reports live in the stats detail.
//!
//! Rows are `FeedItem`s: category tag, status pill. A row opens the dashboard detail modal
//! (`host.surface.overlay`), which renders the same surface again with its `issueId`: the
//! report, its status (`setStatus`), the reply to the guest (`reply`) and its history.

use chrono::{DateTime, Datelike, FixedOffset, Utc};
use portaki_sdk::files::FileRef;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, FeedItem, Field, FieldHint, FilterBar, FilterChip, Form, Image,
    Stack, Text, TextArea,
};
use portaki_sdk::sdui::{FeedStatus, NavigateTarget};
use serde_json::json;
use uuid::Uuid;

use portaki_sdk::host::time;

use crate::category;
use crate::commands::SetStatusArgs;
use crate::entities::IssueReport;
use crate::{status, storage};

mod add;
mod main;
mod reports;
mod stats;
mod stay;

pub use add::render_host_add;
pub use main::render_host_main;
pub use reports::render_host_reports;
pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
pub(crate) fn host_now() -> DateTime<Utc> {
    time::now().unwrap_or_else(|_| DateTime::<Utc>::from_timestamp(0, 0).expect("epoch is valid"))
}

/// « Signalements récents » — the property's latest reports (max 20).
pub(crate) fn recent_reports_card(locale: &str) -> Component {
    let reports = storage::list_recent().unwrap_or_default();

    let body: Component = if reports.is_empty() {
        EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon(IconName::DangerTriangle)
            .into()
    } else {
        let now = host_now();
        Stack::new()
            .gap(0.0)
            .children(
                reports
                    .iter()
                    .map(|report| report_row(report, now, locale))
                    .collect(),
            )
            .into()
    };

    Card::new()
        .title("i18n:host.main.recentTitle")
        .subtitle("i18n:host.main.recentHelp")
        .child(body)
        .into()
}

fn report_row(report: &IssueReport, now: DateTime<Utc>, locale: &str) -> Component {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let when = format_relative_when(report.created_at, now, locale);
    // « Marie Dupont · il y a 3 h » : le voyageur quand on le connaît.
    let mut date = match report.guest_name.trim() {
        "" => when.clone(),
        name => format!("{name} · {when}"),
    };
    let current = status::of(report);
    if let Some(resolved_at) = report.resolved_at {
        let delay = stats::format_delay(resolved_at - report.created_at, fr);
        let resolved = t!("host.main.resolvedIn", delay = delay).unwrap_or_default();
        date = format!("{date} · {resolved}");
    }
    let tone = status::tone(current);
    let status_key = status::label_key(current);
    let tag = category_key(report);
    // The modal header is plain text: resolved here, in the host's language.
    let open_detail = Action::emit(
        crate::ids::HOST_SURFACE_OVERLAY,
        Some(json!({
            "input": { "issueId": report.id },
            "icon": "danger-triangle",
            "kicker": t!("host.detail.kicker", category = t!(&tag).unwrap_or_default(), when = when)
                .unwrap_or_default(),
            "title": report.summary,
            "status": { "label": t!(&status_key).unwrap_or_default(), "tone": tone },
        })),
    );
    let mut row = FeedItem::new()
        .title(report.summary.clone())
        .tag(format!("i18n:{tag}"))
        .date(date)
        .dotTone(tone)
        .status(FeedStatus::new(format!("i18n:{status_key}"), tone))
        .action(open_detail);
    if report.photo.is_some() {
        row = row.meta("i18n:host.main.withPhoto");
    }
    row.into()
}

/// The short category label of the dashboard (« Appareil », « Wi-Fi »).
fn category_key(report: &IssueReport) -> String {
    format!("host.category.{}", category::normalize(&report.category))
}

/// The detail of `input.issueId`, when the surface is rendered in the dashboard's detail modal.
pub(crate) fn detail_from_input(ctx: &HostContext) -> Option<Component> {
    let report = ctx
        .input_str("issueId")
        .and_then(|id| Uuid::parse_str(id).ok())
        .and_then(|id| storage::find_by_id(id).ok().flatten())?;
    Some(report_detail(&report, ctx))
}

/// Body of the detail modal (spec Signaler §1): the status, what the guest wrote, the stay, the
/// reply to the guest and the history. Category, age and title are in the modal header.
pub(crate) fn report_detail(report: &IssueReport, ctx: &HostContext) -> Component {
    let current = status::of(report);
    let status_chips = FilterBar::new().children(
        status::ALL
            .iter()
            .map(|to| {
                FilterChip::new()
                    .label(format!("i18n:{}", status::label_key(to)))
                    .selected(*to == current)
                    .action(crate::ids::module_id().command(
                        crate::commands::SET_STATUS,
                        SetStatusArgs {
                            report_id: report.id,
                            status: (*to).to_string(),
                        },
                    ))
                    .into()
            })
            .collect(),
    );
    let mut children: Vec<Component> = vec![status_chips.into()];
    if let Some(details) = &report.details {
        children.push(Text::new().text(details.clone()).into());
    }
    // The platform swaps the `portaki-file:` reference for a signed URL at render time.
    if let Some(photo) = report.photo.as_deref().and_then(FileRef::parse) {
        children.push(
            Image::new()
                .url(photo.image_url())
                .alt("i18n:host.detail.photoAlt")
                .into(),
        );
    }
    children.push(stay_line(report, ctx));
    children.push(reply_form(report));
    children.push(history(report, ctx));
    Stack::new().gap(16.0).children(children).into()
}

/// « Marie Dupont · 24/08 → 29/08 », and « Ouvrir le séjour ».
fn stay_line(report: &IssueReport, ctx: &HostContext) -> Component {
    let mut parts: Vec<String> = Vec::new();
    if !report.guest_name.trim().is_empty() {
        parts.push(report.guest_name.trim().to_string());
    }
    if let (Some(from), Some(to)) = (report.stay_check_in, report.stay_check_out) {
        let day = |at| local(at, ctx).format("%d/%m").to_string();
        parts.push(format!("{} → {}", day(from), day(to)));
    }
    let mut line = Stack::new().direction(StackDirection::Horizontal).gap(12.0);
    if !parts.is_empty() {
        line = line.child(
            Text::new()
                .text(parts.join(" · "))
                .variant(TextVariant::Caption),
        );
    }
    line.child(
        Button::new()
            .label("i18n:host.detail.openStay")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(
                NavigateTarget::path(format!("/stays/{}", report.stay_id)),
                None,
            )),
    )
    .into()
}

/// « Répondre à Marie » : e-mailed to the guest and shown in their booklet history.
fn reply_form(report: &IssueReport) -> Component {
    let label = match report.guest_name.trim() {
        "" => t!("host.reply.label"),
        name => t!("host.reply.labelNamed", name = name.to_string()),
    };
    Form::new()
        .child(
            Field::new()
                .name("reply")
                .label(label.unwrap_or_default())
                .child(
                    TextArea::new()
                        .name("reply")
                        .rows(3)
                        .placeholder("i18n:host.reply.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.reply.hint")),
        )
        .child(
            Button::new().label("i18n:host.reply.send").action(
                crate::ids::module_id()
                    .command(crate::commands::REPLY, json!({ "reportId": report.id })),
            ),
        )
        .into()
}

/// « Historique » : sent, status changes, replies — oldest first.
fn history(report: &IssueReport, ctx: &HostContext) -> Component {
    let events = storage::events_for_report(report.id).unwrap_or_default();
    let mut lines: Vec<(DateTime<Utc>, String)> = Vec::new();
    // Rows sent before the history existed: their sending, and their resolution.
    if !events.iter().any(|event| event.kind == "created") {
        lines.push((
            report.created_at,
            t!("host.history.created").unwrap_or_default(),
        ));
    }
    if let Some(resolved_at) = report
        .resolved_at
        .filter(|_| !events.iter().any(|event| event.kind == "status"))
    {
        lines.push((resolved_at, status_line(status::RESOLVED)));
    }
    for event in &events {
        let line = match event.kind.as_str() {
            "created" if event.actor == "host" => t!("host.history.added"),
            "created" if report.photo.is_some() => t!("host.history.reportedWithPhoto"),
            "created" => t!("host.history.reported"),
            "status" => Ok(status_line(&event.status)),
            _ => t!(
                "host.history.reply",
                text = event.text.clone().unwrap_or_default()
            ),
        };
        lines.push((event.created_at, line.unwrap_or_default()));
    }
    lines.sort_by_key(|(at, _)| *at);
    let mut list = Stack::new().gap(6.0).child(
        Text::new()
            .text("i18n:host.history.title")
            .variant(TextVariant::Title),
    );
    for (at, line) in lines {
        let when = local(at, ctx).format("%d/%m %H:%M");
        list = list.child(
            Text::new()
                .text(format!("{when} · {line}"))
                .variant(TextVariant::Caption),
        );
    }
    list.into()
}

fn status_line(to: &str) -> String {
    let label = t!(&status::label_key(to)).unwrap_or_default();
    t!("host.history.status", status = label).unwrap_or_default()
}

/// `at` on the property's clock; UTC when its timezone is unknown.
fn local(at: DateTime<Utc>, ctx: &HostContext) -> DateTime<FixedOffset> {
    match ctx.property_tz() {
        Some(tz) => tz.to_local(at),
        None => at.fixed_offset(),
    }
}

/// Compact relative age — design: « il y a 2 h », « hier », « 2 jours ».
fn format_relative_when(created_at: DateTime<Utc>, now: DateTime<Utc>, locale: &str) -> String {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let hours = (now - created_at).num_hours().max(0);
    let days = (now - created_at).num_days().max(0);

    if hours < 1 {
        return if fr {
            "à l'instant".to_string()
        } else {
            "just now".to_string()
        };
    }
    if hours < 24 {
        return if fr {
            format!("il y a {hours} h")
        } else {
            format!("{hours}h ago")
        };
    }
    if days == 1 {
        return if fr {
            "hier".to_string()
        } else {
            "yesterday".to_string()
        };
    }
    if days < 7 {
        return if fr {
            format!("{days} jours")
        } else {
            format!("{days}d")
        };
    }
    format_short_date(created_at, locale)
}

fn format_short_date(created_at: DateTime<Utc>, locale: &str) -> String {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let month = if fr {
        match created_at.month() {
            1 => "janv.",
            2 => "févr.",
            3 => "mars",
            4 => "avr.",
            5 => "mai",
            6 => "juin",
            7 => "juil.",
            8 => "août",
            9 => "sept.",
            10 => "oct.",
            11 => "nov.",
            12 => "déc.",
            _ => "",
        }
    } else {
        match created_at.month() {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        }
    };
    format!("{} {}", created_at.day(), month)
}
