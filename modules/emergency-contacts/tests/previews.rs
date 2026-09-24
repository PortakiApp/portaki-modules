//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use emergency_contacts::render_explore_detail;
use portaki_test_utils::Property;
use serde_json::json;

/// Des numéros d'exemple : les services nationaux, et un médecin et un hôte fictifs.
fn sample_config() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "contacts": [
            { "id": "pharmacie", "label": { "fr": "Pharmacie de garde", "en": "On-call pharmacy" }, "phone": "3237", "note": { "fr": "Service national", "en": "National service" } },
            { "id": "medecin", "label": { "fr": "Médecin généraliste", "en": "General practitioner" }, "phone": "04 93 00 00 00", "note": { "fr": "Dr Martin, sur rendez-vous", "en": "Dr Martin, by appointment" } },
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15" }
        ],
        "host_visible_phone": "06 00 00 00 00"
    }))
    .expect("config json")
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_property(Property::default())
        .with_kv("config", sample_config())
        .run(render_explore_detail);
    previews::check(root, vec![("explore.detail", detail)]);
}
