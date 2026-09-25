//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_sdk::prelude::EmptyArgs;
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use pre_arrival_form::{render_home_card, reset_test_store, send_form_available};
use serde_json::json;
use serial_test::serial;

/// Les questions du modèle, formulaire ouvert 48 h avant l'arrivée.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "show_when": "before",
        "ask_arrival_time": true,
        "ask_occasion": true,
        "ask_allergies": true,
        "ask_guest_count": true,
        "ask_special_needs": false,
        "ask_id_document": false
    }))
}

#[test]
#[serial]
fn every_surface_holds_on_every_case() {
    reset_test_store();
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
#[serial]
fn every_example_runs() {
    reset_test_store();
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
    reset_test_store();
}

/// Ouvert dès la réservation : l'e-mail part tant que l'arrivée est à venir, jamais après.
#[test]
#[serial]
fn form_available_email_stops_at_check_in() {
    check_each(|scenario| {
        reset_test_store();
        let sent = scenario
            .guest()
            .with_config(&json!({ "show_when": "confirm" }))
            .run_with(|ctx, host| {
                send_form_available(ctx, EmptyArgs {}).map(|()| host.sent_emails().len())
            })
            .map_err(|e| e.to_string())?;
        let due = scenario.stay.check_in_offset > 0;
        if (sent == 1) == due {
            Ok(())
        } else {
            Err(format!(
                "{sent} e-mail(s), arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}

/// La carte d'accueil ne propose le formulaire qu'à 48 h de l'arrivée, et plus après : l'envoi
/// serait refusé. Arrivée à 16 h, horloge à 10 h : J+1 est à 30 h, J+2 déjà à 54 h.
#[test]
#[serial]
fn home_task_follows_the_arrival_date() {
    check_each(|scenario| {
        reset_test_store();
        let card = setup(scenario.guest())
            .run(render_home_card)
            .map_err(|e| e.to_string())?;
        let offered = serde_json::to_string(&card)
            .unwrap()
            .contains("home.task.preArrival.label");
        let due = (0..=1).contains(&scenario.stay.check_in_offset);
        if offered == due {
            Ok(())
        } else {
            Err(format!(
                "task offered: {offered}, arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}
