//! Host dashboard surface — design `editorIssueReport` / `issuereport-editor-v1`.
//!
//! No host config: info banner + recent reports with category icons and status pills.
//! Open/resolved is not persisted yet — all rows show « À traiter » (design open state).

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, EmptyState, InfoBanner, List, ListItem, Page, Pill, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::IssueReport;
use crate::storage;

/// Host main — banner + recent property reports (max 20).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let locale = ctx.locale.as_str();
    let reports = storage::list_recent().unwrap_or_default();

    let recent_body: Vec<Component> = if reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon("danger-triangle")
            .into()]
    } else {
        let items: Vec<Component> = reports
            .iter()
            .map(|report| build_report_row(report, locale))
            .collect();
        vec![Component::List(List::new().children(items))]
    };

    let children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        Card::new()
            .title("i18n:host.main.recentTitle")
            .subtitle("i18n:host.main.recentHelp")
            .icon("danger-triangle")
            .children(recent_body)
            .into(),
    ];

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
        .with_id(crate::ids::HOST_MAIN)
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

fn build_report_row(report: &IssueReport, locale: &str) -> Component {
    let label_key = category::category_label_key(report.category.as_str());
    let when = format_relative_when(report.created_at, locale);
    let pill = Pill::new()
        .label("i18n:host.main.status.open")
        .tone(Tone::Warning);

    Component::ListItem(
        ListItem::new()
            .title(report.summary.clone())
            .subtitle(format!("i18n:{label_key}"))
            .leading(category_icon(report.category.as_str()))
            .chevron(false)
            .child(pill)
            .child(Text::new().text(when).variant(TextVariant::Caption)),
    )
}

/// Compact relative age — design: « il y a 2 h », « hier », « 2 jours ».
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
