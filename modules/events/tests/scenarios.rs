//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use events::render_upcoming_card;
use portaki_sdk::capability;
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use serial_test::serial;

/// Déjà passé à l'horloge des cas (15 juin 2026, 10 h).
const PAST_EVENT: &str = "Fête de la musique anticipée";

/// L'agenda de l'hôte autour du 15 juin, dont un rendez-vous déjà passé, plus l'agenda
/// alentour d'OpenAgenda.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder
        .with_extra_capability_ids(&[capability::external::OPEN_AGENDA_POOL.as_str()])
        .with_connector_response("open-agenda", "nearby_events", nearby())
        .with_config(&json!({
            "events": [
                {
                    "id": "fete-anticipee",
                    "title": { "fr": PAST_EVENT, "en": "Early music festival" },
                    "place": { "fr": "Place du village", "en": "Village square" },
                    "starts_at": "2026-06-10T19:00:00Z"
                },
                {
                    "id": "marche-nocturne",
                    "title": { "fr": "Marché nocturne des artisans", "en": "Craft night market" },
                    "place": { "fr": "Place du village", "en": "Village square" },
                    "starts_at": "2026-06-17T16:00:00Z",
                    "ends_at": "2026-06-17T20:00:00Z",
                    "note": { "fr": "Entrée libre", "en": "Free entry" },
                    "lat": 43.5528, "lng": 7.0171
                },
                {
                    "id": "concert",
                    "title": { "fr": "Concert de jazz en plein air", "en": "Open-air jazz concert" },
                    "place": { "fr": "Jardin public", "en": "Public garden" },
                    "starts_at": "2026-06-24T18:30:00Z",
                    "url": "https://www.cannes.com/fr/agenda.html"
                },
                {
                    "id": "feu-artifice",
                    "title": { "fr": "Feu d'artifice du 14 juillet", "en": "Bastille Day fireworks" },
                    "place": { "fr": "Plage du Midi", "en": "Midi beach" },
                    "starts_at": "2026-07-14T20:00:00Z"
                }
            ],
            "disclaimer": { "fr": "Programme indicatif, à vérifier auprès des organisateurs.", "en": "Schedule for guidance, check with the organisers." },
            "nearby_enabled": true,
            "radius_km": 20
        }))
}

fn nearby() -> String {
    json!({
        "total": 1,
        "events": [{
            "uid": 56158955,
            "title": "Festival du port",
            "location": { "name": "Quai Saint-Pierre", "city": "Cannes", "latitude": 43.55, "longitude": 7.01 },
            "nextTiming": { "begin": "2026-06-20T17:00:00.000Z" },
            "canonicalUrl": "https://openagenda.com/events/festival-du-port"
        }]
    })
    .to_string()
}

#[test]
#[serial]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
#[serial]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}

/// La carte « à venir » annonce le prochain rendez-vous, jamais un déjà passé.
#[test]
#[serial]
fn the_upcoming_card_never_headlines_a_past_event() {
    check_each(|scenario| {
        let card = setup(scenario.guest())
            .run(render_upcoming_card)
            .map_err(|e| e.to_string())?;
        if serde_json::to_string(&card).unwrap().contains(PAST_EVENT) {
            Err("past event headlined".into())
        } else {
            Ok(())
        }
    });
}
