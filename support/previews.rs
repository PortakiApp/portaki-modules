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

use portaki_sdk::context::StayContext;
use portaki_sdk::prelude::{DateTime, Utc, Uuid};
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::{json, Map, Value};

pub const LOCALE: &str = "fr-FR";
const FILE: &str = "previews.json";

/// Un contexte voyageur en français, les traductions du module chargées.
///
/// Le logement des fixtures, un séjour du 1er au 8 juin 2026 et une horloge figée la veille
/// de l'arrivée : un aperçu ne doit dépendre ni du jour où le test tourne, ni d'un
/// identifiant tiré au hasard.
pub fn guest(module_root: &str) -> portaki_test_utils::MockContextBuilder {
    let stay = StayContext {
        stay_id: Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222),
        checkin_at: Some(at("2026-06-01T15:00:00Z")),
        checkout_at: Some(at("2026-06-08T10:00:00Z")),
        ..StayContext::default()
    };
    fr_bundle(module_root)
        .into_iter()
        .fold(MockContext::guest(), |builder, (key, value)| {
            builder.with_translation(key, value.as_str().unwrap_or_default())
        })
        .with_property(Property::default())
        .with_stay(stay)
        .with_now(at(NOW))
}

/// L'instant des aperçus : la veille de l'arrivée.
pub const NOW: &str = "2026-05-31T10:00:00Z";

/// Un instant RFC 3339, en UTC.
pub fn at(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("date")
        .with_timezone(&Utc)
}

/// Compare `previews.json` à ce que les surfaces rendent, ou le réécrit.
///
/// Chaque surface voyageur que le livret sert — un `#[surface(guest, path = …)]` — doit être
/// rendue, et rien d'autre : un aperçu d'une surface que le livret ne sert pas mentirait sur le
/// module.
///
/// `emissions` est le dossier où les macros du module ont écrit ses déclarations en compilant :
/// `concat!(env!("OUT_DIR"), "/portaki-emissions")` depuis le test. Le manifeste n'est écrit que
/// par `portaki build`, qui ne tourne pas avant `cargo test`.
pub fn check(module_root: &str, emissions: &str, rendered: Vec<(&str, Surface)>) {
    let root = Path::new(module_root);
    let bundle = fr_bundle(module_root);
    let declared = guest_routes(Path::new(emissions));

    let mut got: Vec<&str> = rendered.iter().map(|(id, _)| *id).collect();
    got.sort_unstable();
    let ids: Vec<&str> = declared.keys().map(String::as_str).collect();
    assert_eq!(
        got, ids,
        "un aperçu par surface voyageur servie par le livret"
    );

    let mut ids_seen = Vec::new();
    let surfaces: Vec<Value> = rendered
        .into_iter()
        .map(|(surface_id, surface)| {
            let mut tree = serde_json::to_value(&surface.root).expect("arbre SDUI");
            stable_uuids(&mut tree, &mut ids_seen);
            let label_key = declared[surface_id]["label_key"]
                .as_str()
                .unwrap_or_default();
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

/// Les surfaces voyageur que le livret sert, par id : celles dont le `#[surface]` donne une `path`.
fn guest_routes(emissions: &Path) -> std::collections::BTreeMap<String, Value> {
    fs::read_dir(emissions)
        .expect("déclarations du module — compilé avec portaki-sdk-macros ?")
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("surface-guest_")
        })
        .map(|entry| read_json(&entry.path()))
        .filter(|surface| surface["catalog"]["path"].is_string())
        .map(|surface| {
            let id = surface["id"].as_str().unwrap_or_default().to_string();
            (id, surface["catalog"].clone())
        })
        .collect()
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

/// Remplace chaque UUID de l'arbre par un identifiant stable, numéroté dans l'ordre d'apparition.
///
/// Un module qui range ses lignes sous un `Uuid::new_v4` les expose dans ses actions ; sans ça,
/// l'aperçu changerait à chaque exécution.
fn stable_uuids(value: &mut Value, seen: &mut Vec<Uuid>) {
    match value {
        Value::String(text) => {
            let mut out = String::with_capacity(text.len());
            let mut rest = text.as_str();
            while !rest.is_empty() {
                match rest.get(..36).and_then(|head| Uuid::try_parse(head).ok()) {
                    Some(id) => {
                        let index = seen.iter().position(|s| *s == id).unwrap_or_else(|| {
                            seen.push(id);
                            seen.len() - 1
                        });
                        out.push_str(&Uuid::from_u128(index as u128 + 1).to_string());
                        rest = &rest[36..];
                    }
                    None => {
                        let next = rest.chars().next().expect("non vide");
                        out.push(next);
                        rest = &rest[next.len_utf8()..];
                    }
                }
            }
            *text = out;
        }
        Value::Array(items) => items.iter_mut().for_each(|item| stable_uuids(item, seen)),
        Value::Object(fields) => fields
            .values_mut()
            .for_each(|item| stable_uuids(item, seen)),
        _ => {}
    }
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
