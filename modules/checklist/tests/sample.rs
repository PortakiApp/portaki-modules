//! Les listes telles que les modèles les créent, rangées comme le SDK les stocke.
#![allow(dead_code)] // chaque fichier de test en prend une partie

use checklist::{create_checklist, CreateChecklistArgs, Lists};
use portaki_test_utils::{MockContext, MockContextBuilder};
use serde_json::{json, Value};

/// Les lignes de la collection après `createChecklist` de chaque modèle.
pub fn lists(templates: &[&str]) -> Value {
    MockContext::host().run(|ctx| {
        for template in templates {
            let args = CreateChecklistArgs {
                template: (*template).into(),
                id: None,
            };
            create_checklist(ctx.clone(), args).expect("create");
        }
        serde_json::to_value(Lists::load(&ctx).expect("load").rows()).expect("json")
    })
}

pub fn with(builder: MockContextBuilder, lists: Value) -> MockContextBuilder {
    builder.with_resource("lists", lists, json!({}))
}

/// La liste de départ du voyageur et le ménage de l'hôte.
pub fn seeded(builder: MockContextBuilder) -> MockContextBuilder {
    with(builder, lists(&["departure", "cleaning"]))
}
