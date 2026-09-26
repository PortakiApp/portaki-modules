//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use wifi_guest::render_explore_detail;

const PASSWORD: &str = "soleil-2026";

/// Un réseau dont le mot de passe se découvre la veille à 16 h : c'est la date d'arrivée qui
/// décide de ce que le voyageur lit.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "ssid": "Maison-Invites",
        "password": PASSWORD,
        "hint": "Le réseau 5 GHz est plus rapide dans le salon.",
        "connection_steps": "Choisissez « Maison-Invites » dans les réglages Wi-Fi, puis saisissez le mot de passe.",
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

/// Le mot de passe n'est montré qu'à partir de la veille de l'arrivée, 16 h. L'horloge des cas
/// est figée à 10 h : une arrivée demain n'est pas encore due.
#[test]
fn the_password_follows_the_arrival_date() {
    check_each(|scenario| {
        let surface = setup(scenario.guest())
            .run(render_explore_detail)
            .map_err(|e| e.to_string())?;
        let shown = serde_json::to_string(&surface).unwrap().contains(PASSWORD);
        // Up to departure, never after.
        let due = scenario.stay.check_in_offset <= 0 && scenario.stay.check_out_offset >= 0;
        if shown == due {
            Ok(())
        } else {
            Err(format!(
                "password shown: {shown}, arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}
