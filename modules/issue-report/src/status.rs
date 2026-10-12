//! Report status (spec Signaler §1) : Nouveau · En cours · Résolu.
//!
//! `resolved_at` stays in step — set on `resolved`, cleared on reopening — so stats, the stay
//! encart and the 24 h task keep reading « open » as `resolved_at.is_none()`.

use portaki_sdk::prelude::*;

use crate::entities::IssueReport;

pub const NEW: &str = "new";
pub const IN_PROGRESS: &str = "in_progress";
pub const RESOLVED: &str = "resolved";

/// In the order of the segmented control.
pub const ALL: &[&str] = &[NEW, IN_PROGRESS, RESOLVED];

/// The report's status; a row written before the column reads from `resolved_at`.
pub fn of(report: &IssueReport) -> &'static str {
    match report.status.as_str() {
        IN_PROGRESS => IN_PROGRESS,
        RESOLVED => RESOLVED,
        NEW => NEW,
        _ if report.resolved_at.is_some() => RESOLVED,
        _ => NEW,
    }
}

/// A status sent by the dashboard; anything else is refused.
pub fn parse(raw: &str) -> Result<&'static str> {
    ALL.iter()
        .copied()
        .find(|status| *status == raw.trim())
        .ok_or_else(|| PortakiError::Host("invalid_status".to_string()))
}

/// The tone of a status: dot and pill.
pub fn tone(status: &str) -> Tone {
    match status {
        NEW => Tone::Warning,
        IN_PROGRESS => Tone::Info,
        _ => Tone::Success,
    }
}

/// i18n key of a status label.
pub fn label_key(status: &str) -> String {
    format!("status.{status}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn legacy(resolved: bool) -> IssueReport {
        IssueReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            category: "appliance".into(),
            summary: "x".into(),
            details: None,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            resolved_at: resolved.then_some(DateTime::<Utc>::UNIX_EPOCH),
            photo: None,
            status: String::new(),
            guest_name: String::new(),
            stay_check_in: None,
            stay_check_out: None,
        }
    }

    /// Une ligne d'avant la colonne : résolue si `resolved_at` est posé, nouvelle sinon.
    #[test]
    fn legacy_rows_read_their_status_from_resolved_at() {
        assert_eq!(of(&legacy(true)), RESOLVED);
        assert_eq!(of(&legacy(false)), NEW);
        assert_eq!(parse(" in_progress ").unwrap(), IN_PROGRESS);
        assert!(parse("closed").is_err());
    }
}
