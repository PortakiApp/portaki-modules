//! Report list rows + mark-restocked form for host surfaces.

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Field, Form, ListItem, Pill, Stack, Text, TextArea};

use crate::commands::UpdateStatusArgs;
use crate::entities::ConsumableReport;
use crate::status;

fn status_tone(wire: &str) -> Tone {
    match wire {
        status::RESTOCKED => Tone::Success,
        status::PLANNED => Tone::Primary,
        _ => Tone::Warning,
    }
}

/// Visual row — label, level (+ note), status pill, relative age.
pub(crate) fn build_report_list_item(
    report: &ConsumableReport,
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
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
    let when = format_relative_when(report.created_at, now, locale);
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
            .leading(Leading::Icon("package".into()))
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

fn status_action(report: &ConsumableReport, next: &str) -> Action {
    // La réponse vient du champ du formulaire, que la coquille fusionne au clic.
    crate::ids::module_id().command(
        crate::commands::UPDATE_STATUS,
        UpdateStatusArgs {
            report_id: report.id,
            status: next.to_string(),
            host_reply: None,
        },
    )
}

/// Réponse au voyageur + « Prévu » (tant qu'à traiter) + « Marquer livré » (tant que pas livré).
pub(crate) fn build_restock_form(report: &ConsumableReport) -> Option<Component> {
    if !status::is_pending(&report.status) {
        return None;
    }

    let mut form = Form::new().child(
        Field::new()
            .name(REPLY_FIELD)
            .label("i18n:host.main.reply")
            .child(
                TextArea::new()
                    .name(REPLY_FIELD)
                    .value(report.host_reply.clone().unwrap_or_default())
                    .placeholder("i18n:host.main.reply.placeholder"),
            ),
    );
    let mut buttons = Stack::new().direction(StackDirection::Horizontal).gap(8.0);
    if report.status == status::DEFAULT {
        buttons = buttons.child(
            Button::new()
                .label("i18n:host.main.markPlanned")
                .variant(ButtonVariant::Outline)
                .action(status_action(report, status::PLANNED)),
        );
    }
    form = form.child(
        buttons.child(
            Button::new()
                .label("i18n:host.main.markRestocked")
                .action(status_action(report, status::RESTOCKED)),
        ),
    );
    Some(Component::Form(form))
}

/// Le nom du champ tel que `UpdateStatusArgs` le lit sur le fil (camelCase).
const REPLY_FIELD: &str = "hostReply";

/// Stack: list row + optional restock button.
pub(crate) fn build_report_block(
    report: &ConsumableReport,
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let mut children = vec![build_report_list_item(report, now, locale)];
    if let Some(form) = build_restock_form(report) {
        children.push(form);
    }
    Component::Stack(Stack::new().gap(6.0).children(children))
}

/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
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
