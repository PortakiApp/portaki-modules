//! Le mot de l'hôte du livret d'exemple, rangé comme le SDK le stocke.
#![allow(dead_code)] // chaque fichier de test en prend une partie

use portaki_test_utils::MockContextBuilder;
use serde_json::json;

pub fn sections() -> serde_json::Value {
    json!([
        { "id": "s1",
          "title": { "fr": "Bienvenue à L'Islette", "en": "Welcome to L'Islette" },
          "body": { "fr": "Nous sommes heureux de vous accueillir.\n\nFaites comme chez vous : le café est dans le placard au-dessus de la machine, et les serviettes de plage sont dans le coffre de l'entrée.",
                    "en": "We are glad to have you.\n\nMake yourself at home: the coffee is in the cupboard above the machine, and the beach towels are in the hallway chest." } }
    ])
}

/// Deux sections, l'une traduite, l'autre en français seulement.
pub fn two() -> serde_json::Value {
    json!([
        { "id": "s1", "title": { "fr": "Bienvenue", "en": "Welcome" },
          "body": { "fr": "Bienvenue à **L'Islette** !", "en": "Welcome to **L'Islette**!" } },
        { "id": "s2", "title": { "fr": "L'appartement" }, "body": { "fr": "2 chambres, terrasse vue mer." } }
    ])
}

pub fn seeded(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_resource("sections", sections(), json!({}))
}

pub fn seeded_two(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_resource("sections", two(), json!({}))
}
