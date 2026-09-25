//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use std::time::Duration;

use nuki::{get_guest_credential, ModuleConfig, StayArgs};
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;

/// Une serrure configurée avec son code clavier : sans code ni clé Nuki Web, rien à ouvrir.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&ModuleConfig {
        smartlock_id: "lock-abc".into(),
        keypad_code: "482910".into(),
        device_name: "Porte d'entrée".into(),
    })
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}

/// Le code ne se donne que de 8 h avant l'arrivée jusqu'au départ : à l'arrivée oui, une heure
/// après le départ non, et à l'horloge du cas seulement si le séjour est en cours — jamais une
/// semaine avant, ni après un séjour terminé.
#[test]
fn the_code_is_only_given_during_the_stay() {
    check_each(|scenario| {
        let given = |builder: MockContextBuilder| {
            setup(builder)
                .run(|ctx| get_guest_credential(ctx, StayArgs::default()))
                .is_ok()
        };
        let during = scenario.stay.check_in_offset <= 0 && scenario.stay.check_out_offset >= 0;
        let today = given(scenario.guest());
        let at_arrival = given(scenario.guest().with_now(scenario.check_in()));
        let after_departure = given(
            scenario
                .guest()
                .with_now(scenario.check_out() + Duration::from_secs(3600)),
        );
        if today == during && at_arrival && !after_departure {
            Ok(())
        } else {
            Err(format!(
                "code today: {today}, at arrival: {at_arrival}, after departure: \
                 {after_departure}"
            ))
        }
    });
}
