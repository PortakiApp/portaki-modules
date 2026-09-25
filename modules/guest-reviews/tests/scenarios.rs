//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use guest_reviews::{render_home_card, submit_review, SubmitReviewArgs};
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;

/// L'avis déposé sur Portaki, et le lien Airbnb pour les séjours réservés là-bas.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "platform_airbnb": true,
        "platform_portaki": true,
        "show_qr_code": true,
        "airbnb_review_url": "https://www.airbnb.fr/users/review/123456",
        "thank_you_message": "Merci pour votre séjour ! Votre avis aide les prochains voyageurs à nous choisir."
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

/// Un séjour qui n'a pas commencé ne se note pas : la carte n'offre le formulaire qu'une fois
/// le voyageur arrivé, et la commande refuse un avis déposé avant.
#[test]
fn a_review_waits_for_the_arrival() {
    check_each(|scenario| {
        let arrived = scenario.stay.check_in_offset < 0;
        let card = setup(scenario.guest())
            .run(render_home_card)
            .map_err(|e| e.to_string())?;
        let offered = serde_json::to_string(&card)
            .unwrap()
            .contains("submitReview");
        let submitted = setup(scenario.guest())
            .run(|ctx| {
                submit_review(
                    ctx,
                    SubmitReviewArgs {
                        rating: 5,
                        comment: String::new(),
                    },
                )
            })
            .is_ok();
        if offered == arrived && submitted == arrived {
            Ok(())
        } else {
            Err(format!(
                "form offered: {offered}, review accepted: {submitted}, arrival in {} day(s)",
                scenario.stay.check_in_offset
            ))
        }
    });
}
