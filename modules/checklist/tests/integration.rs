//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use chrono::{Duration, Utc};
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_sdk::contracts::timeline::{
    TaskCompleteArgs, TaskToggleArgs, TimelineStay, TimelineTasksArgs,
};
use portaki_sdk::prelude::*;
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

use checklist::{
    complete_item, create_checklist, items_of, list_checklists, list_completions, list_items,
    publish_readiness, render_home_card, render_host_main, render_post_stay_card,
    render_stats_checklist, render_stats_cleaning, reset_test_store, stats_summary, task_complete,
    task_toggle, timeline_tasks, uncomplete_item, update_config, CreateChecklistArgs, ItemIdArgs,
    UpdateConfigArgs,
};
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};

fn create(ctx: &Context, template: &str) {
    create_checklist(
        ctx.clone(),
        CreateChecklistArgs {
            template: template.into(),
        },
    )
    .expect("create");
}

fn json_of(surface: &Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

#[test]
#[serial]
fn home_card_empty_when_no_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            assert!(json_of(&render_home_card(ctx)).contains("home.card.empty"));
        });
}

#[test]
#[serial]
fn departure_template_renders_toggles_and_ticks() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            create(&ctx, "cleaning");
            // Only the guest list reaches the booklet.
            let items = list_items(ctx.clone()).expect("list");
            assert_eq!(items.len(), 5);

            let surface = render_home_card(ctx.clone());
            assert!(SurfaceAssertions::new(&surface).contains_type("ChecklistItem"));
            let json = json_of(&surface);
            assert!(json.contains("Fermer les volets"));
            assert!(json.contains("completeItem"));
            assert!(json_of(&render_post_stay_card(ctx.clone())).contains("completeItem"));

            let item_id = items[0].id;
            complete_item(ctx.clone(), ItemIdArgs { item_id }).expect("complete");
            assert_eq!(list_completions(ctx.clone()).expect("done"), vec![item_id]);
            uncomplete_item(ctx.clone(), ItemIdArgs { item_id }).expect("uncomplete");
            assert!(list_completions(ctx).expect("done").is_empty());
        });
}

#[test]
#[serial]
fn a_guest_cannot_tick_a_host_item() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "cleaning");
            let host_list = list_checklists().expect("lists")[0].id;
            let item_id = items_of(host_list).expect("items")[0].id;
            for result in [
                complete_item(ctx.clone(), ItemIdArgs { item_id }),
                uncomplete_item(ctx.clone(), ItemIdArgs { item_id }),
            ] {
                let err = result.expect_err("host item");
                assert!(err.to_string().contains("not_guest_item"), "{err}");
            }
        });
}

#[test]
#[serial]
fn guest_list_waits_for_its_trigger() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            create(&ctx, "departure");
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() - Duration::days(2)),
                checkout_at: Some(Utc::now() + Duration::days(5)),
                ..StayContext::default()
            });
            let json = json_of(&render_home_card(ctx));
            assert!(json.contains("home.card.notYet"));
            assert!(!json.contains("completeItem"));
        });
}

#[test]
#[serial]
fn legacy_show_when_is_adopted_once() {
    reset_test_store();
    let config = serde_json::to_vec(&json!({ "show_when": "always" })).expect("json");
    MockContext::host().with_kv("config", config).run(|ctx| {
        create(&ctx, "departure");
        let lists = list_checklists().expect("lists");
        assert_eq!(lists[0].trigger, "beforeArrival");
        assert!(portaki_sdk::host::kv::get("config").expect("kv").is_none());
    });
}

#[test]
#[serial]
fn host_editor_lists_checklists_and_offers_templates() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let empty = json_of(&render_host_main(ctx.clone()));
            assert!(empty.contains("createChecklist"));
            assert!(empty.contains("template.cleaning.name"));

            create(&ctx, "cleaning");
            let surface = render_host_main(ctx);
            let assertions = SurfaceAssertions::new(&surface);
            assert!(assertions.contains_type("SelectableCard"));
            assert!(assertions.contains_type("EditableList"));
            assert!(assertions.contains_type("ToggleRow"));
            let json = json_of(&surface);
            assert!(json.contains("Consommables réassortis"));
            assert!(json.contains("host.deadline.nextArrivalMinus2h"));
        });
}

#[test]
#[serial]
fn update_config_saves_the_selected_list() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "cleaning");
        let list = list_checklists().expect("lists").remove(0);
        let kept = items_of(list.id).expect("items")[0].id;
        let items = json!([
            { "id": kept, "label": "Draps", "photo": true },
            { "label": "Sols" },
            { "label": "  " },
        ]);
        update_config(
            ctx.clone(),
            UpdateConfigArgs {
                id: list.id.to_string(),
                name: "Ménage express".into(),
                trigger: "onlyIfNextArrival".into(),
                assignee: "Julie Martin · Ménage".into(),
                deadline: "nextArrival".into(),
                notify_assignee: Some(Value::Bool(false)),
                items: Some(Value::String(items.to_string())),
                ..UpdateConfigArgs::default()
            },
        )
        .expect("updateConfig");

        let list = list_checklists().expect("lists").remove(0);
        assert_eq!(list.name_fr, "Ménage express");
        assert_eq!(list.trigger, "onlyIfNextArrival");
        assert_eq!(list.assignee_name.as_deref(), Some("Julie Martin"));
        assert_eq!(list.assignee_role.as_deref(), Some("Ménage"));
        assert!(!list.notify_assignee);
        let items = items_of(list.id).expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!((items[0].id, items[0].photo_required), (kept, true));
    });
}

fn stays_around() -> Vec<TimelineStay> {
    let now = Utc::now();
    let stay = |guest: &str, days_in: i64, days_out: i64| TimelineStay {
        id: Uuid::new_v4(),
        check_in: now + Duration::days(days_in),
        check_out: now + Duration::days(days_out),
        guest_name: guest.into(),
        status: "UPCOMING".into(),
    };
    vec![stay("Marie", -3, 1), stay("Liam", 3, 6)]
}

fn timeline_args(stays: &[TimelineStay]) -> TimelineTasksArgs {
    TimelineTasksArgs {
        property_id: Uuid::nil(),
        from: Utc::now() - Duration::days(7),
        to: Utc::now() + Duration::days(7),
        stays: stays.to_vec(),
    }
}

#[test]
#[serial]
fn photo_item_refuses_a_tick_without_photo() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "cleaning");
        let stays = stays_around();
        let tasks = timeline_tasks(ctx.clone(), timeline_args(&stays))
            .expect("tasks")
            .tasks;
        // After Marie leaves (before Liam arrives), then after Liam leaves.
        assert_eq!(tasks.len(), 2);
        let task = &tasks[0];
        assert!(task.context.fr.contains("Liam"));
        let photo_item = task.items.iter().find(|i| i.photo_required).expect("photo");
        let toggle = |done, photo: Option<&str>| TaskToggleArgs {
            property_id: Uuid::nil(),
            task_id: task.id.clone(),
            item_id: photo_item.id.clone(),
            done,
            photo: photo.map(str::to_string),
        };

        let refused = task_toggle(ctx.clone(), toggle(true, None)).expect_err("refused");
        assert!(refused.to_string().contains("photo_required"));
        let complete = TaskCompleteArgs {
            property_id: Uuid::nil(),
            task_id: task.id.clone(),
        };
        assert!(task_complete(ctx.clone(), complete.clone()).is_err());
        assert!(task_toggle(ctx.clone(), toggle(true, Some("https://evil"))).is_err());

        let photo = format!("portaki-file:{}", Uuid::new_v4());
        task_toggle(ctx.clone(), toggle(true, Some(&photo))).expect("ticked with photo");
        task_complete(ctx.clone(), complete).expect("complete");

        let tasks = timeline_tasks(ctx.clone(), timeline_args(&stays))
            .expect("tasks")
            .tasks;
        assert!(tasks[0].items.iter().all(|item| item.done));
        let ticked = tasks[0]
            .items
            .iter()
            .find(|i| i.photo_required)
            .expect("photo");
        assert_eq!(ticked.photo.as_deref(), Some(photo.as_str()));

        let summary = stats_summary(
            ctx.clone(),
            StatsSummaryArgs {
                property_id: Uuid::nil(),
                period: 30,
                key: "cleaning".into(),
            },
        )
        .expect("summary");
        assert_eq!(summary.value, "100 %");
        assert!(summary.attention.is_none());
        let json = json_of(&render_stats_cleaning(ctx));
        assert!(json.contains("FeedItem"));
        assert!(json.contains("stats.cleaning.status.done"));
    });
}

#[test]
#[serial]
fn guest_ticks_feed_the_checklist_stats() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let items = list_items(ctx.clone()).expect("items");
            for item in &items[..3] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: item.id }).expect("tick");
            }
            let summary = stats_summary(
                ctx.clone(),
                StatsSummaryArgs {
                    property_id: Uuid::nil(),
                    period: 90,
                    key: "checklist".into(),
                },
            )
            .expect("summary");
            assert_eq!(summary.value, "0 %");
            let json = json_of(&render_stats_checklist(ctx));
            assert!(json.contains("stats.checklist.status.incomplete"));
            assert!(json.contains("stats.checklist.forgotten"));
        });
}

/// The period's stays as the platform passes them to a stats detail: dates and status, no name.
fn with_period_stays(mut ctx: Context, stays: &[TimelineStay]) -> Context {
    ctx.input = json!({
        "periodDays": 30,
        "stays": stays
            .iter()
            .map(|stay| json!({
                "id": stay.id,
                "checkIn": stay.check_in,
                "checkOut": stay.check_out,
                "status": stay.status,
            }))
            .collect::<Vec<Value>>(),
    });
    ctx
}

#[test]
#[serial]
fn a_cleaning_is_judged_against_the_next_arrival() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "cleaning");
        let now = Utc::now();
        let stay = |days_in: i64, days_out: i64| TimelineStay {
            id: Uuid::new_v4(),
            check_in: now + Duration::days(days_in),
            check_out: now + Duration::days(days_out),
            guest_name: String::new(),
            status: "COMPLETED".into(),
        };
        let stays = vec![stay(-14, -10), stay(-8, -5), stay(2, 5)];

        // The cleaning after the first departure is ticked now, days after the next arrival.
        let tasks = timeline_tasks(
            ctx.clone(),
            TimelineTasksArgs {
                property_id: Uuid::nil(),
                from: now - Duration::days(30),
                to: now,
                stays: stays.clone(),
            },
        )
        .expect("tasks")
        .tasks;
        let late = &tasks[0];
        let photo = format!("portaki-file:{}", Uuid::new_v4());
        for item in &late.items {
            task_toggle(
                ctx.clone(),
                TaskToggleArgs {
                    property_id: Uuid::nil(),
                    task_id: late.id.clone(),
                    item_id: item.id.clone(),
                    done: true,
                    photo: item.photo_required.then(|| photo.clone()),
                },
            )
            .expect("tick");
        }

        let json = json_of(&render_stats_cleaning(with_period_stays(ctx, &stays)));
        // Late for the first, still open for the second (its deadline is ahead); on time: none.
        assert!(json.contains("stats.cleaning.status.late"));
        assert!(json.contains("stats.cleaning.status.open"));
        assert!(!json.contains("stats.cleaning.status.done"));
        assert!(json.contains("stats.cleaning.onTime"));
        assert!(json.contains("\"0 %\""));
    });
}

#[test]
#[serial]
fn a_stay_that_never_opened_its_list_shows_unfilled() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "departure");
        let now = Utc::now();
        let departed = TimelineStay {
            id: Uuid::new_v4(),
            check_in: now - Duration::days(6),
            check_out: now - Duration::days(2),
            guest_name: String::new(),
            status: "COMPLETED".into(),
        };
        let json = json_of(&render_stats_checklist(with_period_stays(ctx, &[departed])));
        assert!(json.contains("stats.checklist.status.unfilled"));
        assert!(json.contains("\"0 %\""));
    });
}

#[test]
#[serial]
fn publish_readiness_requires_an_item() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let ok = |ctx: &Context| publish_readiness(ctx.clone()).expect("readiness").items[0].ok;
            assert!(!ok(&ctx));
            create(&ctx, "emptyGuest");
            assert!(!ok(&ctx), "an empty list shows nothing");
            create(&ctx, "cleaning");
            assert!(ok(&ctx));
        });
}
