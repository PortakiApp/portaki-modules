//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use ev_parking::render_explore_detail;
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;

const CHARGER_PIN: &str = "2468";
const PARKING_CODE: &str = "1357";

/// Une borne en sous-sol dont les codes se découvrent la veille à 16 h : c'est la date
/// d'arrivée qui décide de ce que le voyageur lit.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "spot_label": "Place n° 8, niveau -1",
        "charger_pin": CHARGER_PIN,
        "parking_code": PARKING_CODE,
        "map_url": "https://maps.example.com/parking",
        "instructions": "Branchez le câble Type 2 fourni, puis saisissez le PIN sur la borne.",
        "reveal_policy": "day_before_16h"
    }))
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}

/// Le PIN de la borne et le code du parking ne sont montrés qu'à partir de la veille de
/// l'arrivée, 16 h. L'horloge des cas est figée à 10 h : une arrivée demain n'est pas encore due.
#[test]
fn the_codes_follow_the_arrival_date() {
    check_each(|scenario| {
        let surface = setup(scenario.guest())
            .run(render_explore_detail)
            .map_err(|e| e.to_string())?;
        let text = serde_json::to_string(&surface).unwrap();
        let due = scenario.stay.check_in_offset <= 0;
        let pin = text.contains(CHARGER_PIN);
        let code = text.contains(PARKING_CODE);
        if pin == due && code == due {
            Ok(())
        } else {
            Err(format!(
                "pin shown: {pin}, code shown: {code}, arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}
