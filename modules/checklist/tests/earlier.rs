//! La reprise des checklists d'avant : des lignes `checklist` / `checklist_item` et des clés KV
//! `display:{listId}` telles que l'ancien module les a écrites (capturées depuis son code sur
//! `main`, `tests/earlier/*.json`), relues une fois.

use checklist::{
    all_lists, complete_item, list_completions, render_home_card, Audience, ItemIdArgs, List,
};
use portaki_test_utils::{MockContext, MockContextBuilder, Property};
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

fn stored(name: &str) -> Value {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn rows(name: &str) -> Vec<Value> {
    stored(name).as_array().unwrap().clone()
}

/// L'ancien logement : ses listes, ses étapes et ses réglages d'affichage.
fn earlier(builder: MockContextBuilder) -> MockContextBuilder {
    let mut builder = builder
        .with_property(Property::default())
        .with_rows("Checklist", rows("lists"))
        .with_rows("ChecklistItem", rows("items"));
    for (key, value) in stored("kv").as_object().unwrap() {
        builder = builder.with_kv(key, serde_json::to_vec(value).unwrap());
    }
    builder
}

/// L'id de la première étape de la liste de départ, telle que l'ancien module l'a stockée.
fn first_step_id() -> String {
    let list = rows("lists")[0]["id"].clone();
    let item = rows("items")
        .into_iter()
        .find(|row| row["checklist_id"] == list && row["sort_order"] == 0)
        .unwrap();
    item["id"].as_str().unwrap().to_string()
}

fn texts(list: &List) -> Vec<(&str, &str)> {
    list.steps
        .iter()
        .map(|s| (s.label.get("fr"), s.label.get("en")))
        .collect()
}

#[test]
#[serial]
fn the_lists_and_their_steps_become_the_collection() {
    earlier(MockContext::host()).run(|_ctx| {
        let lists = all_lists().unwrap();
        let ids: Vec<&str> = lists.iter().map(|l| l.id.as_str()).collect();
        let old: Vec<Value> = rows("lists").iter().map(|l| l["id"].clone()).collect();
        assert_eq!(
            ids,
            old.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>()
        );

        let departure = &lists[0];
        assert_eq!(departure.name.get("fr"), "Avant de partir");
        assert_eq!(departure.trigger(), "departureDay");
        assert_eq!(departure.steps.len(), 5, "the blank step is not taken over");
        assert_eq!(departure.steps[0].id, first_step_id());
        let first = &departure.steps[0];
        assert_eq!(
            (first.label.get("fr"), first.label.get("en")),
            ("Fermer toutes les fenêtres", "Close every window")
        );
        assert_eq!(first.detail.get("en"), "Bathroom included");
        assert_eq!(first.zone.get("fr"), "Dans le logement");
        assert_eq!(departure.visible_limit, Some(7));
        assert_eq!(departure.remind, Some(false));
        assert_eq!(departure.done_message.get("en"), "Thanks, safe trip!");
        assert_eq!(departure.done_message.get("fr"), "Merci, bon retour !");

        let arrival = &lists[1];
        assert_eq!(arrival.trigger(), "duringStay");
        assert_eq!(arrival.visible_limit, Some(12), "kept as typed");
        assert_eq!(arrival.visible_limit(), 10);
        assert_eq!(
            texts(arrival),
            [
                ("Récupérer les clés", "Pick up the keys"),
                ("Allumer le chauffe-eau", "Allumer le chauffe-eau")
            ]
        );
        assert_eq!(arrival.steps[0].zone.get("fr"), "Entrée");

        let cleaning = &lists[2];
        assert_eq!(cleaning.audience, Audience::Host);
        assert_eq!(cleaning.trigger(), "onlyIfNextArrival");
        assert_eq!(cleaning.deadline(), Some("departureEvening"));
        assert_eq!(
            cleaning.assignee(),
            (Some("Julie Martin".into()), Some("Ménage".into()))
        );
        assert!(!cleaning.notify_assignee && cleaning.alert_host);
        assert_eq!(cleaning.steps.len(), 7);
        assert!(cleaning.steps[6].photo && !cleaning.steps[5].photo);
    });
}

/// Une coche désigne l'étape par l'id de l'ancienne ligne : il se retrouve après la reprise, et la
/// carte du livret la montre cochée. (Les coches vivent en mémoire sous test, `with_rows` ne les
/// sème pas : la coche passe par `completeItem`, qui refuserait une étape inconnue.)
#[test]
#[serial]
fn a_tick_on_an_old_step_stays_ticked() {
    earlier(MockContext::guest()).run(|ctx| {
        let item_id = Uuid::parse_str(&first_step_id()).unwrap();
        complete_item(ctx.clone(), ItemIdArgs { item_id }).expect("the old step is found");
        assert_eq!(list_completions(ctx.clone()).expect("done"), vec![item_id]);
        let json = serde_json::to_string(&render_home_card(ctx).unwrap()).unwrap();
        assert!(json.contains(&format!("\"value\":\"{item_id}\"")), "{json}");
        assert!(json.contains("Fermer toutes les fenêtres"), "{json}");
    });
}

#[test]
#[serial]
fn the_old_show_when_moves_the_first_guest_list() {
    earlier(MockContext::host())
        .with_kv("config", br#"{"show_when":"always"}"#.to_vec())
        .run(|_ctx| {
            let lists = all_lists().unwrap();
            assert_eq!(lists[0].trigger(), "beforeArrival");
            assert_eq!(lists[1].trigger(), "duringStay");
        });
}

#[test]
#[serial]
fn a_bare_label_and_its_english_column_are_read() {
    let mut items = rows("items");
    items[0]["label_fr"] = json!("Fermer les volets");
    items[0]["label_en"] = json!("Close the shutters");
    MockContext::host()
        .with_rows("Checklist", rows("lists"))
        .with_rows("ChecklistItem", items)
        .run(|_ctx| {
            let step = &all_lists().unwrap()[0].steps[0];
            assert_eq!(
                (step.label.get("fr"), step.label.get("en")),
                ("Fermer les volets", "Close the shutters")
            );
        });
}

#[test]
#[serial]
fn taken_over_once_and_never_over_the_collection() {
    earlier(MockContext::host())
        .with_resource(
            "lists",
            json!([{ "id": "x", "name": { "fr": "Déjà là" } }]),
            json!({}),
        )
        .run(|_ctx| assert_eq!(all_lists().unwrap().len(), 1));
    earlier(MockContext::host()).run(|ctx| {
        assert_eq!(all_lists().unwrap().len(), 3);
        checklist::__portaki_lists_update_config(ctx, json!({ "lists": [] })).unwrap();
        assert!(all_lists().unwrap().is_empty(), "emptied lists stay empty");
    });
}
