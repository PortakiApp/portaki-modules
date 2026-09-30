//! The access cell of the booklet status strip (§1.5).
//!
//! The strip shows one cell the booklet does not write: what the guest opens the door with. These
//! tests pin the two things that cell must never get wrong — the wording follows the access
//! method, and a code that is not due does not travel.

use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serial_test::serial;

use access_guide::{render_status_cell, HostConfig};

const KEYBOX_CODE: &str = "4821";
const GATE_CODE: &str = "A17B";
const BACKUP_CODE: &str = "7391";

/// A config revealed straight away, so a test that is not about timing reads the real value.
fn revealed(primary_method: &str) -> HostConfig {
    HostConfig {
        primary_method: primary_method.into(),
        reveal_policy: "always".into(),
        address: "Ch. des Douaniers".into(),
        ..HostConfig::default()
    }
}

fn render(config: HostConfig) -> String {
    let mut json = String::new();
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let surface = render_status_cell(ctx).expect("surface");
            assert!(
                SurfaceAssertions::new(&surface).contains_type("KeyValue"),
                "the strip needs one KeyValue to draw a cell",
            );
            json = serde_json::to_string(&surface).expect("json");
        });
    json
}

#[test]
#[serial]
fn a_keybox_names_its_code() {
    let json = render(HostConfig {
        keybox_location: I18nText::new("Sous la boîte aux lettres", ""),
        keybox_code: KEYBOX_CODE.into(),
        ..revealed("keybox")
    });
    assert!(json.contains("i18n:guest.status.keybox"));
    assert!(json.contains(KEYBOX_CODE));
    // A code reads in aligned figures, and the guest may copy it.
    assert!(json.contains("\"mono\":true"));
    assert!(json.contains("\"copy\":true"));
    // The cell is a tile of the strip, not a row of a list.
    assert!(json.contains("\"layout\":\"tile\""));
}

#[test]
#[serial]
fn a_door_code_names_which_door() {
    let json = render(HostConfig {
        door_code_target: "gate".into(),
        door_code: GATE_CODE.into(),
        ..revealed("door_code")
    });
    assert!(json.contains("i18n:guest.status.doorCode.gate"));
    assert!(json.contains(GATE_CODE));
}

#[test]
#[serial]
fn a_smart_lock_names_its_backup_code() {
    let json = render(HostConfig {
        smart_lock_manual_code: BACKUP_CODE.into(),
        ..revealed("smart_lock")
    });
    assert!(json.contains("i18n:guest.status.smartLock"));
    assert!(json.contains(BACKUP_CODE));
}

/// A handover has no code: the cell shows the slot, and nothing is masked.
#[test]
#[serial]
fn a_handover_shows_its_slot() {
    let json = render(HostConfig {
        in_person_meeting_place: I18nText::new("Devant le 12", ""),
        in_person_time_hint: I18nText::new("16–19 h", ""),
        ..revealed("in_person")
    });
    assert!(json.contains("i18n:guest.status.inPerson"));
    assert!(json.contains("16–19 h"));
    assert!(!json.contains("••••••"));
    assert!(!json.contains("\"secret\""));
}

/// No slot written: the cell says it is to be agreed rather than showing the meeting place as if
/// it were a time.
#[test]
#[serial]
fn a_handover_without_a_slot_falls_back_to_the_place() {
    let json = render(HostConfig {
        in_person_meeting_place: I18nText::new("Devant le 12", ""),
        ..revealed("in_person")
    });
    assert!(json.contains("i18n:guest.status.inPerson"));
    assert!(json.contains("Devant le 12"));
}

#[test]
#[serial]
fn a_reception_desk_hands_over_keys() {
    let json = render(HostConfig {
        building_staff_kind: "reception".into(),
        building_staff_desk_location: I18nText::new("Hall, à gauche", ""),
        ..revealed("building_staff")
    });
    assert!(json.contains("i18n:guest.status.keys"));
    assert!(json.contains("i18n:guest.buildingStaff.reception"));
}

/// §1.5 asks for « Accueil · {hôte} ». `Context.host` exists in the SDK and the runtime does not
/// fill it, so the cell names the role. This test is the record of that gap: it fails the day the
/// host arrives, which is when the wording should change.
#[test]
#[serial]
fn a_host_welcome_names_the_role_until_the_runtime_sends_the_host() {
    let json = render(revealed("host_greets"));
    assert!(json.contains("i18n:guest.status.welcome"));
    assert!(json.contains("i18n:guest.status.welcome.host"));
}

#[test]
#[serial]
fn a_host_welcome_prefers_the_eta_the_host_wrote() {
    let json = render(HostConfig {
        host_greets_eta_hint: I18nText::new("vers 17 h", ""),
        ..revealed("host_greets")
    });
    assert!(json.contains("i18n:guest.status.welcome"));
    assert!(json.contains("vers 17 h"));
}

/// A code-bearing method the host left without a code: the cell points at the page instead of
/// masking an emptiness.
#[test]
#[serial]
fn a_method_without_its_code_points_at_the_instructions() {
    let json = render(HostConfig {
        keybox_location: I18nText::new("Sous la boîte aux lettres", ""),
        ..revealed("keybox")
    });
    assert!(json.contains("i18n:guest.status.access.instructions"));
    assert!(!json.contains("••••••"));
}

/// The rule that matters: before its hour, the code is not in the payload at all — not masked in
/// CSS, not hidden in a sibling field. Only the mask travels.
#[test]
#[serial]
fn a_code_that_is_not_due_never_travels() {
    let json = render(HostConfig {
        keybox_code: KEYBOX_CODE.into(),
        reveal_policy: "day_before_16h".into(),
        ..revealed("keybox")
    });
    // No stay, timed policy → fail-safe lock.
    assert!(!json.contains(KEYBOX_CODE));
    assert!(json.contains("••••••"));
    // The label gives away neither the method nor the kind of secret.
    assert!(json.contains("i18n:guest.status.pending"));
    assert!(!json.contains("i18n:guest.status.keybox"));
    assert!(json.contains("\"secret\""));
    // Nothing to copy while there is nothing to read.
    assert!(!json.contains("\"copy\":true"));
}

/// The host has written nothing: no cell. The strip falls back to a grid of one or two cells
/// rather than drawing an empty one (§0.5) — and an unconfigured module is silence, not content.
#[test]
#[serial]
fn nothing_configured_draws_no_cell() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_status_cell(ctx).expect("surface");
            assert!(!SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            // Not an EmptyState either: the strip has no room for one, it just drops the cell.
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}
