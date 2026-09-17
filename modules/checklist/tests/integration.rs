//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use serial_test::serial;

use checklist::{
    complete_item, list_completions, list_items, render_home_card, render_host_main, replace_items,
    reset_test_store, uncomplete_item, update_config, ChecklistItemInput, ItemIdArgs,
    ReplaceItemsArgs, UpdateConfigArgs,
};
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
fn home_card_renders_toggles_with_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![
                        ChecklistItemInput {
                            label: String::new(),
                            label_fr: "Fermer les volets".into(),
                            label_en: "Close shutters".into(),
                            sort_order: 0,
                        },
                        ChecklistItemInput {
                            label: String::new(),
                            label_fr: "Sortir les poubelles".into(),
                            label_en: "Take out bins".into(),
                            sort_order: 1,
                        },
                    ],
                    items_json: None,
                },
            )
            .expect("replace");

            let surface = render_home_card(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ChecklistItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Fermer les volets") || json.contains("Close shutters"));
            assert!(json.contains("completeItem"));
        });
}

#[test]
#[serial]
fn complete_and_uncomplete_roundtrip() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ChecklistItemInput {
                        label: String::new(),
                        label_fr: "Clés".into(),
                        label_en: "Keys".into(),
                        sort_order: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let items = list_items(ctx.clone()).expect("list");
            assert_eq!(items.len(), 1);
            let item_id = items[0].id;

            complete_item(ctx.clone(), ItemIdArgs { item_id }).expect("complete");
            let done = list_completions(ctx.clone()).expect("completions");
            assert_eq!(done, vec![item_id]);

            uncomplete_item(ctx.clone(), ItemIdArgs { item_id }).expect("uncomplete");
            let done = list_completions(ctx).expect("completions after uncomplete");
            assert!(done.is_empty());
        });
}

#[test]
#[serial]
fn host_main_renders_form() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            assert!(SurfaceAssertions::new(&surface).contains_type("IndexedInput"));
            // Workspace header Save owns persistence — no in-form Enregistrer.
            assert!(!SurfaceAssertions::new(&surface).contains_type("Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.item.empty"));
            // Wasm emits all slots; host binding keeps one trailing empty while typing.
            assert!(json.contains("items.0.label"));
            assert!(json.contains("items.5.label"));
            assert!(!json.contains("items.6.label"));
        });
}

#[test]
#[serial]
fn host_main_emits_all_item_slots() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ChecklistItemInput {
                        label: "Fermer les volets".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let json = serde_json::to_string(&render_host_main(ctx)).expect("surface json");
            assert!(json.contains("items.0.label"));
            assert!(json.contains("items.1.label"));
            assert!(json.contains("items.5.label"));
            assert!(!json.contains("items.6.label"));
        });
}

#[test]
#[serial]
fn update_config_replaces_items_from_form() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        update_config(
            ctx.clone(),
            UpdateConfigArgs {
                show_when: "from_checkin".into(),
                items: vec![
                    ChecklistItemInput {
                        label: "Clés".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                    },
                    ChecklistItemInput {
                        label: String::new(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 1,
                    },
                ],
            },
        )
        .expect("updateConfig");

        let items = list_items(ctx).expect("list");
        assert_eq!(items.len(), 1);
    });
}

#[test]
#[serial]
fn host_main_renders_when_choice_list() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx)).expect("surface json");
            assert!(json.contains("show_when"));
            assert!(json.contains("from_checkin"));
            assert!(json.contains("before_checkout"));
            assert!(json.contains("checkout_day"));
            assert!(json.contains("host.section.when"));
        });
}

#[test]
#[serial]
fn home_card_gated_shows_not_yet() {
    use chrono::{Duration, Utc};
    use portaki_sdk::context::StayContext;
    use serde_json::json;

    reset_test_store();
    let config_bytes = serde_json::to_vec(&json!({
        "show_when": "checkout_day"
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_kv("config", config_bytes)
        .run(|mut ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ChecklistItemInput {
                        label: String::new(),
                        label_fr: "Clés".into(),
                        label_en: "Keys".into(),
                        sort_order: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() - Duration::days(2)),
                checkout_at: Some(Utc::now() + Duration::days(3)),
                booking_channel: None,
                ..StayContext::default()
            });

            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.notYet"));
            assert!(!json.contains("completeItem"));
        });
}
