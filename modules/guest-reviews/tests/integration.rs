//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::limits;
use portaki_sdk::prelude::{DateTime, Utc};
use serial_test::serial;

use guest_reviews::{
    publish_readiness, render_home_card, render_host_main, render_host_stats, render_host_stay,
    render_post_stay_card, render_property_public, stats_summary, submit_review, AskFrom,
    ModuleConfig, SubmitReviewArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::publish::PublishLevel;
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_test_utils::{Booking, MockContext, SurfaceAssertions};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");
const AIRBNB_URL: &str = "https://www.airbnb.com/users/review/test";

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        review_url: AIRBNB_URL.into(),
        thank_you_message: I18nText::new("Merci !", ""),
        ..ModuleConfig::default()
    }
}

fn at(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .expect("instant")
        .with_timezone(&Utc)
}

/// A stay booked on `channel`, with the default `Booking` dates.
fn stay_on(channel: &str) -> StayContext {
    let mut stay: StayContext = Booking::default().into();
    stay.booking_channel = Some(channel.into());
    stay
}

fn to_json(surface: &impl serde::Serialize) -> String {
    serde_json::to_string(surface).expect("json")
}

#[test]
#[serial]
fn without_a_link_the_card_offers_the_rating_alone() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "review_url": "" }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            let assertions = SurfaceAssertions::new(&surface);
            assert!(assertions.contains_type("Form"));
            assert!(assertions.contains_type("ChoiceList"));
            assert!(!assertions.contains_type("QRCode"));
            assert!(!assertions.contains_type("EmptyState"));
            let json = to_json(&surface);
            assert!(!json.contains("guest.cta."), "{json}");
            assert!(!json.contains("guest.orPortaki"), "{json}");
        });
}

#[test]
#[serial]
fn home_card_migrates_the_legacy_kv_config() {
    // No `moduleConfig` yet: the KV blob, read through `legacy`.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "review_channel": "both",
                "show_qr_code": false,
                "airbnb_review_url": "https://www.airbnb.com/users/review/legacy",
                "thank_you_message": "Merci !"
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // `show_qr_code` is gone: the QR comes with the link.
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = to_json(&surface);
            assert!(json.contains("i18n:guest.cta.airbnb"), "{json}");
            assert!(json.contains("users/review/legacy"), "{json}");
        });
}

#[test]
#[serial]
fn with_a_link_the_card_shows_the_button_the_qr_and_the_rating() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            let assertions = SurfaceAssertions::new(&surface);
            assert!(assertions.contains_type("Card"));
            assert!(assertions.contains_type("QRCode"));
            assert!(assertions.contains_type("Form"));
            assert!(assertions.contains_type("TextArea"));
            let json = to_json(&surface);
            assert!(json.contains("i18n:guest.cta.airbnb"), "{json}");
            assert!(json.contains(AIRBNB_URL), "{json}");
            assert!(json.contains("i18n:guest.orPortaki"), "{json}");
            assert!(!json.contains("openOverlay"));
        });
}

/// The button names the platform read on the link's domain.
#[test]
#[serial]
fn the_button_names_the_platform_of_the_link() {
    for (url, key) in [
        (
            "https://www.booking.com/hotel/fr/x.html",
            "i18n:guest.cta.booking",
        ),
        ("https://g.page/r/abc/review", "i18n:guest.cta.google"),
        (
            "https://www.abritel.fr/location/1",
            "i18n:guest.cta.abritel",
        ),
        ("https://www.tripadvisor.fr/x", "i18n:guest.cta.tripadvisor"),
        ("https://example.com/avis", "i18n:guest.cta.other"),
    ] {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({ "review_url": url }))
            .run(|ctx| {
                let json = to_json(&render_home_card(ctx).expect("guest card"));
                assert!(json.contains(key), "{url}: {json}");
                assert!(json.contains(url), "{url}: {json}");
            });
    }
}

#[test]
#[serial]
fn without_private_comment_the_text_area_is_gone() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "review_url": AIRBNB_URL, "private_comment": false }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            let assertions = SurfaceAssertions::new(&surface);
            assert!(assertions.contains_type("Form"));
            assert!(assertions.contains_type("ChoiceList"));
            assert!(!assertions.contains_type("TextArea"));
            assert!(!to_json(&surface).contains("i18n:guest.comment"));
        });
}

/// In `auto` mode an Airbnb link is not offered to a guest who booked on Booking.com — the
/// rating still is.
#[test]
#[serial]
fn auto_mode_hides_a_booking_platform_link_from_other_stays() {
    let config = json!({ "review_url": AIRBNB_URL, "channel_mode": "auto" });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .with_stay(stay_on("booking"))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("QRCode"));
            assert!(!to_json(&surface).contains(AIRBNB_URL));
        });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .with_stay(stay_on("airbnb"))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            assert!(to_json(&surface).contains(AIRBNB_URL));
        });
}

/// Filtering the public link on the guest's rating is forbidden: a one-star guest still gets it.
#[test]
#[serial]
fn a_low_rating_still_gets_the_public_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .run(|ctx| {
            submit_review(
                ctx.clone(),
                SubmitReviewArgs {
                    rating: 1,
                    comment: "Décevant".into(),
                    public_consent: false,
                },
            )
            .expect("submit");
            let surface = render_home_card(ctx).expect("guest card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Celebration"));
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            let json = to_json(&surface);
            assert!(json.contains("i18n:guest.cta.airbnb"), "{json}");
            assert!(json.contains(AIRBNB_URL), "{json}");
        });
}

/// Before arrival the home card says when it opens; no form.
#[test]
#[serial]
fn the_home_card_waits_for_the_arrival() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-05-20T12:00:00Z"))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest card");
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(to_json(&surface).contains("i18n:guest.notYet"));
        });
}

#[test]
#[serial]
fn post_stay_card_reuses_home_card_content() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-10T12:00:00Z"))
        .run(|ctx| {
            let home = to_json(&render_home_card(ctx.clone()).expect("guest card"));
            let post = to_json(&render_post_stay_card(ctx).expect("guest card"));
            assert_eq!(home, post);
        });
}

/// « Le lendemain à 10 h » : the post-stay card stays an empty state until 10:00 the day after
/// checkout, property time. `Booking::default` checks out at 2026-06-08T10:00Z (12:00 Paris):
/// the card opens 2026-06-09T10:00 Paris, 08:00Z.
#[test]
#[serial]
fn the_post_stay_card_waits_for_ask_from() {
    let card = |now: &str| {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&ModuleConfig {
                ask_from: AskFrom::NextDay10h,
                ..sample_config()
            })
            .with_stay(Booking::default())
            .with_now(at(now))
            .run(|ctx| render_post_stay_card(ctx).expect("post-stay card"))
    };
    let early = card("2026-06-09T07:59:00Z");
    assert!(SurfaceAssertions::new(&early).contains_type("EmptyState"));
    assert!(!SurfaceAssertions::new(&early).contains_type("Form"));
    let open = card("2026-06-09T08:00:00Z");
    assert!(!SurfaceAssertions::new(&open).contains_type("EmptyState"));
    assert!(SurfaceAssertions::new(&open).contains_type("Form"));

    // « L'heure de départ » : open from checkout.
    let at_checkout = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-08T10:00:00Z"))
        .run(|ctx| render_post_stay_card(ctx).expect("post-stay card"));
    assert!(SurfaceAssertions::new(&at_checkout).contains_type("Form"));
}

#[test]
#[serial]
fn submit_review_validates_rating() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run(|ctx| {
            for rating in [0, 6] {
                let err = submit_review(
                    ctx.clone(),
                    SubmitReviewArgs {
                        rating,
                        comment: "".into(),
                        public_consent: false,
                    },
                );
                assert!(err.is_err(), "{rating}");
            }
        });
}

/// No more Portaki switch: with a public link, the rating is still taken.
#[test]
#[serial]
fn submit_review_is_taken_alongside_a_public_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .run(|ctx| {
            submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                    public_consent: false,
                },
            )
            .expect("submit");
        });
}

#[test]
#[serial]
fn submit_review_stores_one_review_per_stay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run_with(|ctx, host| {
            let review = || SubmitReviewArgs {
                rating: 5,
                comment: "Great".into(),
                public_consent: false,
            };
            submit_review(ctx.clone(), review()).expect("submit");
            let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
            assert!(
                portaki_sdk::host::kv::get(&format!("stay:{stay_id}:review"))
                    .expect("kv")
                    .is_some()
            );
            assert!(portaki_sdk::host::kv::get("reviews").expect("kv").is_none());

            let again = submit_review(ctx, review()).expect_err("second review");
            assert!(again.to_string().contains("review_already_submitted"));
            assert_eq!(host.sent_emails().len(), 1);
        });
}

#[test]
#[serial]
fn a_review_under_the_first_per_stay_key_still_counts() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run_with(|ctx, host| {
            let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
            portaki_sdk::host::kv::set(
                &format!("review:{stay_id}"),
                br#"{"rating":4,"comment":"Bien"}"#,
                None,
            )
            .expect("kv");

            let again = submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                    public_consent: false,
                },
            )
            .expect_err("already reviewed");
            assert!(again.to_string().contains("review_already_submitted"));
            assert!(host.sent_emails().is_empty());
        });
}

#[test]
#[serial]
fn submit_review_needs_a_stay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let err = submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
                    public_consent: false,
                },
            )
            .expect_err("no stay");
            assert!(err.to_string().contains("review_needs_stay"));
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
            let assertions = SurfaceAssertions::new(&surface);
            assert!(assertions.contains_type("ToggleRow"));
            assert!(assertions.contains_type("ChoiceList"));
            assert!(assertions.contains_type("TextInput"));
            assert!(assertions.contains_type("Card"));
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            // The detected platform, read-only.
            assert!(to_json(&surface).contains("\"Airbnb\""));
        });
}

/// A link that is not https: the error sits under the field, in the host's language.
#[test]
#[serial]
fn the_host_form_shows_the_https_error_under_the_link() {
    let error_of = |url: &str| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({ "review_url": url }))
            .run(|mut ctx| {
                ctx.locale = "fr-FR".into();
                let tree =
                    serde_json::to_value(render_host_main(ctx).expect("host main")).expect("tree");
                field_error(&tree, "review_url")
            })
    };
    assert_eq!(
        error_of("http://www.booking.com/x").as_deref(),
        Some("L'adresse doit commencer par https://")
    );
    assert!(error_of("www.booking.com/x").is_some());
    assert_eq!(error_of("https://www.booking.com/x"), None);
    assert_eq!(error_of(""), None);
}

/// Before `review_url` is saved, the old Airbnb link fills the field.
#[test]
#[serial]
fn the_host_form_shows_the_old_airbnb_link() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "airbnb_review_url": "airbnb.fr/users/review/1" }))
        .run(|ctx| {
            let tree =
                serde_json::to_value(render_host_main(ctx).expect("host main")).expect("tree");
            assert_eq!(
                form_value(&tree, "review_url").as_deref(),
                Some("https://airbnb.fr/users/review/1")
            );
        });
}

/// The form shows the message in the host's language; a save in that language keeps the others.
#[test]
#[serial]
fn host_form_shows_the_message_in_the_host_language() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["thank_you_message"]
    );
    let stored = json!({
        "thank_you_message": { "fr": "Merci !", "en": "Thanks!" },
        "review_url": AIRBNB_URL
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            assert_eq!(
                config_save::form_args(&surface)["thank_you_message"],
                "Thanks!"
            );
            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["thank_you_message"], stored["thank_you_message"]);
        });
}

/// The guest reads the message in their own language.
#[test]
#[serial]
fn guest_card_shows_the_message_in_the_guest_language() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "thank_you_message": { "fr": "Merci !", "en": "Thanks!" }
        }))
        .run(|mut ctx| {
            ctx.locale = "en-GB".into();
            let json = to_json(&render_home_card(ctx).expect("card"));
            assert!(json.contains("Thanks!"));
            assert!(!json.contains("Merci !"));
        });
}

/// One rule: a link that is not https blocks. No link is fine — the rating alone.
#[test]
#[serial]
fn publish_readiness_requires_an_https_link() {
    let check = |config: serde_json::Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                publish_readiness(ctx)
                    .expect("publishReadiness")
                    .items
                    .into_iter()
                    .map(|item| (item.id, item.level, item.ok, item.hint.fr))
                    .collect::<Vec<_>>()
            })
    };
    let blocked = vec![(
        "config.review_url".to_string(),
        PublishLevel::Required,
        false,
        "L'adresse doit commencer par https://".to_string(),
    )];

    assert_eq!(check(json!({})), vec![]);
    assert_eq!(check(json!({ "review_url": "" })), vec![]);
    assert_eq!(check(json!({ "review_url": AIRBNB_URL })), vec![]);
    assert_eq!(
        check(json!({ "review_url": "http://www.booking.com/x" })),
        blocked
    );
    assert_eq!(check(json!({ "review_url": "booking.com/x" })), blocked);
}

/// The stay encart: not rated yet, the rating, and whether the public link was offered.
#[test]
#[serial]
fn the_stay_encart_says_the_rating() {
    let encart = |config: serde_json::Value, rating: Option<u8>| {
        let stay_id = "00000000-0000-4000-8000-000000000001";
        let mut builder = MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .with_translation("host.stay.notYet", "Pas encore noté.")
            .with_translation("host.stay.rated", "Note : {rating} / 5")
            .with_translation(
                "host.stay.ratedWithLink",
                "Note : {rating} / 5 · lien public proposé",
            );
        if let Some(rating) = rating {
            builder = builder.with_kv(
                format!("stay:{stay_id}:review"),
                serde_json::to_vec(&json!({ "rating": rating, "comment": "" })).unwrap(),
            );
        }
        builder.run(|mut ctx| {
            ctx.input = json!({ "stayId": stay_id });
            to_json(&render_host_stay(ctx).expect("stay encart"))
        })
    };
    assert!(encart(json!({}), None).contains("i18n:host.stay.notYet"));
    let rated = encart(json!({}), Some(4));
    assert!(rated.contains("Note : 4 / 5"), "{rated}");
    assert!(!rated.contains("lien public"), "{rated}");
    let with_link = encart(json!({ "review_url": AIRBNB_URL }), Some(5));
    assert!(
        with_link.contains("Note : 5 / 5 · lien public proposé"),
        "{with_link}"
    );
    // Without a stay id: nothing to read, not rated.
    let no_stay = MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| to_json(&render_host_stay(ctx).expect("stay encart")));
    assert!(no_stay.contains("i18n:host.stay.notYet"));
}

/// The error a `Field` named `name` shows, if any.
fn field_error(tree: &serde_json::Value, name: &str) -> Option<String> {
    match tree {
        serde_json::Value::Object(object) => {
            if object.get("name").and_then(serde_json::Value::as_str) == Some(name) {
                if let Some(error) = object.get("error").and_then(serde_json::Value::as_str) {
                    return Some(error.to_string());
                }
            }
            object.values().find_map(|child| field_error(child, name))
        }
        serde_json::Value::Array(items) => items.iter().find_map(|item| field_error(item, name)),
        _ => None,
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
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run_with(|ctx, host| {
            submit_review(
                ctx.clone(),
                SubmitReviewArgs {
                    rating: 5,
                    comment: comment.clone(),
                    public_consent: false,
                },
            )
            .expect("submit");

            let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
            let stored: SubmitReviewArgs = serde_json::from_slice(
                &portaki_sdk::host::kv::get(&format!("stay:{stay_id}:review"))
                    .expect("kv")
                    .expect("review"),
            )
            .expect("review json");
            assert_eq!(stored.comment, comment);

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
        .with_config(&json!({}))
        .run(|ctx| {
            for (rating, comment) in [(5, "Très propre, super emplacement"), (4, "")] {
                let mut on_stay = ctx.clone();
                on_stay.stay = Some(Booking::default().into());
                submit_review(
                    on_stay,
                    SubmitReviewArgs {
                        rating,
                        comment: comment.into(),
                        public_consent: false,
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

/// La note que le formulaire envoie doit arriver au module.
///
/// Tout ce qui sort d'un formulaire HTML est une chaîne : l'étoile touchée envoie `"5"`,
/// `collectFormValues` rend un `Record<string, string>`, et la plateforme passe les arguments
/// tels quels — rien ne les convertit d'après le type déclaré. Un `u8` nu les faisait refuser
/// par serde, et le voyageur n'enregistrait jamais sa note.
#[test]
#[serial]
fn the_rating_the_form_sends_reaches_the_command() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run(|ctx| {
            let tree = serde_json::to_value(render_home_card(ctx.clone()).expect("home card"))
                .expect("tree");
            // La valeur de la cinquième étoile : rien n'est coché d'avance, c'est le doigt du
            // voyageur qui la pose.
            let sent = star_choice(&tree, 5).expect("la cinquième étoile");

            // Ce que le navigateur poste : la valeur du champ, en chaîne.
            let args: SubmitReviewArgs =
                serde_json::from_value(json!({ "rating": sent, "comment": "Super séjour" }))
                    .expect("les arguments que le formulaire envoie");
            submit_review(ctx.clone(), args).expect("submit");

            let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
            let stored = portaki_sdk::host::kv::get(&format!("stay:{stay_id}:review"))
                .expect("kv")
                .expect("la note enregistrée");
            let stored: serde_json::Value = serde_json::from_slice(&stored).expect("json");
            assert_eq!(stored["rating"], 5);
        });
}

/// La valeur de la n-ième étoile, telle que le livret la posterait.
fn star_choice(tree: &serde_json::Value, nth: usize) -> Option<String> {
    fn walk(node: &serde_json::Value, nth: usize) -> Option<String> {
        if let serde_json::Value::Object(object) = node {
            if object.get("layout").and_then(serde_json::Value::as_str) == Some("stars") {
                return object
                    .get("choices")?
                    .as_array()?
                    .get(nth - 1)?
                    .get("value")?
                    .as_str()
                    .map(str::to_string);
            }
        }
        match node {
            serde_json::Value::Object(object) => object.values().find_map(|child| walk(child, nth)),
            serde_json::Value::Array(items) => items.iter().find_map(|item| walk(item, nth)),
            _ => None,
        }
    }
    walk(tree, nth)
}

/// La valeur d'un champ du formulaire, en chaîne — ce que `FormData` en fait côté navigateur.
fn form_value(tree: &serde_json::Value, name: &str) -> Option<String> {
    match tree {
        serde_json::Value::Object(object) => {
            if object.get("name").and_then(serde_json::Value::as_str) == Some(name) {
                if let Some(value) = object.get("value") {
                    return Some(match value {
                        serde_json::Value::String(text) => text.clone(),
                        other => other.to_string(),
                    });
                }
            }
            object.values().find_map(|child| form_value(child, name))
        }
        serde_json::Value::Array(items) => items.iter().find_map(|item| form_value(item, name)),
        _ => None,
    }
}

fn uuid_for(days: i64) -> String {
    format!("00000000-0000-4000-8000-{days:012}")
}

/// Déjà noté : le remerciement prend la place du formulaire (§2.19).
///
/// L'envoi refusait bien un second avis (`review_already_submitted`), mais la carte proposait
/// quand même les étoiles : le voyageur remplissait sa note et son mot pour recevoir une erreur.
#[test]
#[serial]
fn a_stay_already_rated_is_thanked_not_asked_again() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .with_translation("guest.given.title", "Merci pour votre avis")
        .run(|ctx| {
            let before = render_home_card(ctx.clone()).expect("home card");
            assert!(SurfaceAssertions::new(&before).contains_type("Form"));

            submit_review(
                ctx.clone(),
                SubmitReviewArgs {
                    rating: 4,
                    comment: "Très bien".into(),
                    public_consent: false,
                },
            )
            .expect("submit");

            let after = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&after).contains_type("Celebration"));
            // Plus de formulaire, plus d'étoiles : la question a reçu sa réponse.
            assert!(!SurfaceAssertions::new(&after).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&after).contains_type("ChoiceList"));
            let json = serde_json::to_string(&after).expect("json");
            assert!(json.contains("Merci pour votre avis"), "{json}");
            assert!(json.contains("💛"), "{json}");
        });
}

/// Aucune étoile n'est servie d'avance, et la note est obligatoire (§2.19).
///
/// Cinq étoiles préchoisies partaient telles quelles chez qui touchait « Envoyer » sans rien
/// noter : le module fabriquait des avis cinq étoiles que personne n'avait donnés.
#[test]
#[serial]
fn no_star_is_given_in_advance() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .with_stay(Booking::default())
        .run(|ctx| {
            let tree = serde_json::to_value(render_home_card(ctx).expect("home card")).unwrap();
            assert_eq!(form_value(&tree, "rating"), None, "{tree}");
            // Les cinq étoiles sont bien là, et le champ est obligatoire : le livret retient
            // l'envoi tant que rien n'est choisi.
            assert_eq!(star_choice(&tree, 5).as_deref(), Some("5"), "{tree}");
            let json = tree.to_string();
            assert!(json.contains(r#""required":true"#), "{json}");
        });
}

const STAY_A: &str = "00000000-0000-4000-8000-00000000000a";
const STAY_B: &str = "00000000-0000-4000-8000-00000000000b";
const STAY_C: &str = "00000000-0000-4000-8000-00000000000c";
const STAY_OLD: &str = "00000000-0000-4000-8000-00000000000d";

/// Trois avis par séjour : deux consentis, un refusé ; et un avis d'avant la case.
fn with_public_reviews(builder: MockContext) -> MockContext {
    let review = |rating: u8, comment: &str, name: &str, consent: Option<bool>| {
        let mut value = json!({
            "rating": rating, "comment": comment, "guest_name": name,
            "at": "2026-07-14T10:00:00Z",
        });
        if let Some(consent) = consent {
            value["public_consent"] = json!(consent);
        }
        serde_json::to_vec(&value).unwrap()
    };
    builder
        .with_kv(
            format!("stay:{STAY_A}:review"),
            review(5, "Maison superbe", "Sophie Lambert", Some(true)),
        )
        .with_kv(
            format!("stay:{STAY_B}:review"),
            review(4, "Plage à pied", "Thomas Girard", Some(true)),
        )
        .with_kv(
            format!("stay:{STAY_C}:review"),
            review(1, "Ne pas publier", "Zoé Dupont", Some(false)),
        )
        .with_kv(
            format!("stay:{STAY_OLD}:review"),
            review(2, "Avis ancien", "Marc Petit", None),
        )
}

fn render_public(config: serde_json::Value) -> serde_json::Value {
    with_public_reviews(MockContext::public_visitor())
        .with_config(&config)
        .with_translation("public.count", "{count} séjours notés")
        .with_translation("public.month.7", "juillet")
        .run(|ctx| {
            assert!(ctx.is_public_visitor());
            serde_json::to_value(render_property_public(ctx).expect("public")).unwrap()
        })
}

/// Le bloc public : note et nombre sur les seuls consentis, avis choisis au prénom, rien de ce
/// qui n'est pas consenti — même choisi —, ni nom de famille, ni séjour.
#[test]
#[serial]
fn the_public_block_shows_only_consented_reviews_by_first_name() {
    let json = render_public(json!({
        "public_enabled": true,
        "public_reviews": format!("[\"{STAY_A}\",\"{STAY_C}\",\"{STAY_OLD}\",\"{STAY_B}\"]"),
    }));
    assert!(portaki_sdk::surfaces::check_property_public_tree(&json["root"]).is_empty());
    assert_eq!(json["root"]["type"], "Section");
    let text = json.to_string();
    assert!(
        text.contains("\"Maison superbe\"") && text.contains("\"Plage à pied\""),
        "{text}"
    );
    assert!(text.contains("Sophie · juillet 2026"), "{text}");
    assert!(text.contains("Thomas · juillet 2026"), "{text}");
    assert!(text.contains("4,5 / 5"), "{text}");
    assert!(text.contains("2 séjours notés"), "{text}");
    for hidden in [
        "Ne pas publier",
        "Zoé",
        "Avis ancien",
        "Marc",
        "Lambert",
        "Girard",
        "00000000-",
    ] {
        assert!(!text.contains(hidden), "{hidden} leaked: {text}");
    }
}

/// Désactivé, ou moins de deux avis choisis encore consentis : une Section sans enfant.
#[test]
#[serial]
fn the_public_block_is_empty_when_disabled_or_short() {
    let empty = |json: serde_json::Value| {
        assert_eq!(json["root"]["type"], "Section", "{json}");
        assert!(
            json["root"]["children"]
                .as_array()
                .is_none_or(Vec::is_empty),
            "{json}"
        );
    };
    let both = format!("[\"{STAY_A}\",\"{STAY_B}\"]");
    empty(render_public(json!({ "public_reviews": both })));
    empty(render_public(
        json!({ "public_enabled": false, "public_reviews": both }),
    ));
    // Le refusé et l'ancien ne comptent pas : il reste un seul avis.
    empty(render_public(json!({
        "public_enabled": true,
        "public_reviews": format!("[\"{STAY_A}\",\"{STAY_C}\",\"{STAY_OLD}\"]"),
    })));
}

/// La case cochée (`"on"`) est enregistrée avec l'avis ; décochée, l'avis reste privé.
#[test]
#[serial]
fn the_consent_box_is_stored_with_the_review() {
    for (sent, consented) in [(json!("on"), true), (json!(null), false)] {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_stay(Booking::default())
            .run(|ctx| {
                let mut args = json!({ "rating": "5", "comment": "Top" });
                if !sent.is_null() {
                    args["public_consent"] = sent.clone();
                }
                submit_review(ctx.clone(), serde_json::from_value(args).unwrap()).expect("submit");
                let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
                let stored: serde_json::Value = serde_json::from_slice(
                    &portaki_sdk::host::kv::get(&format!("stay:{stay_id}:review"))
                        .unwrap()
                        .expect("stored"),
                )
                .unwrap();
                assert_eq!(stored["public_consent"], json!(consented), "{stored}");
            });
    }
}

/// La carte « Page publique » ne propose que les avis consentis, au prénom ; moins de deux
/// choisis : l'erreur sous le champ et un avertissement de publication, qui ne bloque pas.
#[test]
#[serial]
fn the_public_card_picks_among_consented_reviews() {
    let config = json!({ "public_enabled": true, "public_reviews": format!("[\"{STAY_A}\"]") });
    with_public_reviews(MockContext::host())
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let surface = render_host_main(ctx.clone()).expect("host main");
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            let text = to_json(&surface);
            assert!(text.contains(STAY_A) && text.contains(STAY_B), "{text}");
            assert!(!text.contains(STAY_C) && !text.contains(STAY_OLD), "{text}");
            assert!(
                text.contains("Sophie") && !text.contains("Lambert"),
                "{text}"
            );
            assert!(text.contains("Choisissez au moins 2 avis."), "{text}");

            let items = publish_readiness(ctx).expect("readiness").items;
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, "config.public_reviews");
            assert_eq!(items[0].level, PublishLevel::Recommended);
            assert_eq!(items[0].hint.fr, "Choisissez au moins 2 avis.");
        });
}
