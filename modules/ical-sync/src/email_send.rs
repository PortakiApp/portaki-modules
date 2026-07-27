//! Module-owned host transactional emails via `host::email::send`.

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::CalendarFormat;
use crate::email_i18n;
use crate::ics::StayImportRow;
use crate::sync_state::SyncDiff;

const FR_MONTHS: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];
const EN_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Feed fetch failed (empty / unreachable body) → notify host.
pub fn notify_sync_failed(
    property_id: Uuid,
    property_name: &str,
    feed_id: &str,
    source_label: &str,
    last_success_at: Option<&str>,
    day_key: &str,
) -> Result<()> {
    let last_success_fr = last_success_at
        .map(format_instant_fr)
        .unwrap_or_else(|| email_i18n::text("email.syncFailed.lastSuccess.never").fr);
    let last_success_en = last_success_at
        .map(format_instant_en)
        .unwrap_or_else(|| email_i18n::text("email.syncFailed.lastSuccess.never").en);
    let error = email_i18n::text("email.syncFailed.error.empty");

    let vars_fr = [
        ("property", property_name),
        ("source", source_label),
        ("lastSuccess", last_success_fr.as_str()),
        ("error", error.fr.as_str()),
    ];
    let vars_en = [
        ("property", property_name),
        ("source", source_label),
        ("lastSuccess", last_success_en.as_str()),
        ("error", error.en.as_str()),
    ];

    email::send(&SendEmailArgs {
        email_id: format!("sync-failed-{feed_id}-{day_key}"),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: email_i18n::text("email.syncFailed.subject"),
            eyebrow: Some(email_i18n::text("email.syncFailed.eyebrow")),
            title: Some(email_i18n::text("email.syncFailed.title")),
            body: localized_with_pairs("email.syncFailed.body", &vars_fr, &vars_en),
            cta: Some(ModuleEmailCta {
                label: email_i18n::text("email.syncFailed.cta"),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id: None,
        property_id: Some(property_id),
        action_url: None,
    })
}

/// Single new stay imported without guest email → invite host to complete it.
pub fn notify_stay_imported(
    property_id: Uuid,
    property_name: &str,
    source_label: &str,
    row: &StayImportRow,
) -> Result<()> {
    let dates_fr = format_stay_dates_fr(&row.check_in_at, &row.check_out_at);
    let dates_en = format_stay_dates_en(&row.check_in_at, &row.check_out_at);
    let vars_fr = [
        ("property", property_name),
        ("dates", dates_fr.as_str()),
        ("source", source_label),
    ];
    let vars_en = [
        ("property", property_name),
        ("dates", dates_en.as_str()),
        ("source", source_label),
    ];

    email::send(&SendEmailArgs {
        email_id: format!("stay-imported-{}", row.ical_uid),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: localized_with_pairs("email.stayImported.subject", &vars_fr, &vars_en),
            eyebrow: Some(email_i18n::text("email.stayImported.eyebrow")),
            title: Some(email_i18n::text("email.stayImported.title")),
            body: localized_with_pairs("email.stayImported.body", &vars_fr, &vars_en),
            cta: Some(ModuleEmailCta {
                label: email_i18n::text("email.stayImported.cta"),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id: None,
        property_id: Some(property_id),
        action_url: None,
    })
}

/// Batch sync digest after multiple new / updated stays.
pub fn notify_sync_summary(
    property_id: Uuid,
    sync_email_id: &str,
    synced_at: &str,
    diff: &SyncDiff,
) -> Result<()> {
    let new_count = diff.new_rows.len().to_string();
    let updated_count = diff.updated_rows.len().to_string();
    let imported = diff.imported_count().to_string();
    let incomplete_rows: Vec<&StayImportRow> = diff
        .new_rows
        .iter()
        .chain(diff.updated_rows.iter())
        .filter(|row| row.guest_email.as_deref().unwrap_or("").trim().is_empty())
        .collect();
    let incomplete_count = incomplete_rows.len();
    let incomplete_str = incomplete_count.to_string();

    let synced_fr = format_instant_fr(synced_at);
    let synced_en = format_instant_en(synced_at);

    let (subject, title, body, cta) = if incomplete_count == 0 {
        let vars = [
            ("imported", imported.as_str()),
            ("incomplete", "0"),
            ("newCount", new_count.as_str()),
            ("updatedCount", updated_count.as_str()),
            ("incompleteCount", "0"),
            ("syncedAt", synced_fr.as_str()),
        ];
        let vars_en = [
            ("imported", imported.as_str()),
            ("incomplete", "0"),
            ("newCount", new_count.as_str()),
            ("updatedCount", updated_count.as_str()),
            ("incompleteCount", "0"),
            ("syncedAt", synced_en.as_str()),
        ];
        (
            localized_with_pairs("email.syncSummary.subject.noneIncomplete", &vars, &vars_en),
            localized_with_pairs("email.syncSummary.title", &vars, &vars_en),
            localized_with_pairs("email.syncSummary.body.noneIncomplete", &vars, &vars_en),
            email_i18n::text("email.syncSummary.cta.ready"),
        )
    } else {
        let callout_fr = interpolate(
            &email_i18n::text("email.syncSummary.incompleteCallout").fr,
            &[("incompleteCount", incomplete_str.as_str())],
        );
        let callout_en = interpolate(
            &email_i18n::text("email.syncSummary.incompleteCallout").en,
            &[("incompleteCount", incomplete_str.as_str())],
        );
        let missing_fr = email_i18n::text("email.syncSummary.incomplete.email").fr;
        let missing_en = email_i18n::text("email.syncSummary.incomplete.email").en;
        let list_fr = incomplete_list(&incomplete_rows, &missing_fr, true);
        let list_en = incomplete_list(&incomplete_rows, &missing_en, false);

        let vars_fr = [
            ("imported", imported.as_str()),
            ("incomplete", incomplete_str.as_str()),
            ("newCount", new_count.as_str()),
            ("updatedCount", updated_count.as_str()),
            ("incompleteCount", incomplete_str.as_str()),
            ("syncedAt", synced_fr.as_str()),
            ("incompleteCallout", callout_fr.as_str()),
            ("incompleteList", list_fr.as_str()),
        ];
        let vars_en = [
            ("imported", imported.as_str()),
            ("incomplete", incomplete_str.as_str()),
            ("newCount", new_count.as_str()),
            ("updatedCount", updated_count.as_str()),
            ("incompleteCount", incomplete_str.as_str()),
            ("syncedAt", synced_en.as_str()),
            ("incompleteCallout", callout_en.as_str()),
            ("incompleteList", list_en.as_str()),
        ];
        let cta = if incomplete_count == 1 {
            email_i18n::text("email.syncSummary.cta.singular")
        } else {
            localized_with_pairs(
                "email.syncSummary.cta",
                &[("incompleteCount", incomplete_str.as_str())],
                &[("incompleteCount", incomplete_str.as_str())],
            )
        };
        (
            localized_with_pairs("email.syncSummary.subject", &vars_fr, &vars_en),
            localized_with_pairs("email.syncSummary.title", &vars_fr, &vars_en),
            localized_with_pairs("email.syncSummary.body", &vars_fr, &vars_en),
            cta,
        )
    };

    email::send(&SendEmailArgs {
        email_id: sync_email_id.to_string(),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject,
            eyebrow: Some(email_i18n::text("email.syncSummary.eyebrow")),
            title: Some(title),
            body,
            cta: Some(ModuleEmailCta {
                label: cta,
                url: None,
                portaki_action: None,
            }),
        },
        stay_id: None,
        property_id: Some(property_id),
        action_url: None,
    })
}

/// Human label for a calendar format / feed (FR-friendly default for host shell).
pub fn source_label(format: CalendarFormat, feed_label: Option<&str>) -> String {
    if let Some(label) = feed_label.map(str::trim).filter(|s| !s.is_empty()) {
        return label.to_string();
    }
    match format {
        CalendarFormat::Airbnb => "Airbnb".into(),
        CalendarFormat::Booking => "Booking".into(),
        CalendarFormat::AbritelVrbo => "Abritel / Vrbo".into(),
        CalendarFormat::Google => "Google".into(),
        CalendarFormat::Generic => "iCal".into(),
    }
}

fn incomplete_list(rows: &[&StayImportRow], missing_label: &str, french: bool) -> String {
    rows.iter()
        .take(8)
        .map(|row| {
            let dates = if french {
                format_stay_dates_fr(&row.check_in_at, &row.check_out_at)
            } else {
                format_stay_dates_en(&row.check_in_at, &row.check_out_at)
            };
            format!("{} · {} — {missing_label}", row.guest_name, dates)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn localized_with_pairs(
    key: &str,
    vars_fr: &[(&str, &str)],
    vars_en: &[(&str, &str)],
) -> LocalizedEmailText {
    let base = email_i18n::text(key);
    LocalizedEmailText::new(
        interpolate(&base.fr, vars_fr),
        interpolate(&base.en, vars_en),
    )
}

fn interpolate(template: &str, vars: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (name, value) in vars {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn format_stay_dates_fr(check_in: &str, check_out: &str) -> String {
    match (parse_day(check_in), parse_day(check_out)) {
        (Some(start), Some(end)) => format!("{} → {}", format_day_fr(start), format_day_fr(end)),
        _ => format!("{check_in} → {check_out}"),
    }
}

fn format_stay_dates_en(check_in: &str, check_out: &str) -> String {
    match (parse_day(check_in), parse_day(check_out)) {
        (Some(start), Some(end)) => format!("{} → {}", format_day_en(start), format_day_en(end)),
        _ => format!("{check_in} → {check_out}"),
    }
}

fn format_instant_fr(raw: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        let dt = dt.with_timezone(&Utc);
        return format!(
            "{} · {:02}:{:02}",
            format_day_fr(dt.date_naive()),
            dt.hour(),
            dt.minute()
        );
    }
    if let Some(day) = parse_day(raw) {
        return format_day_fr(day);
    }
    raw.to_string()
}

fn format_instant_en(raw: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        let dt = dt.with_timezone(&Utc);
        return format!(
            "{} · {:02}:{:02}",
            format_day_en(dt.date_naive()),
            dt.hour(),
            dt.minute()
        );
    }
    if let Some(day) = parse_day(raw) {
        return format_day_en(day);
    }
    raw.to_string()
}

fn parse_day(raw: &str) -> Option<NaiveDate> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc).date_naive());
    }
    NaiveDate::parse_from_str(&raw[..raw.len().min(10)], "%Y-%m-%d").ok()
}

fn format_day_fr(day: NaiveDate) -> String {
    let month = FR_MONTHS[(day.month0() as usize).min(11)];
    format!("{} {month} {}", day.day(), day.year())
}

fn format_day_en(day: NaiveDate) -> String {
    let month = EN_MONTHS[(day.month0() as usize).min(11)];
    format!("{month} {}, {}", day.day(), day.year())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_failed_copy_has_fr_and_en() {
        let subject = email_i18n::text("email.syncFailed.subject");
        assert!(!subject.fr.is_empty());
        assert!(!subject.en.is_empty());
    }

    #[test]
    fn stay_dates_format() {
        let fr = format_stay_dates_fr("2026-09-02T00:00:00Z", "2026-09-09T00:00:00Z");
        assert!(fr.contains("sept."));
        let en = format_stay_dates_en("2026-09-02T00:00:00Z", "2026-09-09T00:00:00Z");
        assert!(en.contains("Sep"));
    }
}
