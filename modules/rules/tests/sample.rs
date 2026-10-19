//! Le règlement type d'une location de vacances, rangé comme le SDK le stocke.

use portaki_test_utils::MockContextBuilder;
use serde_json::json;

pub fn rules() -> serde_json::Value {
    json!([
        { "id": "r1", "title": { "fr": "Calme après 22 h" }, "detail": { "fr": "Merci de penser au voisinage" },
          "status": "important", "theme": "noise", "hours": "22:00-08:00" },
        { "id": "r2", "title": { "fr": "Logement non-fumeur" }, "detail": { "fr": "Vous pouvez fumer sur la terrasse" },
          "status": "important", "theme": "smoking" },
        { "id": "r3", "title": { "fr": "Pas de fête ni d'événement" }, "status": "forbidden", "theme": "parties" },
        { "id": "r4", "title": { "fr": "Animaux bienvenus" }, "detail": { "fr": "Prévenez-nous avant votre arrivée" },
          "status": "allowed", "theme": "pets" },
        { "id": "r5", "title": { "fr": "Départ avant 10 h" }, "detail": { "fr": "Laissez les clés sur la table de la cuisine" } }
    ])
}

pub fn seeded(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_resource("rules", rules(), json!({ "card_limit": 4 }))
}
