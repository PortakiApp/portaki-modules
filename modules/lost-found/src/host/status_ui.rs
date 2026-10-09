//! Status pills / Select options and recent-report row builders.

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Field, Form, ListItem, Pill, Select, Text};

use crate::commands::UpdateStatusArgs;
use crate::description;
use crate::entities::LostFoundReport;
use crate::status;

/// Select options for host status updates: the current status, then where it can go.
pub(crate) fn status_choice_options(current: &str) -> Vec<ChoiceOption> {
    std::iter::once(current)
        .chain(status::next(current).iter().copied())
        .map(|wire| ChoiceOption::new(wire, status_label_i18n(wire)))
        .collect()
}

pub(crate) fn status_tone(wire: &str) -> Tone {
    match wire {
        "declared" => Tone::Warning,
        "found" => Tone::Info,
        "not_found" => Tone::Neutral,
        _ => Tone::Success,
    }
}

pub(crate) fn status_label_i18n(wire: &str) -> String {
    format!("i18n:status.{wire}")
}

pub(crate) fn report_title(report: &LostFoundReport) -> String {
    let title = description::to_plain_text(&report.item_description);
    if title.is_empty() {
        report.item_description.clone()
    } else {
        title
    }
}

/// Source chip in the subtitle — resolved in-module (subtitle is not an i18n key).
pub(crate) fn source_label(kind: &str, locale: &str) -> &'static str {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    if kind == "found" {
        if fr {
            "Hôte"
        } else {
            "Host"
        }
    } else if fr {
        "Voyageur"
    } else {
        "Guest"
    }
}

fn month_abbr(month: u32, locale: &str) -> &'static str {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    if fr {
        match month {
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
        match month {
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
    }
}

pub(crate) fn format_short_date(created_at: DateTime<Utc>, locale: &str) -> String {
    format!(
        "{} {}",
        created_at.day(),
        month_abbr(created_at.month(), locale)
    )
}

/// Compact relative age — design: « 2 jours », « 3 sem. », « 1 mois ».
///
/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
pub(crate) fn format_relative_when(
    created_at: DateTime<Utc>,
    now: DateTime<Utc>,
    locale: &str,
) -> String {
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let days = (now - created_at).num_days().max(0);

    if days < 1 {
        return if fr {
            "auj.".to_string()
        } else {
            "today".to_string()
        };
    }
    if days < 7 {
        return if fr {
            format!("{days} j")
        } else {
            format!("{days}d")
        };
    }
    if days < 30 {
        let weeks = (days / 7).max(1);
        return if fr {
            format!("{weeks} sem.")
        } else {
            format!("{weeks} wk")
        };
    }
    let months = (days / 30).max(1);
    if fr {
        format!("{months} mois")
    } else {
        format!("{months} mo")
    }
}

fn report_subtitle(report: &LostFoundReport, locale: &str) -> String {
    format!(
        "{} · {}",
        source_label(report.kind.as_str(), locale),
        format_short_date(report.created_at, locale)
    )
}

/// Visual row — title, source · date, status pill, relative age (design list).
pub(crate) fn build_report_list_item(
    report: &LostFoundReport,
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let title = report_title(report);
    let subtitle = report_subtitle(report, locale);
    let when = format_relative_when(report.created_at, now, locale);
    let pill = Pill::new()
        .label(status_label_i18n(report.status.as_str()))
        .tone(status_tone(report.status.as_str()));

    Component::ListItem(
        ListItem::new()
            .title(title)
            .subtitle(subtitle)
            .leading(Leading::Icon("search".into()))
            .chevron(false)
            .child(pill)
            .child(Text::new().text(when).variant(TextVariant::Caption)),
    )
}

/// Compact status update form under a report row (SDUI has no clickable pill).
pub(crate) fn build_status_update_form(report: &LostFoundReport) -> Component {
    let update_action = crate::ids::module_id().command(
        crate::commands::UPDATE_STATUS,
        UpdateStatusArgs {
            report_id: report.id,
            status: report.status.clone(),
        },
    );

    Component::Form(
        Form::new()
            .child(
                Field::new()
                    .name("status")
                    .label("i18n:host.main.status.label")
                    .child(
                        Select::new()
                            .name("status")
                            .options(status_choice_options(&report.status))
                            .value(report.status.as_str()),
                    ),
            )
            .child(
                Button::new()
                    .label("i18n:host.main.updateStatus")
                    .action(update_action),
            ),
    )
}

/// Stack: design list row, what the guest asked for (and where to ship), then the status
/// Select — left out once the item is back with its owner: there is nowhere left to go.
pub(crate) fn build_report_block(
    report: &LostFoundReport,
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let mut children = vec![build_report_list_item(report, now, locale)];
    if let Some(choice) = report.return_choice.as_deref() {
        children.push(
            Text::new()
                .text(format!("i18n:host.item.choice.{choice}"))
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    if let Some(address) = report.return_address.as_deref() {
        children.push(
            Text::new()
                .text(address.to_string())
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    if !status::next(&report.status).is_empty() {
        children.push(build_status_update_form(report));
    }
    Component::Stack(
        portaki_sdk::sdui::primitives::Stack::new()
            .gap(6.0)
            .children(children),
    )
}
