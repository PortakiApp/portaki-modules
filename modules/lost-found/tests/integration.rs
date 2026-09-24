//! Integration-style unit tests with `portaki-test-utils`.

use chrono::Duration;
use serial_test::serial;
use uuid::Uuid;

use lost_found::{
    build_email_context, list_for_stay, list_recent, render_guest_form, render_home_card,
    render_host_create, render_host_main, render_host_stats, render_host_stay,
    render_post_stay_card, reset_test_store, send_checkout_follow_up, submit, submit_found,
    update_config, update_status, EmailContextArgs, ListForStayArgs, SubmitArgs, SubmitFoundArgs,
    UpdateConfigArgs, UpdateStatusArgs, GUEST_TEXT_EMAIL_MAX_CHARS, STATUS_DEFAULT,
};
use portaki_sdk::host::email::{EmailAudience, EmailError};
use portaki_sdk::limits;
use portaki_sdk::prelude::{EmailTemplateKey, PortakiError};
use portaki_sdk::sdui::action::EmptyArgs;
use portaki_test_utils::{Booking, MockContext, Property, SurfaceAssertions};

#[test]
#[serial]
fn post_stay_card_reuses_home_card_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let home = serde_json::to_string(&render_home_card(ctx.clone())).expect("home json");
            let post = serde_json::to_string(&render_post_stay_card(ctx)).expect("post-stay json");
            assert_eq!(home, post);
        });
}

#[test]
#[serial]
fn home_card_opens_form_overlay_when_no_reports() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx.clone());
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("guest.form"));
            assert!(json.contains("home.card.openForm"));

            let form = render_guest_form(ctx);
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("form.kind.label"));
        });
}

#[test]
#[serial]
fn submit_allows_multiple_reports_and_shows_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: "Blue scarf".into(),
                    contact_hint: Some("guest@example.com".into()),
                    details: Some("Left in living room".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].kind, "lost");
            assert_eq!(rows[0].item_description, "Blue scarf");
            assert_eq!(rows[0].status, STATUS_DEFAULT);

            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "found".into(),
                    item_description: "Room key".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit second");

            let rows =
                list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list after second");
            assert_eq!(rows.len(), 2);

            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

#[test]
#[serial]
fn host_stats_list_recent_after_guest_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    kind: "found".into(),
                    item_description: "Umbrella".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let recent = list_recent(ctx.clone()).expect("listRecent");
            assert_eq!(recent.len(), 1);

            let surface = render_host_stats(ctx.clone());
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("List"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Select"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.recentTitle"));
            assert!(json.contains("updateStatus") || json.contains("host.main.updateStatus"));
            // Create lives on stay surface — not the stats list.
            assert!(!json.contains("host.create.submit"));

            // The config tab keeps the note only.
            let main = serde_json::to_string(&render_host_main(ctx)).expect("main json");
            assert!(main.contains("host.main.banner"));
            assert!(!main.contains("host.main.recentTitle"));
        });
}

#[test]
#[serial]
fn host_update_status_changes_report() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![],
                    stay_id: Some(stay_id),
                    description: "Chargeur USB-C".into(),
                    status: Some("to_collect".into()),
                },
            )
            .expect("submitFound");

            let rows = list_for_stay(
                ctx.clone(),
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list");
            assert_eq!(rows.len(), 1);
            let report_id = rows[0].id;

            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id,
                    status: "sent".into(),
                },
            )
            .expect("updateStatus");

            let rows = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list after update");
            assert_eq!(rows[0].status, "sent");
        });
}

#[test]
#[serial]
fn email_context_includes_descriptions_when_declaration_exists() {
    reset_test_store();

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: "Écharpe bleue".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit");

            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::LostFound),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert!(out.has_declaration);
            assert_eq!(out.lost_item_description.as_deref(), Some("Écharpe bleue"));
        });
}

#[test]
#[serial]
fn host_submit_found_creates_report_per_stay() {
    reset_test_store();
    let stay_a = Uuid::new_v4();
    let stay_b = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![stay_a, stay_b],
                    stay_id: None,
                    description: "Chargeur MacBook oublié dans le tiroir".into(),
                    status: None,
                },
            )
            .expect("submitFound");

            let for_a = list_for_stay(
                ctx.clone(),
                ListForStayArgs {
                    stay_id: Some(stay_a),
                },
            )
            .expect("list a");
            assert_eq!(for_a.len(), 1);
            assert_eq!(for_a[0].kind, "found");
            assert_eq!(for_a[0].status, STATUS_DEFAULT);

            let for_b = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_b),
                },
            )
            .expect("list b");
            assert_eq!(for_b.len(), 1);
            assert_eq!(for_b[0].status, "to_collect");
        });
}

#[test]
#[serial]
fn host_submit_found_always_defaults_status_to_collect() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![],
                    stay_id: Some(stay_id),
                    description: r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Doudou"}]}]}"#.into(),
                    status: Some("sent".into()),
                },
            )
            .expect("submitFound");

            let rows = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].status, STATUS_DEFAULT);
            assert!(rows[0].item_description.contains("Doudou"));
        });
}

#[test]
#[serial]
fn host_create_surface_renders_declare_form() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({
                "stayId": stay_id.to_string(),
                "guestName": "Marie Dupont",
                "stayDates": "12–15 juil.",
            });
            let surface = render_host_create(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(SurfaceAssertions::new(&surface).contains_type("RichTextEditor"));
            assert!(SurfaceAssertions::new(&surface).contains_type("FieldHint"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("submitFound") || json.contains("host.create.submit"));
            assert!(json.contains("host.create.description.label") || json.contains("description"));
            // Modal chrome owns title / Annuler — form body has no HeaderTitle.
            assert!(!json.contains("HeaderTitle"));
            assert!(!json.contains("host.main.status.label"));
            assert!(!json.contains("TextArea"));
        });
}

#[test]
#[serial]
fn host_stay_surface_empty_state_when_no_reports() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.empty"));
            assert!(json.contains("host.stay.listTitle"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("List"));
            assert!(!json.contains("submitFound"));
            assert!(!json.contains("host.create.submit"));
            assert!(!json.contains("TextArea"));
            assert!(!json.contains("RichTextEditor"));
        });
}

#[test]
#[serial]
fn host_stay_surface_card_when_reports_exist() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![stay_id],
                    stay_id: None,
                    description: "Chargeur oublié".into(),
                    status: None,
                },
            )
            .expect("submitFound");

            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("List"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Select"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.listTitle"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            assert!(!json.contains("submitFound"));
            assert!(!json.contains("host.create.submit"));
        });
}

#[test]
#[serial]
fn host_main_editor_has_note_not_create_form() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&surface).contains_type("RichTextEditor"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.hostNote.label") || json.contains("host_note"));
            assert!(json.contains("host.main.banner"));
            assert!(!json.contains("host.create.submit"));
            assert!(!json.contains("submitFound"));
        });
}

#[test]
#[serial]
fn host_note_shows_on_guest_card_and_email_context() {
    reset_test_store();
    let config_bytes = serde_json::to_vec(&serde_json::json!({
        "host_note": "Leave found items in the lobby closet."
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[portaki_sdk::capability::core::STORAGE])
        .with_kv("config", config_bytes)
        .run(|ctx| {
            let surface = render_home_card(ctx.clone());
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Leave found items in the lobby closet."));

            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::LostFound),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert_eq!(
                out.checkout_tips.as_deref(),
                Some("Leave found items in the lobby closet.")
            );
        });
}

#[test]
#[serial]
fn update_config_persists_host_note() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[portaki_sdk::capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    host_note: "Lobby closet.".into(),
                },
            )
            .expect("updateConfig");

            let surface = render_host_main(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Lobby closet."));
        });
}

/// The J+2 tick lands inside the platform's guest window (checkout + 7 days), so the
/// after-stay rule lets it through; the same send past the window is refused.
#[test]
#[serial]
fn checkout_follow_up_sends_at_j2_and_stops_after_the_guest_window() {
    for (days_after_checkout, accepted) in [(2, true), (8, false)] {
        reset_test_store();
        let builder = MockContext::guest().with_property(Property::default());
        let stay_id = builder.context().guest.expect("guest").session_id;
        // The invocation stay is the guest's own, so the SDK judges its checkout.
        let booking = Booking {
            id: stay_id,
            ..Booking::default()
        };
        let now = booking.check_out + Duration::days(days_after_checkout);

        builder
            .with_stay(booking)
            .with_now(now)
            .run_with(|ctx, host| {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        kind: "lost".into(),
                        item_description: "Écharpe bleue".into(),
                        contact_hint: None,
                        details: None,
                    },
                )
                .expect("submit");

                let result = send_checkout_follow_up(ctx, EmptyArgs {});
                let follow_ups: Vec<_> = host
                    .sent_emails()
                    .into_iter()
                    .filter(|email| email.email_id == "checkout-j2")
                    .collect();

                if accepted {
                    result.expect("J+2 follow-up");
                    assert_eq!(follow_ups.len(), 1);
                    assert_eq!(follow_ups[0].audience, EmailAudience::Guest);
                    assert_eq!(follow_ups[0].stay_id, Some(stay_id));
                } else {
                    assert!(matches!(
                        result,
                        Err(PortakiError::Email(EmailError::StayEnded))
                    ));
                    assert!(follow_ups.is_empty());
                }
            });
    }
}

/// A 20 000-char guest description: the report keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_description_is_stored_whole_and_quoted_in_the_host_email() {
    reset_test_store();
    let description = format!("{}!", "écharpe ".repeat(2_500).trim_end());
    assert_eq!(description.chars().count(), 20_000);

    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: description.clone(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows[0].item_description, description);

            let email = host.sent_emails().into_iter().last().expect("host email");
            let body = &email.content.body.fr;
            assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
            let quoted = body
                .split("\n\n")
                .find(|part| part.ends_with('…'))
                .expect("quoted description");
            let kept = quoted.trim_end_matches('…');
            assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
            assert!(description.starts_with(kept));
            // Cut on a word boundary: the original goes on with a space.
            assert!(description[kept.len()..].starts_with(' '));

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());

            // Nothing cut: the email keeps its own CTA label.
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "found".into(),
                    item_description: "Parapluie".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("short submit");
            let email = host.sent_emails().into_iter().last().expect("second email");
            assert_eq!(email.content.cta.expect("cta").label.fr, "Voir le logement");
        });
}

/// The report is what the guest submits; the email is a side effect. Past the per-invocation
/// cap the email is refused, and the submit still succeeds with the report saved.
#[test]
#[serial]
fn a_refused_host_email_does_not_fail_the_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            for index in 0..=limits::EMAIL_SENDS_PER_INVOCATION {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        kind: "lost".into(),
                        item_description: format!("Objet {index}"),
                        contact_hint: None,
                        details: None,
                    },
                )
                .expect("submit despite a refused email");
            }
            let rows = list_for_stay(ctx, ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), limits::EMAIL_SENDS_PER_INVOCATION + 1);
            assert_eq!(host.sent_emails().len(), limits::EMAIL_SENDS_PER_INVOCATION);
        });
}

/// Many long declarations: each is quoted, the joined run is bounded, and the J+2 body stays
/// within the platform cap in every locale.
#[test]
#[serial]
fn checkout_follow_up_quotes_long_declarations_within_the_body_cap() {
    reset_test_store();
    let builder = MockContext::guest().with_property(Property::default());
    let stay_id = builder.context().guest.expect("guest").session_id;
    let long = format!("{}!", "valise ".repeat(1_000).trim_end());

    builder.clone().run(|ctx| {
        for _ in 0..12 {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: long.clone(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit");
        }
    });

    let booking = Booking {
        id: stay_id,
        ..Booking::default()
    };
    let now = booking.check_out + Duration::days(2);
    builder
        .with_stay(booking)
        .with_now(now)
        .run_with(|ctx, host| {
            send_checkout_follow_up(ctx, EmptyArgs {}).expect("J+2 follow-up");
            let email = host.sent_emails().into_iter().last().expect("follow-up");
            assert_eq!(email.email_id, "checkout-j2");
            let body = &email.content.body;
            for text in [&body.fr, &body.en]
                .into_iter()
                .chain(body.translations.values())
            {
                assert!(text.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
            }
            assert!(body.fr.contains('…'));
            assert_eq!(email.content.cta.expect("cta").label.fr, "Voir plus");
        });
}
