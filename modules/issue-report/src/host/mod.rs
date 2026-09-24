//! Host dashboard — no config tab; the recent reports live under the property stats tab.
//!
//! Rows carry category icons and status pills; open ones a « Marquer comme résolu » button
//! (`resolve` command).

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::files::FileRef;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ImageSize, Tone};
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Image, List, ListItem, Pill, Stack, Text,
};

use portaki_sdk::host::time;

use crate::category;
use crate::commands::ResolveArgs;
use crate::entities::IssueReport;
use crate::storage;

mod stats;

pub use stats::render_host_stats;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
pub(crate) fn host_now() -> DateTime<Utc> {
    time::now().unwrap_or_else(|_| DateTime::<Utc>::from_timestamp(0, 0).expect("epoch is valid"))
}

/// « Signalements récents » — the property's latest reports (max 20).
pub(crate) fn recent_reports_card(locale: &str) -> Component {
    let reports = storage::list_recent().unwrap_or_default();

    let recent_body: Vec<Component> = if reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon("danger-triangle")
            .into()]
    } else {
        let now = host_now();
        let items: Vec<Component> = reports
            .iter()
            .map(|report| build_report_row(report, now, locale))
            .collect();
        vec![Component::List(List::new().children(items))]
    };

    Card::new()
        .title("i18n:host.main.recentTitle")
        .subtitle("i18n:host.main.recentHelp")
        .icon("danger-triangle")
        .children(recent_body)
        .into()
}

fn category_icon(wire: &str) -> &'static str {
    match wire {
        "appliance" => "plug",
        "cleanliness" => "sparkles",
        "noise" => "bell",
        "access" => "key",
        _ => "info-circle",
    }
}

fn build_report_row(report: &IssueReport, now: DateTime<Utc>, locale: &str) -> Component {
    let label_key = category::category_label_key(report.category.as_str());
    let when = format_relative_when(report.created_at, now, locale);
    let open = report.resolved_at.is_none();
    let pill = if open {
        Pill::new()
            .label("i18n:host.main.status.open")
            .tone(Tone::Warning)
    } else {
        Pill::new()
            .label("i18n:host.main.status.resolved")
            .tone(Tone::Neutral)
    };

    let row: Component = ListItem::new()
        .title(report.summary.clone())
        .subtitle(format!("i18n:{label_key}"))
        .leading(category_icon(report.category.as_str()))
        .chevron(false)
        .child(pill)
        .child(Text::new().text(when).variant(TextVariant::Caption))
        .into();
    // The platform swaps the `portaki-file:` reference for a signed URL at render time.
    let photo: Option<Component> = report
        .photo
        .as_deref()
        .and_then(FileRef::parse)
        .map(|photo| {
            Image::new()
                .url(photo.image_url())
                .alt("i18n:host.main.photoAlt")
                .size(ImageSize::Thumb)
                .into()
        });
    if !open && photo.is_none() {
        return row;
    }
    let mut children = vec![row];
    children.extend(photo);
    if !open {
        return Stack::new().gap(6.0).children(children).into();
    }

    let resolve = crate::ids::module_id().command(
        crate::ids::RESOLVE,
        ResolveArgs {
            report_id: report.id,
        },
    );
    children.push(
        Button::new()
            .label("i18n:host.main.resolve")
            .variant(ButtonVariant::Outline)
            .action(resolve)
            .into(),
    );
    Stack::new().gap(6.0).children(children).into()
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
