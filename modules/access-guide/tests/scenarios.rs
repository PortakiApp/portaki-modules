//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use access_guide::{email_context, render_explore_detail, EmailContextArgs};
use portaki_sdk::prelude::EmailTemplateKey;
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;

const CODE: &str = "4821";

/// Une boîte à clés dont le code se découvre la veille à 16 h : c'est la date d'arrivée qui
/// décide de ce que le voyageur lit.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "primary_method": "keybox",
        "keybox_location": "Sous la boîte aux lettres",
        "keybox_code": CODE,
        "address": "12 rue des Oliviers, Lyon",
        "reveal_policy": "day_before_16h",
        "method_instructions_fr": "Composez le code, les clés sont sur le crochet."
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

/// Le code n'est montré qu'à partir de la veille de l'arrivée, 16 h. L'horloge des cas est figée
/// à 10 h : une arrivée demain n'est pas encore due.
#[test]
fn the_code_follows_the_arrival_date() {
    check_each(|scenario| {
        let surface = setup(scenario.guest())
            .run(render_explore_detail)
            .map_err(|e| e.to_string())?;
        let shown = serde_json::to_string(&surface).unwrap().contains(CODE);
        // Up to departure, never after.
        let due = scenario.stay.check_in_offset <= 0 && scenario.stay.check_out_offset >= 0;
        if shown == due {
            Ok(())
        } else {
            Err(format!(
                "code shown: {shown}, arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}

/// L'e-mail du jour d'arrivée suit la même règle que le livret.
#[test]
fn the_arrival_email_never_leaks_a_locked_code() {
    check_each(|scenario| {
        let args = EmailContextArgs {
            template_key: Some(EmailTemplateKey::ArrivalDay),
            checkin_time_formatted: Some("16:00".into()),
            locale: None,
        };
        let response = setup(scenario.guest())
            .run(|ctx| email_context(ctx, args))
            .map_err(|e| e.to_string())?;
        if response.entry_access_code.is_some() && !response.secrets_revealed {
            Err("code sent while locked".into())
        } else {
            Ok(())
        }
    });
}
