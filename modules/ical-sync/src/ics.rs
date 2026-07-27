//! Minimal iCalendar VEVENT parser for stay import.

use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::config::CalendarFormat;

/// Stay row shape expected by `ModuleGatewayStayImportAdapter`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StayImportRow {
    pub guest_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_email: Option<String>,
    #[serde(default)]
    pub guest_lang: String,
    pub check_in_at: String,
    pub check_out_at: String,
    pub ical_uid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VEvent {
    uid: String,
    summary: String,
    description: String,
    dtstart: Option<DateTime<Utc>>,
    dtend: Option<DateTime<Utc>>,
}

/// Parse ICS text into stay import rows (max `limit`), using the declared feed format.
pub fn parse_stay_rows(
    ics_body: &str,
    guest_lang: &str,
    limit: usize,
    format: CalendarFormat,
) -> Vec<StayImportRow> {
    if ics_body.trim().is_empty() || limit == 0 {
        return Vec::new();
    }

    let unfolded = unfold_ics(ics_body);
    let mut rows = Vec::new();
    let mut current: Option<VEvent> = None;

    for line in unfolded.lines() {
        let line = line.trim_end_matches('\r');
        if line.eq_ignore_ascii_case("BEGIN:VEVENT") {
            current = Some(VEvent {
                uid: String::new(),
                summary: String::new(),
                description: String::new(),
                dtstart: None,
                dtend: None,
            });
            continue;
        }
        if line.eq_ignore_ascii_case("END:VEVENT") {
            if let Some(event) = current.take() {
                if let Some(row) = event_to_row(event, guest_lang, format) {
                    rows.push(row);
                    if rows.len() >= limit {
                        break;
                    }
                }
            }
            continue;
        }
        let Some(event) = current.as_mut() else {
            continue;
        };
        apply_property(event, line);
    }

    rows
}

fn event_to_row(event: VEvent, guest_lang: &str, format: CalendarFormat) -> Option<StayImportRow> {
    let check_in = event.dtstart?;
    let check_out = event
        .dtend
        .unwrap_or_else(|| check_in + chrono::Duration::days(1));
    if check_out <= check_in {
        return None;
    }

    let guest_name = guest_name_for_format(&event, format)?;
    let ical_uid = if event.uid.trim().is_empty() {
        format!(
            "generated:{}:{}",
            check_in.timestamp(),
            guest_name.replace(' ', "_")
        )
    } else {
        event.uid
    };

    Some(StayImportRow {
        guest_name,
        guest_email: None,
        guest_lang: guest_lang.trim().to_string(),
        check_in_at: check_in.to_rfc3339(),
        check_out_at: check_out.to_rfc3339(),
        ical_uid,
    })
}

/// Returns `None` when the VEVENT is a block / unavailability (not a real stay).
fn guest_name_for_format(event: &VEvent, format: CalendarFormat) -> Option<String> {
    match format {
        CalendarFormat::Airbnb => airbnb_guest_name(event),
        CalendarFormat::Booking => booking_guest_name(event),
        CalendarFormat::AbritelVrbo => abritel_vrbo_guest_name(event),
        CalendarFormat::Google | CalendarFormat::Generic => generic_guest_name(event),
    }
}

/// Airbnb: `SUMMARY:Reserved` (+ `Name:` in DESCRIPTION) = stay.
/// `Not available` / `Reserved - Not available` / `Airbnb (Not available)` = block → skip.
fn airbnb_guest_name(event: &VEvent) -> Option<String> {
    let summary = event.summary.trim();
    let normalized = normalize_summary(summary);

    if is_airbnb_block(&normalized) {
        return None;
    }

    if let Some(name) = guest_name_from_description(&event.description) {
        return Some(name);
    }

    if normalized == "reserved" {
        return Some("Guest".to_string());
    }

    if let Some(rest) = strip_reserved_prefix(summary) {
        let rest_norm = normalize_summary(rest);
        if !rest_norm.is_empty() && !is_airbnb_block(&rest_norm) {
            return Some(rest.trim().to_string());
        }
    }

    if !summary.is_empty() && !is_placeholder_summary(&normalized) {
        return Some(summary.to_string());
    }

    Some("Guest".to_string())
}

fn is_airbnb_block(normalized_summary: &str) -> bool {
    if normalized_summary.is_empty() {
        return false;
    }
    normalized_summary.contains("not available")
        || normalized_summary.contains("unavailable")
        || normalized_summary == "blocked"
        || normalized_summary.ends_with(" (not available)")
        || normalized_summary == "airbnb (not available)"
}

/// Booking: `CLOSED - Not available` (and similar) = block; otherwise SUMMARY is the label.
fn booking_guest_name(event: &VEvent) -> Option<String> {
    let summary = event.summary.trim();
    let normalized = normalize_summary(summary);

    if is_booking_block(&normalized) {
        return None;
    }

    if let Some(name) = guest_name_from_description(&event.description) {
        return Some(name);
    }

    if !summary.is_empty() && !is_placeholder_summary(&normalized) {
        return Some(summary.to_string());
    }

    Some("Guest".to_string())
}

fn is_booking_block(normalized_summary: &str) -> bool {
    normalized_summary.contains("not available")
        || normalized_summary.starts_with("closed")
        || normalized_summary == "blocked"
        || normalized_summary.contains("unavailable")
}

/// Abritel / Vrbo: skip obvious blocks; prefer DESCRIPTION name when present.
fn abritel_vrbo_guest_name(event: &VEvent) -> Option<String> {
    let summary = event.summary.trim();
    let normalized = normalize_summary(summary);

    if is_generic_block(&normalized) {
        return None;
    }

    if let Some(name) = guest_name_from_description(&event.description) {
        return Some(name);
    }

    if !summary.is_empty() && !is_placeholder_summary(&normalized) {
        return Some(summary.to_string());
    }

    Some("Guest".to_string())
}

/// Google / generic ICS: import dated events unless SUMMARY is a clear block label.
fn generic_guest_name(event: &VEvent) -> Option<String> {
    let summary = event.summary.trim();
    let normalized = normalize_summary(summary);

    if is_generic_block(&normalized) {
        return None;
    }

    if let Some(name) = guest_name_from_description(&event.description) {
        return Some(name);
    }

    if !summary.is_empty() {
        return Some(summary.to_string());
    }

    Some("Guest".to_string())
}

fn is_generic_block(normalized_summary: &str) -> bool {
    normalized_summary.contains("not available")
        || normalized_summary == "blocked"
        || normalized_summary == "unavailable"
        || normalized_summary.starts_with("closed -")
        || normalized_summary.starts_with("closed –")
}

fn is_placeholder_summary(normalized: &str) -> bool {
    matches!(
        normalized,
        "reserved" | "blocked" | "not available" | "unavailable" | "closed"
    )
}

fn normalize_summary(summary: &str) -> String {
    summary
        .trim()
        .to_ascii_lowercase()
        .replace(['–', '—'], "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_reserved_prefix(summary: &str) -> Option<&str> {
    let trimmed = summary.trim();
    let lower = trimmed.to_ascii_lowercase();
    for prefix in ["reserved - ", "reserved – ", "reserved — ", "reserved: "] {
        if lower.starts_with(prefix) {
            return Some(trimmed[prefix.len()..].trim());
        }
    }
    None
}

fn guest_name_from_description(description: &str) -> Option<String> {
    for line in description.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        for prefix in ["Name:", "Guest:", "Nom:", "Voyageur:"] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = rest.trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
            // Case-insensitive prefix match
            if trimmed.len() >= prefix.len() && trimmed[..prefix.len()].eq_ignore_ascii_case(prefix)
            {
                let name = trimmed[prefix.len()..].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

fn apply_property(event: &mut VEvent, line: &str) {
    let Some((raw_key, value)) = split_property(line) else {
        return;
    };
    let key = raw_key
        .split(';')
        .next()
        .unwrap_or(raw_key)
        .trim()
        .to_ascii_uppercase();
    match key.as_str() {
        "UID" => event.uid = unescape_text(value),
        "SUMMARY" => event.summary = unescape_text(value),
        "DESCRIPTION" => event.description = unescape_text(value),
        "DTSTART" => event.dtstart = parse_ics_datetime(raw_key, value),
        "DTEND" => event.dtend = parse_ics_datetime(raw_key, value),
        _ => {}
    }
}

fn split_property(line: &str) -> Option<(&str, &str)> {
    let idx = line.find(':')?;
    Some((&line[..idx], &line[idx + 1..]))
}

fn unescape_text(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

fn unfold_ics(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            continue;
        }
        if ch == '\n' {
            match chars.peek() {
                Some(' ') | Some('\t') => {
                    chars.next();
                    continue;
                }
                _ => out.push('\n'),
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn parse_ics_datetime(raw_key: &str, value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let value_is_date = raw_key.to_ascii_uppercase().contains("VALUE=DATE")
        || (value.len() == 8 && value.chars().all(|c| c.is_ascii_digit()));

    if value_is_date {
        let date = NaiveDate::parse_from_str(value, "%Y%m%d").ok()?;
        return Some(Utc.from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0)?)));
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }

    // Form 1: 20260720T140000Z
    if value.ends_with('Z') {
        let naive =
            chrono::NaiveDateTime::parse_from_str(value.trim_end_matches('Z'), "%Y%m%dT%H%M%S")
                .ok()?;
        return Some(Utc.from_utc_datetime(&naive));
    }

    // Form 2: floating local time — treat as UTC (feeds rarely use TZID without VTIMEZONE).
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S") {
        return Some(Utc.from_utc_datetime(&naive));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIRBNB_SAMPLE: &str = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
UID:abc-123@airbnb.com\r\n\
DTSTART;VALUE=DATE:20260720\r\n\
DTEND;VALUE=DATE:20260725\r\n\
SUMMARY:Reserved\r\n\
DESCRIPTION:Name: Marie Dupont\\nPhone: +33\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:block-1@airbnb.com\r\n\
DTSTART;VALUE=DATE:20260710\r\n\
DTEND;VALUE=DATE:20260712\r\n\
SUMMARY:Reserved - Not available\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:block-2@airbnb.com\r\n\
DTSTART;VALUE=DATE:20260713\r\n\
DTEND;VALUE=DATE:20260714\r\n\
SUMMARY:Airbnb (Not available)\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:def-456\r\n\
DTSTART:20260801T160000Z\r\n\
DTEND:20260808T100000Z\r\n\
SUMMARY:Julien Roy\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn airbnb_imports_reservations_skips_not_available() {
        let rows = parse_stay_rows(AIRBNB_SAMPLE, "fr", 50, CalendarFormat::Airbnb);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].guest_name, "Marie Dupont");
        assert_eq!(rows[0].ical_uid, "abc-123@airbnb.com");
        assert!(rows[0].check_in_at.starts_with("2026-07-20"));
        assert!(rows[0].check_out_at.starts_with("2026-07-25"));
        assert_eq!(rows[1].guest_name, "Julien Roy");
        assert!(rows[1].check_in_at.contains("T16:00:00"));
    }

    #[test]
    fn airbnb_reserved_without_name_still_imports() {
        let ics = "BEGIN:VEVENT\nUID:r1\nDTSTART;VALUE=DATE:20260901\n\
DTEND;VALUE=DATE:20260903\nSUMMARY:Reserved\nEND:VEVENT\n";
        let rows = parse_stay_rows(ics, "fr", 10, CalendarFormat::Airbnb);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].guest_name, "Guest");
    }

    #[test]
    fn booking_skips_closed_not_available() {
        let ics = "BEGIN:VEVENT\nUID:b1\nDTSTART;VALUE=DATE:20260901\n\
DTEND;VALUE=DATE:20260905\nSUMMARY:CLOSED - Not available\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:b2\nDTSTART;VALUE=DATE:20260910\n\
DTEND;VALUE=DATE:20260912\nSUMMARY:Dupont\nEND:VEVENT\n";
        let rows = parse_stay_rows(ics, "fr", 10, CalendarFormat::Booking);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].guest_name, "Dupont");
    }

    #[test]
    fn generic_skips_blocked_keyword() {
        let ics = "BEGIN:VEVENT\nUID:g1\nDTSTART;VALUE=DATE:20260901\n\
DTEND;VALUE=DATE:20260902\nSUMMARY:Blocked\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:g2\nDTSTART;VALUE=DATE:20260903\n\
DTEND;VALUE=DATE:20260904\nSUMMARY:Family week\nEND:VEVENT\n";
        let rows = parse_stay_rows(ics, "en", 10, CalendarFormat::Generic);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].guest_name, "Family week");
    }

    #[test]
    fn unfolds_continued_lines() {
        let folded = "BEGIN:VEVENT\nUID:x\nSUMMARY:Hel\n lo\nDTSTART;VALUE=DATE:20260101\nDTEND;VALUE=DATE:20260102\nEND:VEVENT\n";
        let rows = parse_stay_rows(folded, "en", 10, CalendarFormat::Generic);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].guest_name, "Hello");
    }

    #[test]
    fn empty_body_returns_no_rows() {
        assert!(parse_stay_rows("", "fr", 10, CalendarFormat::Airbnb).is_empty());
    }
}
