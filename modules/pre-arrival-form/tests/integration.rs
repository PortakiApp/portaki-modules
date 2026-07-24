//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use portaki_test_utils::{MockContext, Property};
use pre_arrival_form::{
    get_status, load_config, render_home_card, render_host_main, render_host_stay,
    reset_test_store, submit, update_config, ShowWhen, SubmitArgs, UpdateConfigArgs,
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
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "TimePicker"));
            assert!(contains_component_type(&surface, "TextArea"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("submit"));
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
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(!json.contains("TimePicker"));
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
                ask_arrival_time: true,
                ask_occasion: false,
                ask_allergies: true,
                ask_guest_count: false,
                ask_special_needs: true,
                ask_id_document: true,
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
            let surface = render_home_card(ctx);
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
