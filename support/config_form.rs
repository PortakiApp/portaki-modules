//! Le formulaire hôte et la config déclarée parlent des mêmes clés.
//!
//! Depuis `#[portaki_sdk::config]`, `updateConfig` n'arrive plus au module : la plateforme le
//! traite, et refuse tout le formulaire (`config_field_unknown`) dès qu'un champ porte un nom
//! qu'aucun `#[field]` ne déclare. Un champ déclaré que le formulaire ne montre pas, l'hôte ne
//! peut plus le remplir. Ce test lit les deux côtés : les `name` des champs de la surface hôte
//! (`items.0.label` compte pour `items`, la liste que le dashboard reconstruit) et les clés que la
//! macro a écrites dans ses émissions.
//!
//! Inclus par `#[path]` depuis `modules/*/tests/*.rs` : un fichier de test, pas une crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use portaki_sdk::sdui::surface::Surface;
use serde_json::Value;

/// Les clés déclarées par `#[portaki_sdk::config]`, lues dans `emissions`
/// (`concat!(env!("OUT_DIR"), "/portaki-emissions")`).
pub fn declared_keys(emissions: &str) -> BTreeSet<String> {
    let config = fs::read_dir(Path::new(emissions))
        .expect("emissions dir")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("config-"))
        })
        .expect("a config-*.json emission: is #[portaki_sdk::config] on the struct?");
    let config: Value = serde_json::from_str(&fs::read_to_string(config).unwrap()).unwrap();
    config["fields"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|field| field["key"].as_str().map(str::to_string))
        .collect()
}

/// Les clés que le formulaire de `surface` envoie, une fois `list.N.field` replié sur `list`.
pub fn form_keys(surface: &Surface) -> BTreeSet<String> {
    fn walk(value: &Value, out: &mut BTreeSet<String>) {
        match value {
            Value::Object(object) => {
                if let (Some(_), Some(Value::String(name))) =
                    (object.get("type"), object.get("name"))
                {
                    out.insert(name.split('.').next().unwrap_or(name).to_string());
                }
                object.values().for_each(|v| walk(v, out));
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(&serde_json::to_value(surface).unwrap(), &mut out);
    out
}

/// Le formulaire envoie exactement les clés déclarées — `not_in_form` excepté, des champs que
/// l'hôte remplit ailleurs.
pub fn assert_form_matches_config(emissions: &str, surface: &Surface, not_in_form: &[&str]) {
    let declared = declared_keys(emissions);
    let form = form_keys(surface);
    let unknown: Vec<&String> = form.difference(&declared).collect();
    assert!(
        unknown.is_empty(),
        "the host form sends {unknown:?}, which #[portaki_sdk::config] does not declare: the \
         platform would refuse every save (config_field_unknown)"
    );
    let missing: Vec<&String> = declared
        .difference(&form)
        .filter(|key| !not_in_form.contains(&key.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "declared fields {missing:?} are not in the host form: the host cannot fill them"
    );
}
