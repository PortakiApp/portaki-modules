//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use emergency_contacts::render_explore_detail;

use serde_json::json;

/// Des numéros d'exemple : les services nationaux, et un cabinet et un hôte fictifs, aux numéros des plages réservées à la fiction.
fn sample_config() -> serde_json::Value {
    json!({
        "contacts": [
            { "id": "pharmacie", "label": { "fr": "Pharmacie de garde", "en": "On-call pharmacy" }, "phone": "3237", "note": { "fr": "Service national", "en": "National service" } },
            { "id": "medecin", "label": { "fr": "Médecin généraliste", "en": "General practitioner" }, "phone": "01 99 00 12 34", "note": { "fr": "Cabinet du centre, sur rendez-vous", "en": "Town centre practice, by appointment" } },
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15" }
        ],
        "host_visible_phone": "06 39 98 12 34"
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_config(&sample_config())
        .run(render_explore_detail)
        .expect("detail");
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
