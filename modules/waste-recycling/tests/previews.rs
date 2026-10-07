//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use serde_json::json;
use waste_recycling::{map_markers, render_explore_detail, render_home_card};

/// Les bacs d'une commune française type et leurs jours de collecte.
fn sample_config() -> serde_json::Value {
    json!({
        "bins": [
            {
                "id": "jaune",
                "title": { "fr": "Bac jaune", "en": "Yellow bin" },
                "items": {
                    "fr": "Emballages plastique et métal\nCartons et papiers",
                    "en": "Plastic and metal packaging\nCardboard and paper"
                },
                "color": "yellow",
                "location": { "fr": "Placard de l'entrée", "en": "Hallway cupboard" }
            },
            {
                "id": "verre",
                "title": { "fr": "Colonne à verre", "en": "Glass bank" },
                "items": { "fr": "Bouteilles et bocaux, sans bouchon", "en": "Bottles and jars, lids off" },
                "color": "green"
            },
            {
                "id": "biodechets",
                "title": { "fr": "Bac à biodéchets", "en": "Food waste bin" },
                "items": { "fr": "Épluchures, marc de café, restes de repas", "en": "Peelings, coffee grounds, leftovers" },
                "color": "brown",
                "location": { "fr": "Sous l'évier", "en": "Under the sink" }
            },
            {
                "id": "ordures",
                "title": { "fr": "Ordures ménagères", "en": "General waste" },
                "items": { "fr": "Tout le reste, en sac fermé", "en": "Everything else, in a closed bag" },
                "color": "grey"
            }
        ],
        // Les jours cochés disent quand, la phrase dit quoi : les deux se lisent ensemble, et c'est
        // exactement ce que l'aperçu doit montrer (§2.7).
        "collects_tue": true,
        "collects_fri": true,
        "collection_schedule": {
            "fr": "Mardi pour le bac jaune, vendredi pour les ordures ménagères.",
            "en": "Tuesday for the yellow bin, Friday for general waste."
        },
        "bin_room_steps": {
            "fr": "Sortez par le portail, allée de gauche\nPorte grise du local, ouverte de 7 h à 22 h\nDéposez vos sacs dans le bon bac",
            "en": "Out through the gate, left-hand path\nGrey door of the store, open 7am to 10pm\nPut your bags in the right bin"
        },
        "takeout_note": {
            "fr": "Sortez les bacs devant le portail la veille au soir, après 19 h.",
            "en": "Put the bins out by the gate the evening before, after 7pm."
        }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context.clone().run(render_explore_detail).expect("detail");
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    // Les points que ce module pose sur la carte du livret (§3). La démo les servait vides.
    let markers = context
        .clone()
        .run(|ctx| map_markers(ctx).expect("repères"));
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
        vec![("home.card", home)],
        Some(serde_json::to_value(markers).expect("repères")),
    );
}
