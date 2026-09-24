//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::limits;
use serial_test::serial;

use guest_reviews::{
    get_config, render_home_card, render_host_stats, render_post_stay_card, stats_summary,
    submit_review, update_config, SubmitReviewArgs, UpdateConfigArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_test_utils::{MockContext, SurfaceAssertions};
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
            assert!(SurfaceAssertions::new(&render_home_card(ctx)).contains_type("EmptyState"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("QRCode"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("QRCode"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(!json.contains("openOverlay"));
        });
}

#[test]
#[serial]
fn post_stay_card_reuses_home_card_content() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let home = serde_json::to_string(&render_home_card(ctx.clone())).expect("home json");
            let post = serde_json::to_string(&render_post_stay_card(ctx)).expect("post-stay json");
            assert_eq!(home, post);
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
            assert!(SurfaceAssertions::new(&surface).contains_type("ToggleRow"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
        });
}

/// A 20 000-char comment: the stored review keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_comment_is_stored_whole_and_quoted_in_the_host_email() {
    let comment = format!("{}!", "parfait ".repeat(2_500).trim_end());
    assert_eq!(comment.chars().count(), 20_000);

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
        .run_with(|ctx, host| {
            submit_review(
                ctx.clone(),
                SubmitReviewArgs {
                    rating: 5,
                    comment: comment.clone(),
                },
            )
            .expect("submit");

            let stored: Vec<SubmitReviewArgs> = serde_json::from_slice(
                &portaki_sdk::host::kv::get("reviews")
                    .expect("kv")
                    .expect("reviews"),
            )
            .expect("reviews json");
            assert_eq!(stored.last().expect("review").comment, comment);

            let email = host.sent_emails().into_iter().last().expect("host email");
            let body = &email.content.body.fr;
            assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
            let quoted = body
                .split("\n\n")
                .find(|part| part.ends_with('…'))
                .expect("quoted comment");
            let kept = quoted.trim_end_matches('…');
            assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
            assert!(comment.starts_with(kept));

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());
        });
}

#[test]
#[serial]
fn stored_reviews_feed_the_stats() {
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
            for (rating, comment) in [(5, "Très propre, super emplacement"), (4, "")] {
                submit_review(
                    ctx.clone(),
                    SubmitReviewArgs {
                        rating,
                        comment: comment.into(),
                    },
                )
                .expect("submit");
            }
            let tile = stats_summary(
                ctx.clone(),
                StatsSummaryArgs {
                    property_id: ctx.property_id,
                    period: 30,
                    key: "reviews".into(),
                },
            )
            .expect("statsSummary");
            assert_eq!(tile.value, "4,5 ★");
            assert_eq!(tile.label.fr, "2 avis · 30 j");

            let detail = serde_json::to_string(&render_host_stats(ctx)).expect("json");
            assert!(detail.contains("FeedItem"));
            assert!(detail.contains("i18n:stats.themes.cleanliness"));
            assert!(detail.contains("i18n:stats.themes.location"));
            assert!(!detail.contains("i18n:stats.themes.noise"));
        });
}
