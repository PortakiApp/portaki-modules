//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use ical_sync as _;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use serial_test::serial;

/// Airbnb et Booking branchés, relevés ce matin : la carte « Synchro » a des séjours à venir,
/// dont deux qui se chevauchent, et deux semaines d'historique.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    let sync_state = json!({
        "uids": {
            "1418fb94e984-4a1c@airbnb.com": {
                "checkInAt": "2026-06-22T14:00:00Z",
                "checkOutAt": "2026-06-29T08:00:00Z",
                "guestName": "Camille Roux",
                "channel": "airbnb",
                "hasGuestEmail": false
            },
            "9c1d5e2f7a@booking.com": {
                "checkInAt": "2026-06-27T14:00:00Z",
                "checkOutAt": "2026-07-02T08:00:00Z",
                "guestName": "",
                "channel": "booking",
                "hasGuestEmail": false
            }
        },
        "lastSuccessAt": "2026-06-15T06:30:00Z",
        "lastRunAt": "2026-06-15T06:30:00Z",
        "history": {
            "2026-06-13": { "ok": 24, "failed": 0, "newStays": 1 },
            "2026-06-14": { "ok": 23, "failed": 1, "newStays": 0 },
            "2026-06-15": { "ok": 7, "failed": 0, "newStays": 1 }
        },
        "summary": "2 stay(s) · 2 feed(s) ok · 0 feed(s) failed"
    });
    builder
        .with_config(&json!({"calendars": [
            {"id": "airbnb", "url": "https://www.airbnb.com/calendar/ical/41872301.ics?s=8f2c", "label": "Airbnb", "format": "airbnb", "channel": ""},
            {"id": "booking", "url": "https://admin.booking.com/hotel/hoteladmin/ical.html?t=5b1e9d", "label": "Booking", "format": "booking", "channel": ""}
        ]}))
        .with_kv("sync_state", serde_json::to_vec(&sync_state).unwrap())
}

#[test]
#[serial]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
#[serial]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
