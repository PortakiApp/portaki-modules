//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use chrono::{Duration, Utc};
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::publish::PublishLevel;
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
    publish_readiness, render_home_card, render_host_main, render_host_stay, render_post_stay_card,
    render_stats_checklist, render_stats_cleaning, reset_test_store, send_departure_reminder,
    set_completed, stats_summary, task_complete, task_toggle, timeline_tasks, uncomplete_item,
    update_config, CreateChecklistArgs, ItemIdArgs, SetCompletedArgs, UpdateConfigArgs,
};
use portaki_test_utils::{Booking, MockContext, Property, SurfaceAssertions};

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
            assert!(json_of(&render_home_card(ctx).expect("render")).contains("home.card.empty"));
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

            // Une seule liste à cocher, et c'est le livret qui la dessine (§2.9).
            let surface = render_home_card(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("ChoiceList"));
            let json = json_of(&surface);
            assert!(json.contains("Fermer les volets"), "{json}");
            assert!(json.contains("\"layout\":\"checklist\""), "{json}");
            assert!(json.contains("setCompleted"), "{json}");
            assert!(
                json_of(&render_post_stay_card(ctx.clone()).expect("render"))
                    .contains("setCompleted")
            );

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
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("home.card.notYet"));
            assert!(!json.contains("completeItem"));
        });
}

#[test]
#[serial]
fn legacy_show_when_is_adopted_once() {
    reset_test_store();
    let config = serde_json::to_vec(&json!({ "show_when": "checkout_day" })).expect("json");
    MockContext::host().with_kv("config", config).run(|ctx| {
        create(&ctx, "departure");
        let lists = list_checklists().expect("lists");
        assert_eq!(lists[0].trigger, "departureDay");
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

/// Un code écrit dans une étape du livret avertit ; une étape trop longue bloque (§3, §2.2).
#[test]
#[serial]
fn publish_readiness_warns_on_a_code_and_blocks_a_long_step() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "emptyGuest");
            let list = list_checklists().expect("lists").remove(0);
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    id: list.id.to_string(),
                    items: Some(json!([
                        { "label": "Laisser la clé dans la boîte, code 4821" },
                        { "label": "x".repeat(81) }
                    ])),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("save");
            let checks = publish_readiness(ctx.clone()).expect("readiness").items;
            let level_of = |prefix: &str| {
                checks
                    .iter()
                    .find(|check| check.id.starts_with(prefix))
                    .map(|check| (check.ok, check.level))
            };
            assert_eq!(level_of("code."), Some((false, PublishLevel::Recommended)));
            assert_eq!(level_of("labels."), Some((false, PublishLevel::Required)));
            assert_eq!(level_of("steps."), None);
        });
}

/// Les rubriques du modèle de départ arrivent au voyageur, et c'est le livret qui les rend : le
/// module ne pose pas de titres, il dit sous quelle rubrique chaque étape se range (§2.9).
#[test]
#[serial]
fn the_departure_template_carries_its_groups() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let json = json_of(&render_home_card(ctx).expect("render"));

            assert!(json.contains("Dans le logement"), "{json}");
            assert!(json.contains("En partant"), "{json}");
            // Deux rubriques pour cinq étapes : un groupe par étape n'aurait rien regroupé.
            assert_eq!(json.matches("Dans le logement").count(), 3, "{json}");
            assert_eq!(json.matches("En partant").count(), 2, "{json}");
        });
}

/// L'état coché part du serveur : une coche doit survivre à la fermeture du livret.
#[test]
#[serial]
fn the_card_hands_the_booklet_what_is_already_ticked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let items = list_items(ctx.clone()).expect("list");
            let first = items[0].id;
            complete_item(ctx.clone(), ItemIdArgs { item_id: first }).expect("complete");

            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains(&format!("\"value\":\"{first}\"")), "{json}");
            assert!(json.contains("\"emitOnChange\":true"), "{json}");
            assert!(json.contains("\"limit\":5"), "{json}");
            assert!(json.contains("\"multi\":true"), "{json}");
        });
}

/// `setCompleted` remplace l'ensemble : il coche ce qui manque et décoche ce qui n'y est plus.
#[test]
#[serial]
fn set_completed_replaces_the_whole_set() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let items = list_items(ctx.clone()).expect("list");
            let (a, b, c) = (items[0].id, items[1].id, items[2].id);

            set_completed(
                ctx.clone(),
                SetCompletedArgs {
                    item_ids: format!("{a},{b}"),
                },
            )
            .expect("set");
            let mut done = list_completions(ctx.clone()).expect("done");
            done.sort();
            let mut expected = vec![a, b];
            expected.sort();
            assert_eq!(done, expected);

            // b disparaît, c arrive : l'ensemble envoyé fait loi.
            set_completed(
                ctx.clone(),
                SetCompletedArgs {
                    item_ids: format!("{a}, {c}"),
                },
            )
            .expect("set");
            let mut done = list_completions(ctx.clone()).expect("done");
            done.sort();
            let mut expected = vec![a, c];
            expected.sort();
            assert_eq!(done, expected);

            // Rejouer le même appel ne double rien.
            set_completed(
                ctx.clone(),
                SetCompletedArgs {
                    item_ids: format!("{a},{c}"),
                },
            )
            .expect("set");
            assert_eq!(list_completions(ctx.clone()).expect("done").len(), 2);

            // Vide : tout se décoche.
            set_completed(
                ctx.clone(),
                SetCompletedArgs {
                    item_ids: String::new(),
                },
            )
            .expect("set");
            assert!(list_completions(ctx).expect("done").is_empty());
        });
}

/// Un voyageur ne coche que les listes écrites pour lui.
#[test]
#[serial]
fn set_completed_refuses_a_host_item() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "cleaning");
            let host_list = list_checklists().expect("lists")[0].id;
            let host_item = items_of(host_list).expect("items")[0].id;

            let refused = set_completed(
                ctx,
                SetCompletedArgs {
                    item_ids: host_item.to_string(),
                },
            );
            assert!(refused.is_err(), "une étape de l'hôte n'est pas cochable");
        });
}

/// Le groupe et la précision que l'hôte écrit arrivent jusqu'au voyageur, et la langue qu'il
/// n'édite pas garde ce qu'elle contenait.
#[test]
#[serial]
fn the_host_group_and_line_reach_the_guest() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "emptyGuest");
            let list = list_checklists().expect("lists")[0].id;
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    id: list.to_string(),
                    items: Some(json!([
                        {
                            "label": "Vider le réfrigérateur",
                            "group": "Cuisine",
                            "description": "Laissez la porte entrouverte"
                        }
                    ])),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("save");

            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("Cuisine"), "{json}");
            assert!(json.contains("Laissez la porte entrouverte"), "{json}");
        });
}

fn save_display(ctx: &Context, list: Uuid, visible_limit: Value, done_message: &str) {
    update_config(
        ctx.clone(),
        UpdateConfigArgs {
            id: list.to_string(),
            trigger: "atDeparture".into(),
            visible_limit: Some(visible_limit),
            done_message: Some(done_message.into()),
            items: Some(json!(items_of(list)
                .expect("items")
                .iter()
                .map(|item| json!({ "id": item.id, "label": item.label_fr }))
                .collect::<Vec<_>>())),
            ..UpdateConfigArgs::default()
        },
    )
    .expect("save");
}

fn readiness_hint(ctx: &Context, prefix: &str) -> Option<String> {
    publish_readiness(ctx.clone())
        .expect("readiness")
        .items
        .into_iter()
        .find(|check| check.id.starts_with(prefix))
        .map(|check| check.hint.get("fr").to_string())
}

/// « Étapes visibles » folds the card, « Message final » replaces the thanks (spec §2.1).
#[test]
#[serial]
fn the_display_settings_reach_the_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let json = json_of(&render_home_card(ctx.clone()).expect("render"));
            assert!(json.contains("\"limit\":5"), "{json}");

            let list = list_checklists().expect("lists")[0].id;
            save_display(&ctx, list, json!("7"), "Claire vous remercie.");
            let json = json_of(&render_home_card(ctx.clone()).expect("render"));
            assert!(json.contains("\"limit\":7"), "{json}");
            assert!(json.contains("Claire vous remercie."), "{json}");
            assert_eq!(readiness_hint(&ctx, "config."), None);
            assert_eq!(items_of(list).expect("items").len(), 5);
        });
}

/// Out of bounds: « Entre 3 et 10. » blocks publication and sits under the field; the card
/// keeps a limit it can draw.
#[test]
#[serial]
fn display_settings_out_of_bounds_block_publication() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let list = list_checklists().expect("lists")[0].id;
            save_display(&ctx, list, json!(12), &"x".repeat(121));
            assert_eq!(
                readiness_hint(&ctx, "config.visible_limit.").as_deref(),
                Some("Entre 3 et 10.")
            );
            assert_eq!(
                readiness_hint(&ctx, "config.done_message.").as_deref(),
                Some("120 caractères au maximum.")
            );
            let editor = json_of(&render_host_main(ctx.clone()));
            assert!(editor.contains("Entre 3 et 10."), "{editor}");
        });
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let list = list_checklists().expect("lists")[0].id;
            save_display(&ctx, list, json!(12), "");
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("\"limit\":10"), "{json}");
            assert!(json.contains("guest.done.message"), "{json}");
        });
}

/// A blank step is kept for « Écrivez l'étape. » but reaches no one; group ≤ 30, detail ≤ 120.
#[test]
#[serial]
fn each_step_is_checked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "emptyGuest");
            let list = list_checklists().expect("lists")[0].id;
            let save = |items: Value| {
                update_config(
                    ctx.clone(),
                    UpdateConfigArgs {
                        id: list.to_string(),
                        items: Some(items),
                        ..UpdateConfigArgs::default()
                    },
                )
                .expect("save");
            };
            save(json!([{ "label": "Sortir les poubelles" }, { "label": "  " }]));
            assert_eq!(
                readiness_hint(&ctx, "labels.").as_deref(),
                Some("Écrivez l'étape.")
            );
            assert_eq!(list_items(ctx.clone()).expect("items").len(), 1);
            let json = json_of(&render_home_card(ctx.clone()).expect("render"));
            assert_eq!(json.matches("\"label\":").count(), 1, "{json}");

            save(json!([{ "label": "Poubelles", "group": "x".repeat(31) }]));
            assert_eq!(
                readiness_hint(&ctx, "labels.").as_deref(),
                Some("30 caractères au maximum.")
            );
            save(json!([{ "label": "Poubelles", "group": "", "description": "x".repeat(121) }]));
            assert_eq!(
                readiness_hint(&ctx, "labels.").as_deref(),
                Some("120 caractères au maximum.")
            );
            save(json!([{ "label": "Poubelles", "description": "Bac jaune" }]));
            assert_eq!(readiness_hint(&ctx, "labels."), None);
        });
}

/// « 3 / 5 étapes · non terminée », then « Terminée le … » once every step is ticked (§1).
#[test]
#[serial]
fn the_stay_detail_tells_the_stay_progress() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            create(&ctx, "cleaning");
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            let mut host = ctx.clone();
            host.input = json!({ "stayId": stay_id.to_string() });
            let stay = |host: &Context| json_of(&render_host_stay(host.clone()).expect("render"));

            let items = list_items(ctx.clone()).expect("items");
            for item in &items[..3] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: item.id }).expect("tick");
            }
            assert!(stay(&host).contains("3 / 5"), "{}", stay(&host));
            for item in &items[3..] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: item.id }).expect("tick");
            }
            assert!(stay(&host).contains("Terminée le"), "{}", stay(&host));

            host.input = json!({});
            assert!(stay(&host).contains("host.stay.missingStay"));
        });
}

/// The editor's ✕ drops the row from the form value: a removed step is gone, and blocks nothing.
#[test]
#[serial]
fn a_removed_step_does_not_block_publication() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "emptyGuest");
            let list = list_checklists().expect("lists")[0].id;
            let save = |items: Value| {
                update_config(
                    ctx.clone(),
                    UpdateConfigArgs {
                        id: list.to_string(),
                        items: Some(items),
                        ..UpdateConfigArgs::default()
                    },
                )
                .expect("save");
            };
            save(json!([{ "label": "Poubelles" }, { "label": "  " }]));
            let kept = items_of(list).expect("items")[0].id;
            save(json!([{ "id": kept, "label": "Poubelles" }]));
            assert_eq!(readiness_hint(&ctx, "labels."), None);
            let checks = publish_readiness(ctx.clone()).expect("readiness").items;
            assert!(checks.iter().all(|check| check.ok), "{checks:?}");
            assert_eq!(items_of(list).expect("items").len(), 1);
        });
}

/// L'éditeur rend la rubrique et la précision d'une étape dans la langue éditée, et chaque langue
/// garde la sienne à l'enregistrement.
#[test]
#[serial]
fn step_group_and_detail_render_and_persist_per_language() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "emptyGuest");
        create(&ctx, "emptyHost");
        let lists = list_checklists().expect("lists");
        let (guest, host) = (lists[0].id, lists[1].id);
        let save = |locale: &str, id: Option<Uuid>, group: &str, description: &str| {
            let mut ctx = ctx.clone();
            ctx.locale = locale.into();
            let mut row =
                json!({ "label": "Vider le frigo", "group": group, "description": description });
            if let Some(id) = id {
                row["id"] = json!(id);
            }
            update_config(
                ctx,
                UpdateConfigArgs {
                    id: guest.to_string(),
                    items: Some(json!([row])),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("save");
            items_of(guest).expect("items")[0].clone()
        };
        let item = save("fr-FR", None, "Cuisine", "Porte entrouverte");
        let item = save("en-US", Some(item.id), "Kitchen", "Leave the door ajar");
        assert!(item.group_i18n.contains("Cuisine") && item.group_i18n.contains("Kitchen"));
        assert!(item.description_i18n.contains("Porte entrouverte"));
        assert!(item.description_i18n.contains("Leave the door ajar"));

        let editor = |locale: &str, list: Uuid| {
            let mut ctx = ctx.clone();
            ctx.locale = locale.into();
            ctx.input = json!({ "selectedId": list.to_string() });
            json_of(&render_host_main(ctx))
        };
        let fr = editor("fr-FR", guest);
        assert!(fr.contains(r#""groupField":true"#), "{fr}");
        assert!(
            fr.contains(r#""group":"Cuisine""#) && !fr.contains("Kitchen"),
            "{fr}"
        );
        assert!(fr.contains(r#""description":"Porte entrouverte""#), "{fr}");
        let en = editor("en-US", guest);
        assert!(
            en.contains(r#""group":"Kitchen""#) && !en.contains("Cuisine"),
            "{en}"
        );
        // Une tâche d'équipe ne lit ni rubrique ni précision : pas de champ mort.
        let team = editor("fr-FR", host);
        assert!(!team.contains(r#""groupField":true"#), "{team}");
    });
}

/// « Rappel le matin du départ » : un e-mail le jour du départ si la liste n'est pas terminée,
/// rien quand elle l'est ou que l'hôte a coupé le rappel.
#[test]
#[serial]
fn the_departure_reminder_goes_only_to_an_unfinished_list() {
    for case in ["unfinished", "finished", "off", "gone"] {
        reset_test_store();
        let builder = MockContext::guest().with_property(Property::default());
        let stay_id = builder.context().guest.expect("guest").session_id;
        let booking = Booking {
            id: stay_id,
            ..Booking::default()
        };
        let now = match case {
            "gone" => booking.check_out + Duration::hours(1),
            _ => booking.check_out - Duration::hours(2),
        };
        builder
            .with_stay(booking)
            .with_now(now)
            .run_with(|ctx, host| {
                create(&ctx, "departure");
                let list = list_checklists().expect("lists")[0].id;
                if case == "finished" {
                    let ids: Vec<String> = items_of(list)
                        .expect("items")
                        .iter()
                        .map(|item| item.id.to_string())
                        .collect();
                    set_completed(
                        ctx.clone(),
                        SetCompletedArgs {
                            item_ids: ids.join(","),
                        },
                    )
                    .expect("tick all");
                }
                if case == "off" {
                    update_config(
                        ctx.clone(),
                        UpdateConfigArgs {
                            id: list.to_string(),
                            trigger: "atDeparture".into(),
                            remind: Some(json!(false)),
                            items: Some(json!(items_of(list)
                                .expect("items")
                                .iter()
                                .map(|item| json!({ "id": item.id, "label": item.label_fr }))
                                .collect::<Vec<_>>())),
                            ..UpdateConfigArgs::default()
                        },
                    )
                    .expect("save");
                }
                send_departure_reminder(ctx, EmptyArgs {}).expect("reminder");
                let sent = host
                    .sent_emails()
                    .into_iter()
                    .filter(|email| email.email_id == "departure-reminder")
                    .count();
                assert_eq!(sent, usize::from(case == "unfinished"), "{case}");
            });
    }
}
