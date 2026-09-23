//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use issue_report::{
    list_for_stay, list_recent, render_guest_form, render_home_card, render_host_main,
    render_host_stats, reset_test_store, submit, SubmitArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
use portaki_sdk::limits;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};

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
            assert!(json.contains("danger-triangle"));
            assert!(json.contains("guest.form"));
            assert!(json.contains("home.card.openForm"));

            let form = render_guest_form(ctx);
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("form.category.label"));
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
                    category: "appliance".into(),
                    summary: "Oven broken".into(),
                    details: Some("Won't heat".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].category, "appliance");
            assert_eq!(rows[0].summary, "Oven broken");

            submit(
                ctx.clone(),
                SubmitArgs {
                    category: "noise".into(),
                    summary: "Loud neighbors".into(),
                    details: None,
                },
            )
            .expect("submit second");

            let rows = list_for_stay(ctx.clone()).expect("list after second");
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
fn host_main_lists_recent_after_guest_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    category: "cleanliness".into(),
                    summary: "Bathroom not clean".into(),
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

            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("List"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.banner"));
            assert!(json.contains("host.main.status.open"));
            assert!(json.contains("danger-triangle") || json.contains("sparkles"));
        });
}

#[test]
#[serial]
fn host_stats_counts_window_reports_by_category() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            for category in ["appliance", "appliance", "access"] {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        category: category.into(),
                        summary: "x".into(),
                        details: None,
                    },
                )
                .expect("submit");
            }
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_stats(ctx);
            let json = serde_json::to_value(&surface).expect("surface json");
            let text = json.to_string();
            assert!(SurfaceAssertions::new(&surface).contains_type("Stat"));
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            assert!(text.contains(r#""label":"i18n:stats.reports","type":"Stat","value":"3""#));
            let appliance = text.find("form.category.appliance").expect("appliance row");
            let access = text.find("form.category.access").expect("access row");
            assert!(appliance < access, "most reported category first");
            assert!(text.contains("stats.delay.empty"));
        });
}

/// A 20 000-char description: the report keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_details_are_stored_whole_and_quoted_in_the_host_email() {
    reset_test_store();
    let details = format!("{}!", "robinet ".repeat(2_500).trim_end());
    assert_eq!(details.chars().count(), 20_000);

    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    category: "appliance".into(),
                    summary: "Fuite sous l'évier".into(),
                    details: Some(details.clone()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone()).expect("list");
            assert_eq!(rows[0].details.as_deref(), Some(details.as_str()));

            let email = host.sent_emails().into_iter().last().expect("host email");
            let body = &email.content.body.fr;
            assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
            assert!(body.contains("Fuite sous l'évier"));
            let quoted = body
                .split("\n\n")
                .find(|part| part.ends_with('…'))
                .expect("quoted details");
            let kept = quoted.trim_end_matches('…');
            assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
            assert!(details.starts_with(kept));

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());
        });
}
