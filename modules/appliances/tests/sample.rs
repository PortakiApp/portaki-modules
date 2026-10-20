//! Trois appareils courants d'une location, dont un seul mis en avant, rangés comme le SDK les
//! stocke.
#![allow(dead_code)] // chaque fichier de test en prend une partie

use portaki_test_utils::MockContextBuilder;
use serde_json::{json, Value};

pub fn doc(paragraphs: &[&str]) -> String {
    let blocks: Vec<Value> = paragraphs
        .iter()
        .map(|text| json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] }))
        .collect();
    json!({ "type": "doc", "content": blocks }).to_string()
}

pub fn devices() -> Value {
    json!([
        { "id": "plaques", "name": { "fr": "Plaques à induction" }, "emoji": "🍳",
          "location": { "fr": "Cuisine" }, "featured": true,
          "description": { "fr": doc(&["Appuyez 2 secondes sur la touche marche, puis choisissez le foyer.",
                                       "Seules les casseroles à fond aimanté chauffent."]) },
          "manual_url": "https://example.com/notice-plaques.pdf",
          "safety_note": { "fr": "La surface reste chaude quelques minutes après l'arrêt." },
          "steps": [
            { "id": "p1", "text": { "fr": "Posez une casserole à fond aimanté au centre du foyer." },
              "image": "portaki-file:plaques-induction-1" },
            { "id": "p2", "text": { "fr": "Appuyez 2 secondes sur la touche marche, puis choisissez la puissance." } }
          ] },
        { "id": "lave-linge", "name": { "fr": "Lave-linge" }, "emoji": "🌀",
          "location": { "fr": "Salle de bain" },
          "description": { "fr": doc(&["Programme « Coton 40° » pour un lavage courant, lessive dans le placard du dessous."]) },
          "safety_note": { "fr": "Merci de ne pas lancer de machine après 22 h." } },
        { "id": "tv", "name": { "fr": "Télévision" }, "emoji": "📺", "location": { "fr": "Salon" },
          "description": { "fr": doc(&["Allumez avec la télécommande noire ; la source « HDMI 1 » donne accès aux applications."]) } }
    ])
}

pub fn seeded(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_resource("devices", devices(), json!({}))
}

pub fn with(builder: MockContextBuilder, devices: Value, meta: Value) -> MockContextBuilder {
    builder.with_resource("devices", devices, meta)
}
