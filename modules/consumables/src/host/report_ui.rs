//! Report list rows + mark-restocked form for host surfaces.

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Form, ListItem, Pill, Stack, Text};

use crate::commands::UpdateStatusArgs;
use crate::entities::ConsumableReport;
use crate::status;

fn status_tone(wire: &str) -> Tone {
    match wire {
        "restocked" => Tone::Success,
        _ => Tone::Warning,
    }
}

/// Visual row — label, level (+ note), status pill, relative age.
pub(crate) fn build_report_list_item(report: &ConsumableReport, locale: &str) -> Component {
    let level_label = level_label_plain(report.level.as_str(), locale);
    let subtitle = match report
        .note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(note) => format!("{level_label} · {note}"),
        None => level_label,
    };
    let when = format_relative_when(report.created_at, locale);
    let pill = Pill::new()
        .label(format!(
            "i18n:{}",
            status::status_label_key(report.status.as_str())
        ))
        .tone(status_tone(report.status.as_str()));

    Component::ListItem(
        ListItem::new()
            .title(report.item_label.clone())
            .subtitle(subtitle)
            .leading("package")
            .chevron(false)
            .child(pill)
            .child(Text::new().text(when).variant(TextVariant::Caption)),
    )
}

fn level_label_plain(wire: &str, locale: &str) -> String {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    match (wire, fr) {
        ("low", true) => "Bientôt vide".to_string(),
        ("low", false) => "Running low".to_string(),
        (_, true) => "Manque".to_string(),
        (_, false) => "Missing".to_string(),
    }
}

/// One-click mark restocked (only when still open).
pub(crate) fn build_restock_form(report: &ConsumableReport) -> Option<Component> {
    if report.status != status::DEFAULT {
        return None;
    }

    let action = crate::ids::module_id().command(
        crate::ids::UPDATE_STATUS,
        UpdateStatusArgs {
            report_id: report.id,
            status: "restocked".to_string(),
        },
    );

    Some(Component::Form(
        Form::new().child(
            Button::new()
                .label("i18n:host.main.markRestocked")
                .action(action),
        ),
    ))
}

/// Stack: list row + optional restock button.
pub(crate) fn build_report_block(report: &ConsumableReport, locale: &str) -> Component {
    let mut children = vec![build_report_list_item(report, locale)];
    if let Some(form) = build_restock_form(report) {
        children.push(form);
    }
    Component::Stack(Stack::new().gap(6.0).children(children))
}

fn format_relative_when(created_at: DateTime<Utc>, locale: &str) -> String {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let now = Utc::now();
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
