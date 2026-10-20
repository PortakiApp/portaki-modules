//! La reprise des signalements d'avant : des lignes `issue_report` / `issue_report_event` telles
//! que l'ancien module les a écrites (capturées depuis son code sur `main`, `tests/earlier/*.json`),
//! relues une fois.

use issue_report::{Category, Reports, Status};
use portaki_sdk::resource::{Actor, HistoryEvent};
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use uuid::Uuid;

fn stored(name: &str) -> Vec<Value> {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn host() -> MockContext {
    MockContext::host()
        .with_rows("IssueReport", stored("reports"))
        .with_rows("IssueReportEvent", stored("events"))
}

fn id(raw: &str) -> Uuid {
    Uuid::parse_str(raw).unwrap()
}

fn kinds(history: &[portaki_sdk::resource::HistoryLine]) -> Vec<String> {
    history
        .iter()
        .map(|line| match &line.event {
            HistoryEvent::Created => format!("created:{:?}", line.actor),
            HistoryEvent::Status { to } => format!("status:{to}"),
            HistoryEvent::Note { text } => format!("note:{text}"),
        })
        .collect()
}

#[test]
fn every_report_keeps_its_id_stay_guest_status_and_history() {
    host().run(|ctx| {
        let reports = Reports::list(&ctx).unwrap();
        assert_eq!(reports.len(), 4);
        let oven = Reports::get(&ctx, id("2935980e-b786-4caf-995d-b94a2e3c5cda"))
            .unwrap()
            .expect("the oven keeps its id");
        assert_eq!(oven.item.category, Category::Appliance);
        assert_eq!(oven.item.summary, "Le four ne chauffe plus");
        assert!(oven.item.photo.starts_with("portaki-file:"));
        assert_eq!(oven.status, Status::Resolved);
        assert_eq!(
            oven.stay_id,
            Some(id("5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54"))
        );
        assert_eq!(oven.guest_name, "Camille Martin");
        assert!(oven.check_in.is_some() && oven.check_out.is_some());
        assert_eq!(oven.actor, Actor::Guest);
        assert_eq!(
            kinds(&oven.history),
            [
                "created:Guest",
                "status:in_progress",
                "note:Le technicien passe demain à 9 h.",
                "status:resolved"
            ]
        );

        let wifi = Reports::get(&ctx, id("2473108c-4fa5-4f9b-bed4-fc33e6e7072f"))
            .unwrap()
            .unwrap();
        assert_eq!(wifi.status, Status::New, "reopened");
        assert_eq!(wifi.item.details, "");
        assert_eq!(
            kinds(&wifi.history),
            ["created:Guest", "status:resolved", "status:new"]
        );

        let leak = Reports::get(&ctx, id("496e78e6-7a56-4ad3-a61b-7dbbf4a3fa64"))
            .unwrap()
            .unwrap();
        assert_eq!(leak.actor, Actor::Host);
        assert_eq!(leak.guest_name, "");
        assert_eq!(leak.notes().collect::<Vec<_>>(), ["Plombier prévenu."]);

        let stay = id("8a2b3c4d-5e6f-4a7b-9c8d-0e1f2a3b4c5d");
        assert_eq!(Reports::for_stay(&ctx, stay).unwrap().len(), 2);
    });
}

/// Une ligne v1 (avant `status` et l'historique) : le statut se lit de `resolved_at`.
#[test]
fn a_row_from_before_the_status_column_reads_resolved_at() {
    let v1 = json!({
        "id": "0b5e1f6a-9c3d-4e2f-8a7b-6c5d4e3f2a1b",
        "stay_id": "5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54",
        "category": "heating",
        "summary": "Radiateur froid",
        "details": null,
        "created_at": "2026-09-01T08:00:00Z",
        "resolved_at": "2026-09-02T08:00:00Z",
        "photo": null,
        "status": "",
        "guest_name": "",
        "stay_check_in": null,
        "stay_check_out": null
    });
    MockContext::host()
        .with_rows("IssueReport", vec![v1])
        .run(|ctx| {
            let report = &Reports::list(&ctx).unwrap()[0];
            assert_eq!(report.status, Status::Resolved);
            assert_eq!(report.item.category, Category::Other);
            assert_eq!(kinds(&report.history), ["created:Guest", "status:resolved"]);
        });
}

#[test]
fn taken_over_once_and_never_over_the_records() {
    host()
        .with_resource(
            "reports",
            json!({ "category": "noise", "summary": "Déjà là" }),
            json!({}),
        )
        .run(|ctx| assert_eq!(Reports::list(&ctx).unwrap().len(), 1));
    host().run(|ctx| {
        assert_eq!(Reports::list(&ctx).unwrap().len(), 4);
        assert_eq!(
            Reports::list(&ctx).unwrap().len(),
            4,
            "not taken over twice"
        );
    });
}
