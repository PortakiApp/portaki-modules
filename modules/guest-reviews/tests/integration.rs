//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::limits;
use serial_test::serial;

use guest_reviews::{
    publish_readiness, render_home_card, render_host_main, render_host_stats,
    render_post_stay_card, stats_summary, submit_review, ChannelMode, Localized, ModuleConfig,
    SubmitReviewArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
use portaki_sdk::contracts::publish::PublishLevel;
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        channel_mode: ChannelMode::Manual,
        platform_airbnb: true,
        platform_portaki: true,
        show_qr_code: true,
        airbnb_review_url: "https://www.airbnb.com/users/review/test".into(),
        thank_you_message: Localized::singleton("fr", "Merci !"),
    }
}

#[test]
#[serial]
fn home_card_empty_for_airbnb_without_url() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "platform_airbnb": true,
            "platform_portaki": false,
            "airbnb_review_url": ""
        }))
        .run(|ctx| {
            assert!(SurfaceAssertions::new(&render_home_card(ctx)).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_migrates_legacy_both_channel() {
    // No `moduleConfig` yet: the KV blob, read the old way.
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
        .with_config(&json!({
            "platform_airbnb": true,
            "platform_portaki": false,
            "airbnb_review_url": "https://www.airbnb.com/users/review/test",
            "show_qr_code": false
        }))
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
        .with_config(&json!({
            "platform_airbnb": true,
            "platform_portaki": true,
            "airbnb_review_url": ""
        }))
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
        .with_config(&sample_config())
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
        .with_config(&sample_config())
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
        .with_config(&json!({
            "platform_airbnb": false,
            "platform_portaki": true
        }))
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
        .with_config(&json!({
            "platform_airbnb": true,
            "platform_portaki": false,
            "airbnb_review_url": "https://www.airbnb.com/users/review/test"
        }))
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
fn submit_review_stores_the_review() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
        .run(|ctx| {
            submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                },
            )
            .expect("submit");
            assert!(portaki_sdk::host::kv::get("reviews").expect("kv").is_some());
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            assert!(SurfaceAssertions::new(&surface).contains_type("ToggleRow"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
        });
}

/// An older per-language message shows in the host's language in the one-message form.
#[test]
#[serial]
fn host_form_shows_the_message_in_the_host_language() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "thank_you_message": { "fr": "Merci !", "en": "Thanks!" } }))
        .run(|mut ctx| {
            ctx.locale = "en".into();
            let json = serde_json::to_string(&render_host_main(ctx).expect("host main")).unwrap();
            assert!(json.contains("Thanks!"));
        });
}

/// Airbnb selected without its URL: recommended, never blocking; Airbnb off: nothing.
#[test]
#[serial]
fn publish_readiness_recommends_the_airbnb_url_when_selected() {
    for (config, expected) in [
        (
            json!({ "platform_airbnb": true, "airbnb_review_url": "" }),
            Some(false),
        ),
        (
            json!({ "platform_airbnb": true, "airbnb_review_url": "https://airbnb.com/r/1" }),
            Some(true),
        ),
        (json!({ "platform_airbnb": false }), None),
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let items = publish_readiness(ctx).expect("publishReadiness").items;
                assert_eq!(items.first().map(|item| item.ok), expected, "{config}");
                assert!(items
                    .iter()
                    .all(|item| item.level == PublishLevel::Recommended));
            });
    }
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
        .with_config(&json!({
            "platform_airbnb": false,
            "platform_portaki": true
        }))
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
        .with_config(&json!({
            "platform_airbnb": false,
            "platform_portaki": true
        }))
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

            let detail = serde_json::to_string(&render_host_stats(ctx.clone())).expect("json");
            assert!(detail.contains("FeedItem"));
            assert!(detail.contains("i18n:stats.themes.cleanliness"));
            assert!(detail.contains("i18n:stats.themes.location"));
            assert!(!detail.contains("i18n:stats.themes.noise"));
            // Sans les séjours de la période, pas de taux de réponse.
            assert!(!detail.contains("i18n:stats.responseRate"));

            // Quatre départs dans la période, un avant : 2 avis sur 4 départs.
            let stays: Vec<serde_json::Value> = [2, 5, 9, 20, 45]
                .iter()
                .map(|days| {
                    json!({
                        "id": uuid_for(*days),
                        "checkIn": chrono_like_days_ago(days + 3),
                        "checkOut": chrono_like_days_ago(*days),
                        "status": "COMPLETED",
                    })
                })
                .collect();
            let mut with_stays = ctx;
            with_stays.input = json!({ "periodDays": 30, "stays": stays });
            let detail = serde_json::to_string(&render_host_stats(with_stays)).expect("json");
            assert!(detail.contains("i18n:stats.responseRate"));
            assert!(detail.contains("\"50 %\""));
        });
}

/// RFC 3339 instant `days` days before now.
#[allow(clippy::disallowed_methods)] // test natif : l'horloge du système y est disponible
fn chrono_like_days_ago(days: i64) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64
        - days * 86_400;
    portaki_sdk::prelude::DateTime::<portaki_sdk::prelude::Utc>::from_timestamp(secs, 0)
        .expect("instant")
        .to_rfc3339()
}

fn uuid_for(days: i64) -> String {
    format!("00000000-0000-4000-8000-{days:012}")
}
