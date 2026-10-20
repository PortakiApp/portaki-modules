//! Le catalogue, rangé comme le SDK le stocke, et les réglages des demandes.
#![allow(dead_code)] // chaque fichier de test en prend une partie

use portaki_sdk::prelude::Context;
use portaki_test_utils::MockContextBuilder;
use serde_json::{json, Value};
use uuid::Uuid;

/// L'id du `n`-ième produit semé (à partir de 0).
pub fn id(n: usize) -> Uuid {
    Uuid::from_u128(0xc0ffee00 + n as u128)
}

/// `(fr, en, emoji)` → les lignes de la collection.
pub fn products(items: &[(&str, &str, &str)]) -> Value {
    Value::Array(
        items
            .iter()
            .enumerate()
            .map(|(n, (fr, en, emoji))| {
                let mut label = json!({});
                if !fr.is_empty() {
                    label["fr"] = json!(fr);
                }
                if !en.is_empty() {
                    label["en"] = json!(en);
                }
                json!({ "id": id(n).to_string(), "label": label, "emoji": emoji })
            })
            .collect(),
    )
}

/// La liste de départ de `seedDefaults`.
pub fn starter() -> Value {
    products(&[
        ("Café", "Coffee", "☕"),
        ("Thé", "Tea", "🍵"),
        ("Papier toilette", "Toilet paper", "🧻"),
        ("Liquide vaisselle", "Dish soap", "🧽"),
        ("Sel", "Salt", "🧂"),
        ("Huile", "Oil", "🫒"),
    ])
}

pub fn with(builder: MockContextBuilder, items: &[(&str, &str, &str)]) -> MockContextBuilder {
    builder.with_resource("products", products(items), json!({}))
}

pub fn with_meta(
    builder: MockContextBuilder,
    items: &[(&str, &str, &str)],
    meta: Value,
) -> MockContextBuilder {
    builder.with_resource("products", products(items), meta)
}

pub fn seeded(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_resource("products", starter(), json!({}))
}

/// Ce que l'autosave du formulaire envoie, tel quel.
pub fn save(ctx: &Context, form: Value) {
    consumables::__portaki_products_update_config(ctx.clone(), form).expect("updateConfig");
}

/// Le message en français sous `field` (`notify`, `products.<id>.label`…), s'il bloque.
pub fn blocking(ctx: &Context, field: &str) -> Option<String> {
    consumables::__portaki_products_publish_readiness(ctx.clone())
        .expect("readiness")
        .items
        .into_iter()
        .find(|item| item.id.ends_with(field))
        .map(|item| item.hint.get("fr").to_string())
}

/// Le formulaire du voyageur, envoyé à la commande `submit` générée.
pub fn submit(ctx: &Context, args: consumables::SubmitArgs) -> portaki_sdk::Result<()> {
    consumables::__portaki_requests_submit(ctx.clone(), serde_json::to_value(args).unwrap())
}

/// Les demandes du logement, les plus récentes d'abord.
pub fn requests(ctx: &Context) -> Vec<consumables::Entry> {
    consumables::Restocks::list(ctx).expect("requests")
}

/// Les demandes pas encore livrées.
pub fn open_count(ctx: &Context) -> usize {
    requests(ctx)
        .iter()
        .filter(|r| r.status != consumables::Status::Restocked)
        .count()
}

/// Le même logement, vu du tableau de bord.
pub fn as_host(ctx: &Context) -> Context {
    let mut host = ctx.clone();
    host.guest = None;
    host
}
