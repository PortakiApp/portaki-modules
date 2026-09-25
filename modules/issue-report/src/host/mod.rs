//! Host surfaces — no config tab; the recent reports live in the stats detail.
//!
//! Rows are `FeedItem`s: category tag, status pill. A row opens the dashboard detail modal
//! (`host.surface.overlay`), which renders the stats detail again with its `issueId`: the
//! report, its follow-up and « Marquer comme résolu » (`resolve`).

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::files::FileRef;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, ChecklistItem, EmptyState, FeedItem, Image, KeyValue, Stack, Text,
};
use portaki_sdk::sdui::{FeedStatus, ImageSize, NavigateTarget};
use serde_json::json;

use portaki_sdk::host::time;

use crate::category;
use crate::commands::ResolveArgs;
use crate::entities::IssueReport;
use crate::storage;

mod stats;

pub use stats::{render_host_stats, stats_summary};

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
    let mut date = format_relative_when(report.created_at, now, locale);
    let (status, tone) = match report.resolved_at {
        None => ("host.main.status.open", Tone::Warning),
        Some(resolved_at) => {
            let delay = stats::format_delay(resolved_at - report.created_at, fr);
            let resolved = t!("host.main.resolvedIn", delay = delay).unwrap_or_default();
            date = format!("{date} · {resolved}");
            ("host.main.status.resolved", Tone::Success)
        }
    };
    let tag = format!("stats.category.{}", category::normalize(&report.category));
    // The modal header is plain text: resolved here, in the host's language.
    let open_detail = Action::emit(
        crate::ids::HOST_SURFACE_OVERLAY,
        Some(json!({
            "input": { "issueId": report.id },
            "icon": "danger-triangle",
            "kicker": t!("host.detail.kicker", category = t!(&tag).unwrap_or_default())
                .unwrap_or_default(),
            "title": report.summary,
            "status": { "label": t!(status).unwrap_or_default(), "tone": tone },
        })),
    );
    let mut row = FeedItem::new()
        .title(report.summary.clone())
        .tag(format!("i18n:{tag}"))
        .date(date)
        .dotTone(tone)
        .status(FeedStatus::new(format!("i18n:{status}"), tone))
        .action(open_detail);
    if report.photo.is_some() {
        row = row.meta("i18n:host.main.withPhoto");
    }
    row.into()
}

/// Body of the detail modal: what, when, the follow-up and the actions.
pub(crate) fn report_detail(report: &IssueReport, locale: &str) -> Component {
    let date = |at| format_short_date(at, locale);
    let category = format!(
        "i18n:stats.category.{}",
        category::normalize(&report.category)
    );
    let mut children: Vec<Component> = vec![
        KeyValue::new()
            .key("i18n:host.detail.category")
            .value(category)
            .into(),
        KeyValue::new()
            .key("i18n:host.detail.date")
            .value(date(report.created_at))
            .into(),
    ];
    if let Some(details) = &report.details {
        children.push(Text::new().text(details.clone()).into());
    }
    // The platform swaps the `portaki-file:` reference for a signed URL at render time.
    if let Some(photo) = report.photo.as_deref().and_then(FileRef::parse) {
        children.push(
            Image::new()
                .url(photo.image_url())
                .alt("i18n:host.detail.photoAlt")
                .size(ImageSize::Thumb)
                .into(),
        );
    }
    let resolved = match report.resolved_at {
        Some(at) => t!("host.detail.resolved", date = date(at)),
        None => t!("host.detail.toResolve"),
    };
    children.extend([
        Text::new()
            .text("i18n:host.detail.followUp")
            .variant(TextVariant::Title)
            .into(),
        ChecklistItem::new()
            .label(t!("host.detail.reported", date = date(report.created_at)).unwrap_or_default())
            .checked(true)
            .into(),
        ChecklistItem::new()
            .label(resolved.unwrap_or_default())
            .checked(report.resolved_at.is_some())
            .into(),
        Button::new()
            .label("i18n:host.detail.openStay")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(
                NavigateTarget::path(format!("/stays/{}", report.stay_id)),
                None,
            ))
            .into(),
        Button::new()
            .label("i18n:host.detail.message")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(NavigateTarget::path("/messages"), None))
            .into(),
    ]);
    if report.resolved_at.is_none() {
        children.push(
            Button::new()
                .label("i18n:host.detail.resolve")
                .action(crate::ids::module_id().command(
                    crate::commands::RESOLVE,
                    ResolveArgs {
                        report_id: report.id,
                    },
                ))
                .into(),
        );
    }
    Stack::new().gap(12.0).children(children).into()
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
