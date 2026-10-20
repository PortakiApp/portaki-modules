//! Report list rows + mark-restocked form for host surfaces.

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::ids::OperationName;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::Choice;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Field, Form, ListItem, Pill, Stack, Text, TextArea};
use serde_json::json;

use crate::domain::request::Asked;
use crate::{Entry, Status};

fn status_tone(status: Status) -> Tone {
    match status {
        Status::Restocked => Tone::Success,
        Status::Planned => Tone::Primary,
        Status::Open => Tone::Warning,
    }
}

/// Visual row — label, level (+ note), status pill, relative age.
pub(crate) fn build_report_list_item(
    report: &Entry,
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let level_label = level_label_plain(report.item.level.wire(), locale);
    let mut subtitle = match report.guest_name.trim() {
        "" => level_label,
        guest => format!("{guest} · {level_label}"),
    };
    if let Some(note) = report.note() {
        subtitle = format!("{subtitle} · {note}");
    }
    let when = format_relative_when(report.created_at, now, locale);
    let pill = Pill::new()
        .label(report.status.label_key())
        .tone(status_tone(report.status));

    Component::ListItem(
        ListItem::new()
            .title(report.item.item_label.clone())
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

fn status_button(report: &Entry, label: &str, to: Status) -> Button {
    Button::new()
        .label(label)
        .action(crate::ids::module_id().command(
            OperationName::new("setStatus"),
            json!({ "id": report.id, "status": to.wire() }),
        ))
}

/// La réponse au voyageur (« Répondre » l'envoie par e-mail sans changer le statut), puis
/// « Prévu » (tant qu'à traiter) et « Marquer livré » (tant que pas livré).
pub(crate) fn build_restock_form(report: &Entry) -> Option<Component> {
    if !report.pending() {
        return None;
    }
    let mut form = Form::new();
    if let Some(last) = report.notes().last() {
        let said = t!("home.card.reply", reply = last.to_string()).unwrap_or_default();
        form = form.child(Text::new().text(said).variant(TextVariant::Caption));
    }
    let reply =
        crate::ids::module_id().command(OperationName::new("note"), json!({ "id": report.id }));
    form = form
        .child(
            Field::new()
                .name("text")
                .label("i18n:host.main.reply")
                .child(
                    TextArea::new()
                        .name("text")
                        .placeholder("i18n:host.main.reply.placeholder"),
                ),
        )
        .child(
            Button::new()
                .label("i18n:host.main.sendReply")
                .variant(ButtonVariant::Outline)
                .action(reply),
        );
    let mut buttons = Stack::new().direction(StackDirection::Horizontal).gap(8.0);
    if report.status == Status::Open {
        buttons = buttons.child(
            status_button(report, "i18n:host.main.markPlanned", Status::Planned)
                .variant(ButtonVariant::Outline),
        );
    }
    buttons = buttons.child(status_button(
        report,
        "i18n:host.main.markRestocked",
        Status::Restocked,
    ));
    Some(Stack::new().gap(8.0).child(form).child(buttons).into())
}

/// Stack: list row + optional restock button.
pub(crate) fn build_report_block(report: &Entry, now: DateTime<Utc>, locale: &str) -> Component {
    let mut children = vec![build_report_list_item(report, now, locale)];
    if let Some(form) = build_restock_form(report) {
        children.push(form);
    }
    Component::Stack(Stack::new().gap(6.0).children(children))
}

/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
pub(crate) fn format_relative_when(
    created_at: DateTime<Utc>,
    now: DateTime<Utc>,
    locale: &str,
) -> String {
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
    short_date(created_at, locale)
}

pub(crate) fn short_date(created_at: DateTime<Utc>, locale: &str) -> String {
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
