//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::limits;
use serial_test::serial;

use guest_reviews::{
    publish_readiness, render_home_card, render_host_main, render_host_stats,
    render_post_stay_card, stats_summary, submit_review, ChannelMode, ModuleConfig,
    SubmitReviewArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
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

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        channel_mode: ChannelMode::Manual,
        platform_airbnb: true,
        platform_portaki: true,
        show_qr_code: true,
        airbnb_review_url: "https://www.airbnb.com/users/review/test".into(),
        thank_you_message: I18nText::new("Merci !", ""),
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
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("guest card"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_migrates_legacy_both_channel() {
    // No `moduleConfig` yet: the KV blob, read through `legacy`.
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
            let surface = render_home_card(ctx).expect("guest card");
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
            let surface = render_home_card(ctx).expect("guest card");
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
            let surface = render_home_card(ctx).expect("guest card");
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
            let surface = render_home_card(ctx).expect("guest card");
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
            let home = serde_json::to_string(&render_home_card(ctx.clone()).expect("guest card"))
                .expect("home json");
            let post = serde_json::to_string(&render_post_stay_card(ctx).expect("guest card"))
                .expect("post-stay json");
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
fn submit_review_stores_one_review_per_stay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
        .with_stay(Booking::default())
        .run_with(|ctx, host| {
            let review = || SubmitReviewArgs {
                rating: 5,
                comment: "Great".into(),
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
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
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
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
        .run(|ctx| {
            let err = submit_review(
                ctx,
                SubmitReviewArgs {
                    rating: 5,
                    comment: "Great".into(),
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
            assert!(SurfaceAssertions::new(&surface).contains_type("ToggleRow"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
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
        "airbnb_review_url": "https://www.airbnb.com/users/review/test"
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
            "platform_airbnb": false,
            "thank_you_message": { "fr": "Merci !", "en": "Thanks!" }
        }))
        .run(|mut ctx| {
            ctx.locale = "en-GB".into();
            let json = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(json.contains("Thanks!"));
            assert!(!json.contains("Merci !"));
        });
}

/// No platform ticked: blocks. Airbnb selected without its URL: recommended, never blocking.
#[test]
#[serial]
fn publish_readiness_requires_a_platform_and_recommends_the_airbnb_url() {
    let check = |config: serde_json::Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                publish_readiness(ctx)
                    .expect("publishReadiness")
                    .items
                    .into_iter()
                    .map(|item| (item.id, item.level, item.ok))
                    .collect::<Vec<_>>()
            })
    };
    let platform = |ok| ("platform".to_string(), PublishLevel::Required, ok);
    let url = |ok| {
        (
            "airbnb-review-url".to_string(),
            PublishLevel::Recommended,
            ok,
        )
    };

    assert_eq!(
        check(json!({ "platform_airbnb": false, "platform_portaki": false })),
        vec![platform(false)]
    );
    assert_eq!(
        check(json!({ "platform_airbnb": false, "platform_portaki": true })),
        vec![platform(true)]
    );
    assert_eq!(
        check(json!({ "platform_airbnb": true, "airbnb_review_url": "" })),
        vec![platform(true), url(false)]
    );
    assert_eq!(
        check(json!({ "platform_airbnb": true, "airbnb_review_url": "https://airbnb.com/r/1" })),
        vec![platform(true), url(true)]
    );
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
        .with_stay(Booking::default())
        .run_with(|ctx, host| {
            submit_review(
                ctx.clone(),
                SubmitReviewArgs {
                    rating: 5,
                    comment: comment.clone(),
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
        .with_config(&json!({
            "platform_airbnb": false,
            "platform_portaki": true
        }))
        .run(|ctx| {
            for (rating, comment) in [(5, "Très propre, super emplacement"), (4, "")] {
                let mut on_stay = ctx.clone();
                on_stay.stay = Some(Booking::default().into());
                submit_review(
                    on_stay,
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
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
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
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
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
        .with_config(&json!({ "platform_airbnb": false, "platform_portaki": true }))
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
