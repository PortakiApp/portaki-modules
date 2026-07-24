//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use chrono::{Duration, Utc};
use portaki_sdk::prelude::StayContext;
use portaki_test_utils::{MockContext, Property};
use pre_arrival_form::{
    get_status, load_config, render_guest_form, render_home_card, render_host_main,
    render_host_stay, reset_test_store, send_form_available, submit, update_config, ShowWhen,
    SubmitArgs, UpdateConfigArgs,
};
use serde_json::json;

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::Text(_) if type_name == "Text" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::TimePicker(_) if type_name == "TimePicker" => true,
            Component::TextArea(_) if type_name == "TextArea" => true,
            Component::ChoiceList(_) if type_name == "ChoiceList" => true,
            Component::ToggleRow(_) if type_name == "ToggleRow" => true,
            Component::Grid(_) if type_name == "Grid" => true,
            Component::Pill(_) if type_name == "Pill" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::HostFragment(_) if type_name == "HostFragment" => true,
            Component::ChecklistItem(_) if type_name == "ChecklistItem" => true,
            _ => false,
        };
        if matches {
            return true;
        }
        for child in child_components(node) {
            if walk(child, type_name) {
                return true;
            }
        }
        false
    }
    walk(&surface.root, type_name)
}

fn child_components(node: &Component) -> Vec<&Component> {
    match node {
        Component::Stack(inner) => inner.children.iter().collect(),
        Component::Card(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
        Component::ListItem(inner) => inner.children.iter().collect(),
        Component::Grid(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

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
            let surface = render_home_card(ctx.clone());
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "HostFragment"));
            assert!(contains_component_type(&surface, "ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.pending"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("guest.form"));
            assert!(!json.contains("TimePicker"));

            let form = render_guest_form(ctx);
            assert!(contains_component_type(&form, "Form"));
            assert!(contains_component_type(&form, "TimePicker"));
            assert!(contains_component_type(&form, "TextArea"));
            assert!(contains_component_type(&form, "Button"));
            // Overlay chrome owns framing — no nested Card around the form.
            assert!(!contains_component_type(&form, "Card"));
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
        .run(|ctx| {
            let before = get_status(ctx.clone()).expect("status");
            assert!(!before.completed);

            submit(ctx.clone(), sample_submit()).expect("submit");

            let after = get_status(ctx.clone()).expect("status after");
            assert!(after.completed);
            assert_eq!(after.arrival_time_estimated.as_deref(), Some("17:30"));
            assert_eq!(after.guest_occasion.as_deref(), Some("Anniversaire"));

            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "ListItem"));
            assert!(contains_component_type(&surface, "HostFragment"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.allReady"));
            assert!(json.contains("home.task.completed"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("check-circle"));
            assert!(!json.contains("TimePicker"));
        });
}

#[test]
#[serial]
fn home_card_gated_omits_form_teaser_keeps_police_fragment() {
    reset_test_store();
    let config_bytes = serde_json::to_vec(&json!({
        "show_when": "before",
        "questions": {
            "ask_arrival_time": true,
            "ask_occasion": true,
            "ask_allergies": true,
            "ask_guest_count": true,
            "ask_special_needs": false,
            "ask_id_document": false
        }
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_kv("config", config_bytes)
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
            });

            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "HostFragment"));
            // Gated: no form ListItem / soon teaser — police fragment only.
            assert!(!contains_component_type(&surface, "ListItem"));
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
    let config_bytes = serde_json::to_vec(&json!({
        "show_when": "checkin",
        "questions": {}
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_kv("config", config_bytes)
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
            });
            send_form_available(ctx, portaki_sdk::prelude::EmptyArgs {})
                .expect("sendFormAvailable gated no-op");
        });
}

#[test]
#[serial]
fn send_form_available_ok_when_confirm() {
    reset_test_store();
    let config_bytes = serde_json::to_vec(&json!({
        "show_when": "confirm",
        "questions": {}
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_kv("config", config_bytes)
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
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "Page"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "ChoiceList"));
            assert!(contains_component_type(&surface, "ToggleRow"));
            assert!(contains_component_type(&surface, "Grid"));
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
fn update_config_persists_show_when_and_questions() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        update_config(
            ctx,
            UpdateConfigArgs {
                show_when: "checkin".into(),
                ask_arrival_time: Some(true),
                ask_occasion: Some(false),
                ask_allergies: Some(true),
                ask_guest_count: Some(false),
                ask_special_needs: Some(true),
                ask_id_document: Some(true),
            },
        )
        .expect("updateConfig");

        let cfg = load_config().expect("config");
        assert_eq!(cfg.show_when, ShowWhen::Checkin);
        assert!(cfg.questions.ask_arrival_time);
        assert!(!cfg.questions.ask_occasion);
        assert!(cfg.questions.ask_allergies);
        assert!(!cfg.questions.ask_guest_count);
        assert!(cfg.questions.ask_special_needs);
        assert!(cfg.questions.ask_id_document);
    });
}

#[test]
#[serial]
fn update_config_false_toggles_stick_and_empty_keeps_kv() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        update_config(
            ctx.clone(),
            UpdateConfigArgs {
                show_when: "confirm".into(),
                ask_arrival_time: Some(false),
                ask_occasion: Some(false),
                ask_allergies: Some(true),
                ask_guest_count: Some(false),
                ask_special_needs: Some(false),
                ask_id_document: Some(false),
            },
        )
        .expect("seed");

        // Host Save with empty `{}` (formApiRef miss) must not reset toggles ON.
        let empty: UpdateConfigArgs = serde_json::from_value(json!({})).expect("empty args");
        update_config(ctx.clone(), empty).expect("empty updateConfig");

        let cfg = load_config().expect("config");
        assert_eq!(cfg.show_when, ShowWhen::Confirm);
        assert!(!cfg.questions.ask_arrival_time);
        assert!(!cfg.questions.ask_occasion);
        assert!(cfg.questions.ask_allergies);
        assert!(!cfg.questions.ask_guest_count);

        // Wire JSON with explicit false (dashboard nestFlatFormValues).
        let from_json: UpdateConfigArgs = serde_json::from_value(json!({
            "show_when": "before",
            "ask_arrival_time": false,
            "ask_occasion": true,
            "ask_allergies": false,
            "ask_guest_count": true,
            "ask_special_needs": false,
            "ask_id_document": false
        }))
        .expect("json args");
        update_config(ctx, from_json).expect("json updateConfig");

        let cfg = load_config().expect("config");
        assert_eq!(cfg.show_when, ShowWhen::Before);
        assert!(!cfg.questions.ask_arrival_time);
        assert!(cfg.questions.ask_occasion);
        assert!(!cfg.questions.ask_allergies);
        assert!(cfg.questions.ask_guest_count);
    });
}

#[test]
#[serial]
fn guest_form_respects_question_toggles() {
    reset_test_store();
    let config_bytes = serde_json::to_vec(&json!({
        "show_when": "confirm",
        "questions": {
            "ask_arrival_time": true,
            "ask_occasion": false,
            "ask_allergies": true,
            "ask_guest_count": false,
            "ask_special_needs": true,
            "ask_id_document": true
        }
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_kv("config", config_bytes)
        .run(|ctx| {
            let surface = render_guest_form(ctx);
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
            let surface = render_host_stay(ctx);
            assert!(contains_component_type(&surface, "Page"));
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Pill"));
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
            let surface = render_host_stay(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Pill"));
            assert!(contains_component_type(&surface, "ListItem"));
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
            let surface = render_host_stay(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.missingStay"));
        });
}
