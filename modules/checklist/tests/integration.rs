//! Ce que les checklists font de plus que la collection du SDK : la carte du livret, les coches,
//! les tâches d'équipe, les statistiques, le rappel, la fiche du séjour, la publication. La fusion
//! du formulaire (par id, langue de l'hôte, ligne vide retirée) est celle du SDK, testée là-bas.

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
    __portaki_lists_update_config, all_lists, booklet_lists, complete_item, create_checklist,
    list_completions, publish_readiness, render_home_card, render_host_main, render_host_stay,
    render_post_stay_card, render_stats_checklist, render_stats_cleaning, reset_test_store,
    send_departure_reminder, set_completed, stats_summary, task_complete, task_toggle,
    timeline_tasks, uncomplete_item, CreateChecklistArgs, ItemIdArgs, List, SetCompletedArgs,
};
use portaki_test_utils::{Booking, MockContext, Property, SurfaceAssertions};

fn create(ctx: &Context, template: &str) {
    create_checklist(
        ctx.clone(),
        CreateChecklistArgs {
            template: template.into(),
            id: None,
        },
    )
    .expect("create");
}

fn json_of(surface: &Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

/// Les étapes du livret, liste par liste.
fn guest_ids() -> Vec<Uuid> {
    booklet_lists()
        .expect("lists")
        .iter()
        .flat_map(|list| list.written_steps())
        .map(|step| step.uuid())
        .collect()
}

fn list(n: usize) -> List {
    all_lists().expect("lists").remove(n)
}

fn step_ids(list: &List) -> Vec<Uuid> {
    list.written_steps()
        .iter()
        .map(|step| step.uuid())
        .collect()
}

/// L'hôte modifie ses listes : ce que le formulaire enverrait, toutes les lignes avec leur id.
fn edit(ctx: &Context, change: impl FnOnce(&mut Value)) {
    let mut lists = serde_json::to_value(all_lists().expect("lists")).expect("json");
    change(&mut lists);
    __portaki_lists_update_config(ctx.clone(), json!({ "lists": lists })).expect("updateConfig");
}

fn readiness_hint(ctx: &Context, needle: &str) -> Option<String> {
    publish_readiness(ctx.clone())
        .expect("readiness")
        .items
        .into_iter()
        .find(|check| check.id.contains(needle))
        .map(|check| check.hint.get("fr").to_string())
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

/// `setCompleted` remplace l'ensemble : il coche ce qui manque et décoche ce qui n'y est plus.
#[test]
#[serial]
fn set_completed_replaces_the_whole_set() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let items = guest_ids();
            let (a, b, c) = (items[0], items[1], items[2]);

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
            // Rien à dire tant que la liste n'ouvre pas : un état vide sans texte, que le livret
            // masque — pas de carte « pas encore ».
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains(r#""type":"EmptyState""#), "{json}");
            assert!(!json.contains("home.card.notYet"), "{json}");
            assert!(!json.contains("completeItem"));
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
            let items = guest_ids();
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

            let item_id = items[0];
            complete_item(ctx.clone(), ItemIdArgs { item_id }).expect("complete");
            assert_eq!(list_completions(ctx.clone()).expect("done"), vec![item_id]);
            uncomplete_item(ctx.clone(), ItemIdArgs { item_id }).expect("uncomplete");
            assert!(list_completions(ctx).expect("done").is_empty());
        });
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
                views: None,
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
            let items = guest_ids();
            for item in &items[..3] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: *item }).expect("tick");
            }
            let summary = stats_summary(
                ctx.clone(),
                StatsSummaryArgs {
                    property_id: Uuid::nil(),
                    period: 90,
                    key: "checklist".into(),
                    views: None,
                },
            )
            .expect("summary");
            assert_eq!(summary.value, "0 %");
            let json = json_of(&render_stats_checklist(ctx));
            assert!(json.contains("stats.checklist.status.incomplete"));
            assert!(json.contains("stats.checklist.forgotten"));
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
            let items = guest_ids();
            let first = items[0];
            complete_item(ctx.clone(), ItemIdArgs { item_id: first }).expect("complete");

            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains(&format!("\"value\":\"{first}\"")), "{json}");
            assert!(json.contains("\"emitOnChange\":true"), "{json}");
            assert!(json.contains("\"limit\":5"), "{json}");
            assert!(json.contains("\"multi\":true"), "{json}");
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

            let items = guest_ids();
            for item in &items[..3] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: *item }).expect("tick");
            }
            assert!(stay(&host).contains("3 / 5"), "{}", stay(&host));
            for item in &items[3..] {
                complete_item(ctx.clone(), ItemIdArgs { item_id: *item }).expect("tick");
            }
            assert!(stay(&host).contains("Terminée le"), "{}", stay(&host));
            // Une liste d'équipe a sa ligne dès que sa tâche sur ce séjour a commencé.
            assert!(!stay(&host).contains("Ménage"));
            let cleaning = list(1);
            let mut team = host.clone();
            team.guest = None;
            task_toggle(
                team,
                TaskToggleArgs {
                    property_id: Uuid::nil(),
                    task_id: checklist::task_id(cleaning.uuid(), stay_id),
                    item_id: step_ids(&cleaning)[0].to_string(),
                    done: true,
                    photo: None,
                },
            )
            .expect("toggle");
            assert!(stay(&host).contains("Ménage · 1 / 7"), "{}", stay(&host));

            host.input = json!({});
            assert!(stay(&host).contains("host.stay.missingStay"));
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
            // The eve at 08:00 UTC, when the platform runs `offset_days = -1`.
            _ => booking.check_out - Duration::hours(26),
        };
        builder
            .with_stay(booking)
            .with_now(now)
            .run_with(|ctx, host| {
                create(&ctx, "departure");
                if case == "finished" {
                    let ids: Vec<String> = step_ids(&list(0)).iter().map(Uuid::to_string).collect();
                    set_completed(
                        ctx.clone(),
                        SetCompletedArgs {
                            item_ids: ids.join(","),
                        },
                    )
                    .expect("tick all");
                }
                if case == "off" {
                    edit(&ctx, |lists| {
                        lists[0]["guest_trigger"] = json!("atDeparture");
                        lists[0]["remind"] = json!(false);
                    });
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

/// Une seule checklist « Voyageur · Départ » dans le livret : la première ; l'éditeur le dit.
#[test]
#[serial]
fn only_the_first_departure_list_reaches_the_booklet() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        create(&ctx, "departure");
        create(&ctx, "departure");
        create(&ctx, "arrival");
        assert_eq!(guest_ids().len(), 5 + 3);
        let editor = json_of(&render_host_main(ctx).expect("render"));
        assert_eq!(
            editor.matches("host.lists.meta.live").count(),
            2,
            "{editor}"
        );
        assert_eq!(
            editor.matches("host.lists.meta.steps").count(),
            1,
            "{editor}"
        );
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
            let item_id = step_ids(&list(0))[0];
            for result in [
                complete_item(ctx.clone(), ItemIdArgs { item_id }),
                uncomplete_item(ctx.clone(), ItemIdArgs { item_id }),
            ] {
                let err = result.expect_err("host item");
                assert!(err.to_string().contains("not_guest_item"), "{err}");
            }
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
            let host_item = step_ids(&list(0))[0];

            let refused = set_completed(
                ctx,
                SetCompletedArgs {
                    item_ids: host_item.to_string(),
                },
            );
            assert!(refused.is_err(), "une étape de l'hôte n'est pas cochable");
        });
}

/// Les étapes sont les clés des coches : retirer une étape ne fait pas glisser la coche d'une
/// autre sur elle, et la coche de l'étape gardée la désigne toujours.
#[test]
#[serial]
fn a_removed_step_does_not_move_the_ticks_of_the_others() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            let ids = guest_ids();
            complete_item(ctx.clone(), ItemIdArgs { item_id: ids[1] }).expect("tick");
            edit(&ctx, |lists| {
                let steps = lists[0]["steps"].as_array_mut().unwrap();
                steps.remove(0);
            });
            let after = guest_ids();
            assert_eq!(after.len(), 4);
            assert_eq!(after[0], ids[1], "the second step keeps its id");
            assert_eq!(list_completions(ctx.clone()).expect("done"), vec![ids[1]]);
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(
                json.contains(&format!("\"value\":\"{}\"", ids[1])),
                "{json}"
            );
        });
}

#[test]
#[serial]
fn the_editor_is_a_table_of_lists_and_the_templates() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        let empty = json_of(&render_host_main(ctx.clone()).expect("render"));
        assert!(empty.contains("createChecklist") && empty.contains("template.cleaning.name"));

        create(&ctx, "cleaning");
        let json = json_of(&render_host_main(ctx.clone()).expect("render"));
        assert!(json.contains(r#""rowName":"lists""#), "{json}");
        assert!(json.contains(r#""name":"lists.0.steps.6.photo""#), "{json}");
        assert!(
            json.contains(r#""visibleWhen":"lists.0.audience=host""#),
            "{json}"
        );
        assert!(json.contains("host.deadline.nextArrivalMinus2h"), "{json}");
        assert!(json.contains("Consommables réassortis"), "{json}");
    });
}

#[test]
#[serial]
fn publication_waits_for_a_written_step_and_warns_on_a_code() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        assert!(readiness_hint(&ctx, "steps").is_some());
        create(&ctx, "emptyGuest");
        assert!(
            readiness_hint(&ctx, "steps").is_some(),
            "an empty list shows nothing"
        );
        edit(&ctx, |lists| {
            lists[0]["steps"] = json!([
                { "label": { "fr": "Laisser la clé dans la boîte, code 4821" } },
                { "label": { "fr": "x".repeat(81) } }
            ]);
        });
        let checks = publish_readiness(ctx.clone()).expect("readiness").items;
        let code = checks.iter().find(|c| c.id.starts_with("code."));
        assert_eq!(code.map(|c| c.level), Some(PublishLevel::Recommended));
        assert_eq!(
            readiness_hint(&ctx, ".steps").as_deref(),
            Some("80 caractères au maximum.")
        );
        assert!(readiness_hint(&ctx, "resources.lists.steps").is_none());
    });
}

/// « Étapes visibles » plie la carte, « Message final » remplace le remerciement (§2.1) ; hors
/// bornes, la publication l'attend et la carte garde ce qu'elle sait dessiner.
#[test]
#[serial]
fn the_display_settings_reach_the_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "departure");
            assert!(
                json_of(&render_home_card(ctx.clone()).expect("render")).contains("\"limit\":5")
            );
            edit(&ctx, |lists| {
                lists[0]["guest_trigger"] = json!("atDeparture");
                lists[0]["visible_limit"] = json!(7);
                lists[0]["done_message"] = json!({ "fr": "Claire vous remercie." });
            });
            let json = json_of(&render_home_card(ctx.clone()).expect("render"));
            assert!(
                json.contains("\"limit\":7") && json.contains("Claire vous remercie."),
                "{json}"
            );

            edit(&ctx, |lists| lists[0]["visible_limit"] = json!(12));
            assert_eq!(
                readiness_hint(&ctx, "visible_limit").as_deref(),
                Some("Entre 3 et 10.")
            );
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("\"limit\":10"), "{json}");
        });
}

/// La rubrique et la précision qu'écrit l'hôte arrivent au voyageur.
#[test]
#[serial]
fn the_host_group_and_line_reach_the_guest() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            create(&ctx, "emptyGuest");
            edit(&ctx, |lists| {
                lists[0]["steps"] = json!([{ "label": { "fr": "Vider le réfrigérateur" },
                    "zone": { "fr": "Cuisine" }, "detail": { "fr": "Laissez la porte entrouverte" } }]);
            });
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("Cuisine") && json.contains("Laissez la porte entrouverte"), "{json}");
        });
}

/// « Avant l'arrivée » n'est pas dans la maquette, mais une liste qui l'a s'ouvre avant l'arrivée.
#[test]
#[serial]
fn a_before_arrival_list_keeps_opening_before_arrival() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            create(&ctx, "departure");
            edit(&ctx, |lists| {
                lists[0]["guest_trigger"] = json!("beforeArrival")
            });
            assert_eq!(list(0).trigger(), "beforeArrival");
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(5)),
                checkout_at: Some(Utc::now() + Duration::days(9)),
                ..StayContext::default()
            });
            let json = json_of(&render_home_card(ctx).expect("render"));
            assert!(json.contains("setCompleted"), "{json}");
        });
}

/// Un modèle choisi deux fois sous le même id ne double pas la liste.
#[test]
#[serial]
fn a_template_lands_once_under_its_id() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        let id = Uuid::new_v4();
        for _ in 0..2 {
            create_checklist(
                ctx.clone(),
                CreateChecklistArgs {
                    template: "cleaning".into(),
                    id: Some(id),
                },
            )
            .expect("create");
        }
        let lists = all_lists().expect("lists");
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].uuid(), id);
        assert_eq!(lists[0].assignee(), (None, None));
    });
}
