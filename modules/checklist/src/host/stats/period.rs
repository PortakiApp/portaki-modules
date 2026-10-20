//! Ce que les détails lisent de la page : la période, ses séjours, la langue ; et le pourcentage.

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

pub(crate) fn percent(part: usize, whole: usize) -> String {
    match (part * 100).checked_div(whole) {
        Some(value) => format!("{value} %"),
        None => "—".to_string(),
    }
}

/// Dashboard period selector: 30 j / 90 j / 12 mois; anything else falls back to 30.
pub(crate) fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
}

/// The period's stays the platform passes to the detail page, oldest first; `None` when it sends
/// none (developer preview, older platform).
pub(crate) fn period_stays(ctx: &HostContext) -> Option<Vec<TimelineStay>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        id: Uuid,
        check_in: DateTime<Utc>,
        check_out: DateTime<Utc>,
        #[serde(default)]
        status: String,
    }
    let raw: Vec<Raw> = serde_json::from_value(ctx.input.get("stays")?.clone()).ok()?;
    let mut stays: Vec<TimelineStay> = raw
        .into_iter()
        .map(|stay| TimelineStay {
            id: stay.id,
            check_in: stay.check_in,
            check_out: stay.check_out,
            guest_name: String::new(),
            status: stay.status,
        })
        .collect();
    stays.sort_by_key(|stay| stay.check_in);
    Some(stays)
}

pub(crate) fn is_fr(locale: &str) -> bool {
    locale.trim().to_ascii_lowercase().starts_with("fr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(format_duration(Duration::minutes(130)), "2 h 10");
        assert_eq!(format_duration(Duration::minutes(40)), "40 min");
        assert_eq!(format_duration(Duration::minutes(120)), "2 h");
        assert_eq!(decimal(5.14, true), "5,1");
        assert_eq!(percent(7, 9), "77 %");
        assert_eq!(percent(0, 0), "—");
    }
}
