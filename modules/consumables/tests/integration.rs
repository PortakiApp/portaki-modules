//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;
use uuid::Uuid;

use consumables::{
    list_for_stay, list_items, list_open_count, render_guest_form, render_home_card,
    render_host_main, render_host_stats, render_host_stay, replace_items, reset_test_store,
    seed_defaults, submit, update_config, update_status, ConsumableItemInput, ListForStayArgs,
    ReplaceItemsArgs, SubmitArgs, UpdateConfigArgs, UpdateStatusArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
    LEVEL_DEFAULT, STATUS_DEFAULT,
};
use portaki_sdk::limits;
use portaki_sdk::prelude::EmptyArgs;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};

#[test]
#[serial]
fn home_card_empty_when_no_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.empty"));
        });
}

#[test]
#[serial]
fn home_card_opens_form_overlay_with_catalog() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Café".into(),
                        label_en: "Coffee".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let surface = render_home_card(ctx.clone());
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("guest.form"));
            assert!(json.contains("package"));
            assert!(json.contains("home.card.openForm"));

            let form = render_guest_form(ctx);
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("ChoiceList"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("Café") || form_json.contains("Coffee"));
        });
}

#[test]
#[serial]
fn submit_creates_open_report_and_lists_on_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Papier toilette".into(),
                        label_en: "Toilet paper".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let items = list_items(ctx.clone()).expect("list items");
            assert_eq!(items.len(), 1);
            let item_id = items[0].id;

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_id,
                    level: LEVEL_DEFAULT.into(),
                    note: Some("Salle de bain".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].status, STATUS_DEFAULT);
            assert_eq!(rows[0].level, LEVEL_DEFAULT);
            assert!(rows[0].item_label.contains("Papier") || rows[0].item_label.contains("Toilet"));

            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
        });
}

#[test]
#[serial]
fn host_mark_restocked_clears_open_list() {
    reset_test_store();
    let mut report_id = Uuid::nil();

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Savon".into(),
                        label_en: "Soap".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");
            let item_id = list_items(ctx.clone()).expect("items")[0].id;
            submit(
                ctx.clone(),
                SubmitArgs {
                    item_id,
                    level: "low".into(),
                    note: None,
                },
            )
            .expect("submit");
            report_id = list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0].id;
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let open = list_open_count(ctx.clone()).expect("open count");
            assert_eq!(open.open_count, 1);

            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id,
                    status: "restocked".into(),
                },
            )
            .expect("restock");

            let open = list_open_count(ctx.clone()).expect("open after");
            assert_eq!(open.open_count, 0);

            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("IndexedInput"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.emptyRecent"));
        });
}

#[test]
#[serial]
fn seed_defaults_fills_empty_catalog() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed");
            let items = list_items(ctx.clone()).expect("items");
            assert_eq!(items.len(), 8);

            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed again");
            assert_eq!(list_items(ctx).expect("items").len(), 8);
        });
}

#[test]
#[serial]
fn update_config_replaces_catalog() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    items: vec![ConsumableItemInput {
                        label: "Coffee pods".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                },
            )
            .expect("updateConfig");
            let items = list_items(ctx).expect("items");
            assert_eq!(items.len(), 1);
        });
}

#[test]
#[serial]
fn host_main_and_stats_render() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let main = render_host_main(ctx.clone());
            assert!(SurfaceAssertions::new(&main).contains_type("Page"));
            assert!(SurfaceAssertions::new(&main).contains_type("IndexedInput"));
            assert!(SurfaceAssertions::new(&main).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&main).contains_type("Button"));

            let stats = render_host_stats(ctx);
            assert!(SurfaceAssertions::new(&stats).contains_type("Card"));
            let json = serde_json::to_string(&stats).expect("stats json");
            assert!(json.contains("stats.catalog"));
            assert!(json.contains("stats.open"));
        });
}

#[test]
#[serial]
fn host_stay_empty_state_when_no_reports() {
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
            assert!(json.contains("host.stay.listTitle"));
            assert!(json.contains("host.stay.empty"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("List"));
        });
}

#[test]
#[serial]
fn submit_rejects_unknown_item() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let err = submit(
                ctx,
                SubmitArgs {
                    item_id: Uuid::new_v4(),
                    level: "missing".into(),
                    note: None,
                },
            );
            assert!(err.is_err());
        });
}

/// A 20 000-char guest note: the report keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_note_is_stored_whole_and_quoted_in_the_host_email() {
    reset_test_store();
    let note = format!("{}!", "serviette ".repeat(2_000).trim_end());
    assert_eq!(note.chars().count(), 20_000);

    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Serviettes".into(),
                        label_en: "Towels".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");
            let item_id = list_items(ctx.clone()).expect("items")[0].id;

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_id,
                    level: LEVEL_DEFAULT.into(),
                    note: Some(note.clone()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows[0].note.as_deref(), Some(note.as_str()));

            let email = host.sent_emails().into_iter().last().expect("host email");
            for (body, prefix) in [
                (&email.content.body.fr, "Précision : "),
                (&email.content.body.en, "Note: "),
            ] {
                assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
                let quoted = body
                    .split("\n\n")
                    .find(|part| part.ends_with('…'))
                    .expect("quoted note");
                let kept = quoted
                    .trim_end_matches('…')
                    .strip_prefix(prefix)
                    .expect("note prefix");
                assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
                assert!(note.starts_with(kept));
            }

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());
        });
}
