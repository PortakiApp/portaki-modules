//! Persisted UID snapshot used to detect new / updated stays between syncs.

use std::collections::BTreeMap;

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

use crate::ics::StayImportRow;

const SYNC_STATE_KEY: &str = "sync_state";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeenStay {
    pub check_in_at: String,
    pub check_out_at: String,
    #[serde(default)]
    pub guest_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    /// `icalUid` → last seen dates / name.
    #[serde(default)]
    pub uids: BTreeMap<String, SeenStay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<String>,
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

pub fn next_state(rows: &[StayImportRow], last_success_at: Option<String>) -> SyncState {
    let mut uids = BTreeMap::new();
    for row in rows {
        uids.insert(
            row.ical_uid.clone(),
            SeenStay {
                check_in_at: row.check_in_at.clone(),
                check_out_at: row.check_out_at.clone(),
                guest_name: row.guest_name.clone(),
            },
        );
    }
    SyncState {
        uids,
        last_success_at,
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
                },
            )]),
            last_success_at: None,
        };
        let rows = vec![row("a", "2026-08-01T00:00:00Z", "2026-08-06T00:00:00Z")];
        let diff = diff_rows(&previous, &rows);
        assert!(diff.new_rows.is_empty());
        assert_eq!(diff.updated_rows.len(), 1);
    }
}
