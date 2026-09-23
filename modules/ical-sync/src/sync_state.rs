//! Persisted UID snapshot used to detect new / updated stays between syncs, plus the daily
//! run history the stats card reads.

use std::collections::BTreeMap;

use portaki_sdk::contracts::booking_channel::BookingChannel;
use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

use crate::ics::StayImportRow;

const SYNC_STATE_KEY: &str = "sync_state";

/// Days of run history kept — the stats card's longest period (12 months) plus slack.
const HISTORY_DAYS: usize = 400;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeenStay {
    pub check_in_at: String,
    pub check_out_at: String,
    #[serde(default)]
    pub guest_name: String,
    /// Platform the stay came from (stats: stays per calendar).
    #[serde(default)]
    pub channel: BookingChannel,
    /// The feed carried a guest email (stats: incomplete stays).
    #[serde(default)]
    pub has_guest_email: bool,
}

/// Sync runs of one day (`YYYY-MM-DD`, UTC).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayRuns {
    /// Runs where every feed was read.
    #[serde(default)]
    pub ok: u32,
    /// Runs where at least one feed failed.
    #[serde(default)]
    pub failed: u32,
    /// Stays seen for the first time that day.
    #[serde(default)]
    pub new_stays: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    /// `icalUid` → last seen dates / name.
    #[serde(default)]
    pub uids: BTreeMap<String, SeenStay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<String>,
    /// `YYYY-MM-DD` → runs that day, oldest dropped past [`HISTORY_DAYS`].
    #[serde(default)]
    pub history: BTreeMap<String, DayRuns>,
}

impl SyncState {
    /// Counts one run on `day` (`YYYY-MM-DD`).
    pub fn record_run(&mut self, day: &str, failed: bool, new_stays: usize) {
        if day.is_empty() {
            return;
        }
        let runs = self.history.entry(day.to_string()).or_default();
        if failed {
            runs.failed += 1;
        } else {
            runs.ok += 1;
        }
        runs.new_stays += u32::try_from(new_stays).unwrap_or(u32::MAX);
        while self.history.len() > HISTORY_DAYS {
            self.history.pop_first();
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncDiff {
    pub new_rows: Vec<StayImportRow>,
    pub updated_rows: Vec<StayImportRow>,
}

impl SyncDiff {
    pub fn is_empty(&self) -> bool {
        self.new_rows.is_empty() && self.updated_rows.is_empty()
    }

    pub fn imported_count(&self) -> usize {
        self.new_rows.len() + self.updated_rows.len()
    }
}

pub fn load_sync_state() -> Result<SyncState> {
    let Some(bytes) = host::kv::get(SYNC_STATE_KEY)? else {
        return Ok(SyncState::default());
    };
    serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid sync_state JSON: {error}"))
    })
}

pub fn save_sync_state(state: &SyncState) -> Result<()> {
    let bytes = serde_json::to_vec(state).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("sync_state serialize: {error}"))
    })?;
    host::kv::set(SYNC_STATE_KEY, &bytes, None)
}

pub fn diff_rows(previous: &SyncState, rows: &[StayImportRow]) -> SyncDiff {
    let mut new_rows = Vec::new();
    let mut updated_rows = Vec::new();

    for row in rows {
        match previous.uids.get(&row.ical_uid) {
            None => new_rows.push(row.clone()),
            Some(seen)
                if seen.check_in_at != row.check_in_at || seen.check_out_at != row.check_out_at =>
            {
                updated_rows.push(row.clone());
            }
            Some(_) => {}
        }
    }

    SyncDiff {
        new_rows,
        updated_rows,
    }
}

/// Snapshot of `rows`, keeping `previous`'s run history.
pub fn next_state(
    previous: &SyncState,
    rows: &[StayImportRow],
    last_success_at: Option<String>,
) -> SyncState {
    let mut uids = BTreeMap::new();
    for row in rows {
        uids.insert(
            row.ical_uid.clone(),
            SeenStay {
                check_in_at: row.check_in_at.clone(),
                check_out_at: row.check_out_at.clone(),
                guest_name: row.guest_name.clone(),
                channel: row.booking_channel,
                has_guest_email: row
                    .guest_email
                    .as_deref()
                    .is_some_and(|e| !e.trim().is_empty()),
            },
        );
    }
    SyncState {
        uids,
        last_success_at,
        history: previous.history.clone(),
    }
}

#[cfg(test)]
mod tests {
    use portaki_sdk::contracts::booking_channel::{BookingChannel, ChannelSignal};

    use super::*;

    fn row(uid: &str, check_in: &str, check_out: &str) -> StayImportRow {
        StayImportRow {
            guest_name: "Ada".into(),
            guest_email: None,
            guest_lang: "fr".into(),
            check_in_at: check_in.into(),
            check_out_at: check_out.into(),
            ical_uid: uid.into(),
            booking_channel: BookingChannel::Airbnb,
            booking_channel_signal: ChannelSignal::IcalUidSuffix,
        }
    }

    #[test]
    fn first_sync_marks_all_new() {
        let rows = vec![
            row("a", "2026-08-01T00:00:00Z", "2026-08-05T00:00:00Z"),
            row("b", "2026-08-10T00:00:00Z", "2026-08-12T00:00:00Z"),
        ];
        let diff = diff_rows(&SyncState::default(), &rows);
        assert_eq!(diff.new_rows.len(), 2);
        assert!(diff.updated_rows.is_empty());
    }

    #[test]
    fn date_change_is_updated() {
        let previous = SyncState {
            uids: BTreeMap::from([(
                "a".into(),
                SeenStay {
                    check_in_at: "2026-08-01T00:00:00Z".into(),
                    check_out_at: "2026-08-05T00:00:00Z".into(),
                    guest_name: "Ada".into(),
                    ..SeenStay::default()
                },
            )]),
            ..SyncState::default()
        };
        let rows = vec![row("a", "2026-08-01T00:00:00Z", "2026-08-06T00:00:00Z")];
        let diff = diff_rows(&previous, &rows);
        assert!(diff.new_rows.is_empty());
        assert_eq!(diff.updated_rows.len(), 1);
    }

    #[test]
    fn history_counts_runs_and_drops_the_oldest_days() {
        let mut state = SyncState::default();
        state.record_run("2026-09-01", false, 2);
        state.record_run("2026-09-01", true, 0);
        assert_eq!(
            state.history["2026-09-01"],
            DayRuns {
                ok: 1,
                failed: 1,
                new_stays: 2
            }
        );
        for day in 0..HISTORY_DAYS + 5 {
            state.record_run(&format!("2027-{day:05}"), false, 0);
        }
        assert_eq!(state.history.len(), HISTORY_DAYS);
        assert!(!state.history.contains_key("2026-09-01"));
    }
}
