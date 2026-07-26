//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use guest_reviews::{
    get_config, render_home_card, submit_review, update_config, SubmitReviewArgs, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "platform_airbnb": true,
        "platform_portaki": true,
        "show_qr_code": true,
        "airbnb_review_url": "https://www.airbnb.com/users/review/test",
        "thank_you_message": "Merci !"
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::QRCode(_) if type_name == "QRCode" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Select(_) if type_name == "Select" => true,
            Component::ToggleRow(_) if type_name == "ToggleRow" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
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
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Grid(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_empty_for_airbnb_without_url() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "platform_airbnb": true,
                "platform_portaki": false,
                "airbnb_review_url": ""
            }))
            .unwrap(),
        )
        .run(|ctx| {
            assert!(contains_component_type(
                &render_home_card(ctx),
                "EmptyState"
            ));
        });
}

#[test]
#[serial]
fn home_card_migrates_legacy_both_channel() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "review_channel": "both",
                "show_qr_code": true,
                "airbnb_review_url": "https://www.airbnb.com/users/review/legacy",
                "thank_you_message": "Merci !"
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "QRCode"));
            assert!(contains_component_type(&surface, "Form"));
        });
}

#[test]
#[serial]
fn home_card_airbnb_only_skips_portaki_form() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "platform_airbnb": true,
                "platform_portaki": false,
                "airbnb_review_url": "https://www.airbnb.com/users/review/test",
                "show_qr_code": false
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Button"));
            assert!(!contains_component_type(&surface, "Form"));
            assert!(!contains_component_type(&surface, "QRCode"));
        });
}

#[test]
#[serial]
fn home_card_portaki_only_when_airbnb_url_missing() {
    // Both selected, but Airbnb not feasible without URL → Portaki only.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "platform_airbnb": true,
                "platform_portaki": true,
                "airbnb_review_url": ""
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Form"));
            assert!(!contains_component_type(&surface, "QRCode"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(!json.contains("airbnb.com"));
            assert!(!json.contains("guest.airbnbCta"));
        });
}

#[test]
#[serial]
fn home_card_inline_both_platforms() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "QRCode"));
            assert!(contains_component_type(&surface, "Form"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(!json.contains("openOverlay"));
        });
}

#[test]
#[serial]
fn submit_review_validates_rating() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "platform_airbnb": false,
                "platform_portaki": true
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let err = submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 0,
                    comment: "".into(),
                },
            );
            assert!(err.is_err());
        });
}

#[test]
#[serial]
fn submit_review_rejects_when_portaki_disabled() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "platform_airbnb": true,
                "platform_portaki": false,
                "airbnb_review_url": "https://www.airbnb.com/users/review/test"
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let err = submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                },
            );
            assert!(err.is_err());
        });
}

#[test]
#[serial]
fn update_config_requires_airbnb_url_when_selected() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let err = update_config(
                ctx,
                UpdateConfigArgs {
                    platform_airbnb: Some(true),
                    platform_portaki: Some(false),
                    review_channel: String::new(),
                    show_qr_code: Some(true),
                    airbnb_review_url: String::new(),
                    thank_you_message: "Thanks".into(),
                },
            );
            assert!(err.is_err());
        });
}

#[test]
#[serial]
fn update_config_requires_at_least_one_platform() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let err = update_config(
                ctx,
                UpdateConfigArgs {
                    platform_airbnb: Some(false),
                    platform_portaki: Some(false),
                    review_channel: String::new(),
                    show_qr_code: None,
                    airbnb_review_url: String::new(),
                    thank_you_message: String::new(),
                },
            );
            assert!(err.is_err());
        });
}

#[test]
#[serial]
fn submit_review_and_update_config() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    platform_airbnb: Some(false),
                    platform_portaki: Some(true),
                    review_channel: String::new(),
                    show_qr_code: Some(false),
                    airbnb_review_url: String::new(),
                    thank_you_message: "Thanks".into(),
                },
            )
            .expect("update");
            let cfg = get_config(ctx.clone()).expect("cfg");
            assert!(!cfg.platform_airbnb);
            assert!(cfg.platform_portaki);
            assert_eq!(cfg.thank_you_message.get("fr"), "Thanks");

            submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                },
            )
            .expect("submit");
        });
}

#[test]
#[serial]
fn host_main_renders_platform_toggles() {
    use guest_reviews::render_host_main;

    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "ToggleRow"));
            assert!(contains_component_type(&surface, "Card"));
        });
}
