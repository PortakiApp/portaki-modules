//! The six statuses of the spec (§1): what the guest reads for each, the host's transitions, the
//! workspace tab and the « Objet à renvoyer » task.

use chrono::{DateTime, Duration, Utc};
use serial_test::serial;
use uuid::Uuid;

use lost_found::{
    list_for_stay, render_home_card, render_host_items, reset_test_store, submit, task_complete,
    task_toggle, timeline_tasks, update_status, ListForStayArgs, SubmitArgs, UpdateStatusArgs,
};
use portaki_sdk::contracts::timeline::{
    TaskCompleteArgs, TaskToggleArgs, TimelineStay, TimelineTask, TimelineTasksArgs,
};
use portaki_test_utils::{MockContext, MockContextBuilder, Property};

fn t0() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-09-01T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// A guest stay with one declaration — `ship` when asked — and its report id.
fn declared(choice: Option<&str>) -> (MockContextBuilder, Uuid, Uuid) {
    let builder = MockContext::guest()
        .with_property(Property::default())
        .with_now(t0());
    let stay_id = builder.context().guest.expect("guest").session_id;
    let mut report_id = Uuid::nil();
    builder.clone().run(|ctx| {
        submit(
            ctx.clone(),
            SubmitArgs {
                kind: "lost".into(),
                item_description: "Chargeur".into(),
                return_choice: choice.map(str::to_string),
                ..Default::default()
            },
        )
        .expect("submit");
        report_id = list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0].id;
    });
    (builder, stay_id, report_id)
}

fn host() -> MockContextBuilder {
    MockContext::host()
        .with_property(Property::default())
        .with_now(t0() + Duration::days(2))
}

fn move_to(report_id: Uuid, status: &str) -> Result<(), String> {
    let mut out = Ok(());
    host().run(|ctx| {
        out = update_status(
            ctx,
            UpdateStatusArgs {
                report_id,
                status: status.into(),
            },
        )
        .map_err(|error| error.to_string());
    });
    out
}

fn guest_card(builder: &MockContextBuilder) -> String {
    let mut json = String::new();
    builder.clone().run(|ctx| {
        json = serde_json::to_string(&render_home_card(ctx).expect("card")).expect("json");
    });
    json
}

#[test]
#[serial]
fn the_guest_reads_a_message_for_each_status() {
    // declared → not_found → found → shipped
    reset_test_store();
    let (guest, _, report) = declared(None);
    assert!(guest_card(&guest).contains("guest.status.declared"));
    for status in ["not_found", "found", "shipped"] {
        move_to(report, status).expect(status);
        assert!(
            guest_card(&guest).contains(&format!("guest.status.{status}")),
            "{status}"
        );
    }
    // picked_up and donated, each from found.
    for status in ["picked_up", "donated"] {
        reset_test_store();
        let (guest, _, report) = declared(None);
        move_to(report, "found").expect("found");
        move_to(report, status).expect(status);
        assert!(guest_card(&guest).contains(&format!("guest.status.{status}")));
    }
}

#[test]
#[serial]
fn the_host_cannot_move_an_item_backwards_or_out_of_a_return() {
    reset_test_store();
    let (_, _, report) = declared(None);
    move_to(report, "found").expect("found");
    assert!(move_to(report, "declared").is_err());
    assert!(move_to(report, "not_found").is_err());
    move_to(report, "donated").expect("donated");
    assert!(move_to(report, "found").is_err());
    assert!(move_to(report, "lost").is_err());
    // Staying put is not a move.
    move_to(report, "donated").expect("same status");
}

#[test]
#[serial]
fn the_workspace_tab_lists_items_with_the_next_statuses() {
    reset_test_store();
    let (_, _, report) = declared(Some("ship"));
    let render = || {
        let mut json = String::new();
        host().run(|ctx| {
            json = serde_json::to_string(&render_host_items(ctx).expect("tab")).expect("json");
        });
        json
    };
    let json = render();
    assert!(json.contains("status.declared"), "{json}");
    assert!(json.contains("status.not_found"), "{json}");
    assert!(json.contains("host.item.choice.ship"), "{json}");
    assert!(json.contains("Select"), "{json}");

    move_to(report, "shipped").expect("shipped");
    let json = render();
    assert!(json.contains("status.shipped"), "{json}");
    // Nothing left to choose once the item is back.
    assert!(!json.contains("\"Select\""), "{json}");
}

fn tasks(stays: Vec<TimelineStay>) -> Vec<TimelineTask> {
    let mut out = Vec::new();
    host().run(|ctx| {
        out = timeline_tasks(
            ctx,
            TimelineTasksArgs {
                property_id: Uuid::nil(),
                from: t0() - Duration::days(7),
                to: t0() + Duration::days(7),
                stays,
            },
        )
        .expect("tasks")
        .tasks;
    });
    out
}

#[test]
#[serial]
fn an_item_to_ship_is_a_task_until_shipped() {
    reset_test_store();
    let (_, stay_id, report) = declared(Some("ship"));
    declared(Some("pickup")); // no task: the guest comes for it

    let stays = vec![TimelineStay {
        id: stay_id,
        check_in: t0() - Duration::days(5),
        check_out: t0() - Duration::days(1),
        guest_name: "Marie".into(),
        status: "COMPLETED".into(),
    }];
    let listed = tasks(stays);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title.fr, "Objet à renvoyer à Marie");
    assert_eq!(listed[0].context.fr, "Chargeur · déclaré il y a 2 j");
    assert_eq!(listed[0].stay_id, Some(stay_id));
    assert_eq!(listed[0].at, t0());
    assert_eq!(tasks(Vec::new())[0].title.fr, "Objet à renvoyer");

    let task_id = listed[0].id.clone();
    host().run(|ctx| {
        let untick = task_toggle(
            ctx.clone(),
            TaskToggleArgs {
                property_id: Uuid::nil(),
                task_id: task_id.clone(),
                item_id: "shipped".into(),
                done: false,
                photo: None,
            },
        );
        assert!(untick.is_err());
        task_complete(
            ctx.clone(),
            TaskCompleteArgs {
                property_id: Uuid::nil(),
                task_id: task_id.clone(),
            },
        )
        .expect("complete");
        let rows = list_for_stay(
            ctx,
            ListForStayArgs {
                stay_id: Some(stay_id),
            },
        )
        .expect("list");
        assert_eq!(rows[0].id, report);
        assert_eq!(rows[0].status, "shipped");
    });
    assert!(tasks(Vec::new()).is_empty());
}
