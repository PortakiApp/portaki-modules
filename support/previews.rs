//! Les aperçus du catalogue public : chaque surface voyageur, rendue sur des données d'exemple.
//!
//! La fiche d'un module sur portaki.app montre ce que le voyageur verra. Pas une capture : l'arbre
//! SDUI que le module produit, peint par le moteur du livret. Il est rendu ici, par le même code
//! que le binaire wasm, sur une configuration d'exemple — jamais sur la donnée d'un hôte.
//!
//! `previews.json` est committé à côté du manifeste : l'aperçu se relit dans une PR comme le reste,
//! `portaki build` l'emporte dans l'artefact, et le registre le sert avec la version. Ce test
//! échoue quand le fichier ne correspond plus à ce que le module rend ; pour le régénérer :
//!
//! ```sh
//! PORTAKI_UPDATE_PREVIEWS=1 cargo test -p <module> --test previews
//! ```
//!
//! Inclus par `#[path]` depuis `modules/*/tests/previews.rs` : un fichier de test, pas une crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::{json, Map, Value};

pub const LOCALE: &str = "fr-FR";
const FILE: &str = "previews.json";

/// Un contexte voyageur en français, les traductions du module chargées.
pub fn guest(module_root: &str) -> portaki_test_utils::MockContextBuilder {
    fr_bundle(module_root)
        .into_iter()
        .fold(MockContext::guest(), |builder, (key, value)| {
            builder.with_translation(key, value.as_str().unwrap_or_default())
        })
}

/// Compare `previews.json` à ce que les surfaces rendent, ou le réécrit.
///
/// Chaque surface voyageur du manifeste doit être rendue, et rien d'autre : un aperçu
/// d'une surface que le livret ne sert pas mentirait sur le module.
pub fn check(module_root: &str, rendered: Vec<(&str, Surface)>) {
    let root = Path::new(module_root);
    let manifest = read_json(&root.join("portaki.module.json"));
    let bundle = fr_bundle(module_root);

    let declared: Vec<&Value> = manifest["guestSurfaces"]
        .as_array()
        .map(|s| s.iter().collect())
        .unwrap_or_default();
    let ids: Vec<&str> = declared
        .iter()
        .filter_map(|s| s["surfaceId"].as_str())
        .collect();
    let got: Vec<&str> = rendered.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        got, ids,
        "un aperçu par surface voyageur du manifeste, dans son ordre"
    );

    let surfaces: Vec<Value> = rendered
        .into_iter()
        .zip(declared)
        .map(|((surface_id, surface), declared)| {
            let tree = serde_json::to_value(&surface.root).expect("arbre SDUI");
            let label_key = declared["labelKey"].as_str().unwrap_or_default();
            let mut keys = i18n_refs(&tree);
            keys.insert(label_key.to_string());
            let i18n: Map<String, Value> = keys
                .into_iter()
                .filter_map(|key| bundle.get(&key).map(|value| (key, value.clone())))
                .collect();
            json!({
                "surfaceId": surface_id,
                "title": bundle.get(label_key).cloned().unwrap_or(Value::Null),
                "tree": tree,
                "i18n": i18n,
            })
        })
        .collect();

    let expected = serde_json::to_string_pretty(&json!({ "locale": LOCALE, "surfaces": surfaces }))
        .expect("json")
        + "\n";
    let path = root.join(FILE);
    if std::env::var_os("PORTAKI_UPDATE_PREVIEWS").is_some() {
        fs::write(&path, &expected).expect("écrire previews.json");
        return;
    }
    let current = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == expected,
        "{FILE} ne correspond plus au rendu — PORTAKI_UPDATE_PREVIEWS=1 cargo test --test previews"
    );
}

fn fr_bundle(module_root: &str) -> Map<String, Value> {
    read_json(
        &Path::new(module_root)
            .join("i18n")
            .join(format!("{LOCALE}.json")),
    )
    .as_object()
    .cloned()
    .expect("bundle fr-FR")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("lecture")).expect("json")
}

/// Toutes les clés `i18n:` que l'arbre référence.
fn i18n_refs(tree: &Value) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut stack = vec![tree];
    while let Some(value) = stack.pop() {
        match value {
            Value::String(text) => {
                if let Some(key) = text.strip_prefix("i18n:") {
                    keys.insert(key.to_string());
                }
            }
            Value::Array(items) => stack.extend(items),
            Value::Object(fields) => stack.extend(fields.values()),
            _ => {}
        }
    }
    keys
}
