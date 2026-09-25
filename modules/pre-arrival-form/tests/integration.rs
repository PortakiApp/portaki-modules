//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use serial_test::serial;

use uuid::Uuid;

use chrono::{Duration, Utc};
use portaki_sdk::prelude::StayContext;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use pre_arrival_form::{
    get_status, publish_readiness, render_guest_form, render_home_card, render_host_main,
    render_host_stay, reset_test_store, send_form_available, submit, ModuleConfig, ShowWhen,
    SubmitArgs,
};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_submit() -> SubmitArgs {
    SubmitArgs {
        arrival_time_estimated: Some("17:30".into()),
        guest_occasion: Some("Anniversaire".into()),
        guest_allergies: None,
        guest_count: Some("2".into()),
        special_needs: None,
        id_document: None,
        message_to_host: Some("Merci !".into()),
    }
}

#[test]
#[serial]
fn home_card_renders_form_when_incomplete() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.pending"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("guest.form"));
            assert!(!json.contains("TimePicker"));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("TimePicker"));
            assert!(SurfaceAssertions::new(&form).contains_type("TextArea"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            // Overlay chrome owns framing — no nested Card around the form.
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("home.card.intro"));
            assert!(form_json.contains("submit"));
        });
}

#[test]
#[serial]
fn submit_then_status_and_thanks_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            // Still before check-in — completed form stays editable.
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(2)),
                checkout_at: Some(Utc::now() + Duration::days(5)),
                booking_channel: None,
                ..StayContext::default()
            });

            let before = get_status(ctx.clone()).expect("status");
            assert!(!before.completed);

            submit(ctx.clone(), sample_submit()).expect("submit");

            let after = get_status(ctx.clone()).expect("status after");
            assert!(after.completed);
            assert_eq!(after.arrival_time_estimated.as_deref(), Some("17:30"));
            assert_eq!(after.guest_occasion.as_deref(), Some("Anniversaire"));

            let surface = render_home_card(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.allReady"));
            assert!(json.contains("home.task.completed"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("check-circle"));
            assert!(!json.contains("TimePicker"));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("form.submitUpdate"));
            assert!(form_json.contains("17:30"));
            assert!(form_json.contains("Anniversaire"));
        });
}

#[test]
#[serial]
fn completed_form_locks_after_checkin() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::hours(2)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            submit(ctx.clone(), sample_submit()).expect("submit before check-in");

            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() - Duration::hours(1)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });

            let form = render_guest_form(ctx.clone()).expect("guest surface");
            assert!(!SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Button"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("home.card.thanks"));
            assert!(form_json.contains("home.card.lockedHint"));
            assert!(form_json.contains("17:30"));

            let err = submit(
                ctx,
                SubmitArgs {
                    arrival_time_estimated: Some("18:00".into()),
                    guest_occasion: None,
                    guest_allergies: None,
                    guest_count: None,
                    special_needs: None,
                    id_document: None,
                    message_to_host: None,
                },
            )
            .expect_err("submit locked after check-in");
            assert!(format!("{err:?}").contains("form_locked_after_checkin"));
        });
}

#[test]
#[serial]
fn home_card_gated_omits_form_teaser_keeps_police_fragment() {
    reset_test_store();
    let config = json!({
        "show_when": "before",
        "ask_arrival_time": true,
        "ask_occasion": true,
        "ask_allergies": true,
        "ask_guest_count": true,
        "ask_special_needs": false,
        "ask_id_document": false
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(10)),
                checkout_at: Some(Utc::now() + Duration::days(14)),
                booking_channel: None,
                ..StayContext::default()
            });

            let surface = render_home_card(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            // Gated: no form ListItem / soon teaser — police fragment only.
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(!json.contains("home.formalities.pendingGate"));
            assert!(!json.contains("home.card.notYet"));
            assert!(!json.contains("home.task.preArrival"));
            assert!(json.contains("home.formalities.pending"));
            assert!(!json.contains("home.card.intro"));
            assert!(!json.contains("TimePicker"));
            assert!(!json.contains("\"type\":\"Form\"") && !json.contains("\"type\": \"Form\""));
        });
}

#[test]
#[serial]
fn send_form_available_noops_when_gated() {
    reset_test_store();
    let config = json!({
        "show_when": "checkin"
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(5)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            send_form_available(ctx, portaki_sdk::prelude::EmptyArgs {})
                .expect("sendFormAvailable gated no-op");
        });
}

#[test]
#[serial]
fn send_form_available_ok_when_confirm() {
    reset_test_store();
    let config = json!({
        "show_when": "confirm"
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(20)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            send_form_available(ctx, portaki_sdk::prelude::EmptyArgs {})
                .expect("sendFormAvailable when available");
        });
}

#[test]
#[serial]
fn host_main_renders_config_editor() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_config(&ModuleConfig {
            show_when: ShowWhen::Checkin,
            ask_special_needs: true,
            ask_id_document: true,
            ..ModuleConfig::default()
        })
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            // Six flat toggles and `show_when`: exactly the declared keys.
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ChoiceList"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ToggleRow"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("show_when"));
            assert!(json.contains("ask_arrival_time"));
            assert!(json.contains("ask_id_document"));
            assert!(json.contains("host.section.when"));
            assert!(json.contains("host.section.questions"));
            // Design question tiles: bordered ToggleRow + leading icon chip.
            assert!(json.contains("\"icon\":\"clock-circle\""));
            assert!(json.contains("\"icon\":\"gift\""));
            assert!(json.contains("\"icon\":\"users\""));
        });
}

#[test]
#[serial]
fn guest_form_respects_question_toggles() {
    reset_test_store();
    let config = json!({
        "show_when": "confirm",
        "ask_arrival_time": true,
        "ask_occasion": false,
        "ask_allergies": true,
        "ask_guest_count": false,
        "ask_special_needs": true,
        "ask_id_document": true
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|ctx| {
            let surface = render_guest_form(ctx).expect("guest surface");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("form.arrival.label"));
            assert!(!json.contains("form.occasion.label"));
            assert!(json.contains("form.specialNeeds.label"));
            assert!(json.contains("form.idDocument.label"));
            assert!(!json.contains("form.guestCount.label"));
        });
}

#[test]
#[serial]
fn host_stay_surface_pending_without_response() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({
                "stayId": stay_id.to_string(),
                "guestName": "Liam O'Brien",
                "stayDates": "21 – 26 août",
            });
            let surface = render_host_stay(ctx).expect("host stay");
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("surface.host.stay.title"));
            assert!(json.contains("host.stay.status.pending"));
            assert!(json.contains("host.stay.pending"));
            assert!(!json.contains("form.arrival.label"));
        });
}

#[test]
#[serial]
fn host_stay_surface_shows_completed_response() {
    reset_test_store();
    let stay_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("uuid");

    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            if let Some(guest) = ctx.guest.as_mut() {
                guest.session_id = stay_id;
            }
            submit(
                ctx,
                SubmitArgs {
                    arrival_time_estimated: Some("17:30".into()),
                    guest_occasion: Some("Lune de miel".into()),
                    guest_allergies: Some("Fruits à coque".into()),
                    guest_count: None,
                    special_needs: None,
                    id_document: None,
                    message_to_host: Some("Champagne au frais".into()),
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx).expect("host stay");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.status.done"));
            assert!(
                json.contains("\"icon\":\"clipboard\"") || json.contains("\"icon\": \"clipboard\"")
            );
            assert!(json.contains("host.stay.arrival.label"));
            assert!(json.contains("host.stay.occasion.label"));
            assert!(json.contains("host.stay.allergies.label"));
            assert!(json.contains("clock-circle"));
            assert!(json.contains("17:30"));
            assert!(json.contains("Lune de miel"));
            assert!(json.contains("Fruits à coque"));
            assert!(json.contains("Champagne au frais"));
        });
}

#[test]
#[serial]
fn host_stay_surface_missing_stay_id() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_stay(ctx).expect("host stay");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.missingStay"));
        });
}

#[test]
#[serial]
fn publish_readiness_recommends_one_question() {
    let config = json!({
        "ask_arrival_time": false,
        "ask_occasion": false,
        "ask_allergies": false,
        "ask_guest_count": false
    });
    MockContext::host()
        .with_property(Property::default())
        .with_config(&config)
        .run(|ctx| {
            let item = &publish_readiness(ctx).expect("publishReadiness").items[0];
            assert_eq!(item.id, "questions");
            assert!(!item.ok);
        });
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            assert!(publish_readiness(ctx).expect("publishReadiness").items[0].ok);
        });
}
