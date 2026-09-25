//! Un enregistrement du formulaire hôte, rejoué comme la plateforme l'écrit.
//!
//! La plateforme ne remplace pas un texte traduit : elle écrit la chaîne reçue dans la langue de
//! l'hôte et garde les autres (`localized`), et fusionne chaque ligne d'une liste avec la ligne
//! stockée de même `id`, sinon de même position (`item`), en gardant les sous-clés que le
//! formulaire n'envoie pas. Ce qui la fait tenir est côté module : le champ déclaré `localized`,
//! le formulaire qui envoie une chaîne sous le nom stocké, l'id de chaque ligne. [`save`] rejoue
//! ce contrat sur la surface rendue, pour qu'un test dise « enregistrer en `en` ne perd pas le
//! `fr` » sans plateforme.
//!
//! Inclus par `#[path]` depuis `modules/*/tests/*.rs` : un fichier de test, pas une crate.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use portaki_sdk::sdui::surface::Surface;
use serde_json::{Map, Value};

/// `config.fields` de `#[portaki_sdk::config]`, `item` résolu depuis les `#[params]` des lignes —
/// ce que `portaki build` écrit dans le manifeste.
pub fn declared_fields(emissions: &str) -> Vec<Value> {
    let read = |prefix: &str| -> Vec<Value> {
        fs::read_dir(Path::new(emissions))
            .expect("emissions dir")
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
            })
            .map(|path| serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap())
            .collect()
    };
    let params = read("params-");
    let mut fields = read("config-").pop().expect("a config-*.json emission")["fields"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    portaki_sdk::config::resolve_items(&mut fields, |name| {
        params.iter().find(|shape| shape["name"] == name).cloned()
    });
    fields
}

/// Les textes traduits déclarés : `key` pour un champ `localized`, `key.sub` pour une sous-clé
/// d'`item.localized`.
pub fn localized_paths(emissions: &str) -> Vec<String> {
    let mut out = Vec::new();
    for field in declared_fields(emissions) {
        let key = field["key"].as_str().unwrap_or_default();
        if field["type"] == "localized" {
            out.push(key.to_string());
        }
        for sub in field["item"]["localized"].as_array().into_iter().flatten() {
            out.push(format!("{key}.{}", sub.as_str().unwrap_or_default()));
        }
    }
    out.sort();
    out
}

/// Ce que le formulaire de `surface` envoie, tel quel : `name` → `value` (ou `checked`), les
/// `list.N.sub` regroupés en lignes.
pub fn form_args(surface: &Surface) -> Map<String, Value> {
    fn walk(value: &Value, out: &mut Vec<(String, Value)>) {
        match value {
            Value::Object(object) => {
                if let Some(Value::String(name)) = object.get("name") {
                    if let Some(sent) = object.get("value").or_else(|| object.get("checked")) {
                        out.push((name.clone(), sent.clone()));
                    }
                }
                object.values().for_each(|v| walk(v, out));
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut sent = Vec::new();
    walk(&serde_json::to_value(surface).unwrap(), &mut sent);
    let mut flat = Map::new();
    let mut lists: BTreeMap<String, BTreeMap<usize, Map<String, Value>>> = BTreeMap::new();
    for (name, value) in sent {
        let parts: Vec<&str> = name.splitn(3, '.').collect();
        match parts.as_slice() {
            [list, index, sub] if index.parse::<usize>().is_ok() => {
                lists
                    .entry(list.to_string())
                    .or_default()
                    .entry(index.parse().unwrap())
                    .or_default()
                    .insert(sub.to_string(), value);
            }
            _ => {
                flat.insert(name, value);
            }
        }
    }
    for (list, rows) in lists {
        let rows = rows.into_values().map(Value::Object).collect();
        flat.insert(list, Value::Array(rows));
    }
    flat
}

/// `stored` après un enregistrement du formulaire de `surface` par un hôte qui parle `lang`.
pub fn save(emissions: &str, surface: &Surface, stored: &Value, lang: &str) -> Value {
    let fields = declared_fields(emissions);
    let mut out = stored.as_object().cloned().unwrap_or_default();
    for (key, value) in form_args(surface) {
        let field = fields
            .iter()
            .find(|f| f["key"] == key.as_str())
            .unwrap_or_else(|| panic!("the form sends `{key}`, which the config does not declare"));
        let merged = match field["type"].as_str().unwrap_or("text") {
            "localized" => localized(out.get(&key), &value, lang),
            "secret" if value.as_str().is_some_and(str::is_empty) => continue,
            "number" => match value.as_str().map(str::trim) {
                Some("") => Value::Null,
                Some(text) => serde_json::from_str(&text.replace(',', ".")).unwrap_or(value),
                None => value,
            },
            "structured" if field["item"].is_object() && value.is_array() => {
                rows(&field["item"], out.get(&key), &value, lang)
            }
            _ => value,
        };
        if merged.is_null() {
            out.remove(&key);
        } else {
            out.insert(key, merged);
        }
    }
    Value::Object(out)
}

fn localized(existing: Option<&Value>, value: &Value, lang: &str) -> Value {
    let Some(text) = value.as_str() else {
        return value.clone();
    };
    let mut out = match existing {
        Some(Value::Object(map)) => map.clone(),
        Some(Value::String(old)) if !old.trim().is_empty() => {
            Map::from_iter([("fr".to_string(), Value::from(old.as_str()))])
        }
        _ => Map::new(),
    };
    if text.is_empty() {
        out.remove(lang);
    } else {
        out.insert(lang.to_string(), Value::from(text));
    }
    Value::Object(out)
}

fn rows(item: &Value, existing: Option<&Value>, sent: &Value, lang: &str) -> Value {
    let previous = existing
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let id_key = item["id"].as_str().unwrap_or_default();
    let is_localized = |sub: &str| {
        item["localized"]
            .as_array()
            .is_some_and(|subs| subs.iter().any(|s| s == sub))
    };
    let blank = |v: &Value| match v {
        Value::String(s) => s.trim().is_empty(),
        Value::Null => true,
        _ => false,
    };
    let rows = sent
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row = row.as_object().unwrap();
            // Vidée par l'hôte : remplacée. Une ligne qui n'envoie que son id garde tout.
            if row.keys().any(|sub| sub != id_key)
                && row.iter().all(|(sub, v)| sub == id_key || blank(v))
            {
                return Value::Object(row.clone());
            }
            let by_id = row
                .get(id_key)
                .filter(|id| !blank(id))
                .and_then(|id| previous.iter().find(|p| p.get(id_key) == Some(id)));
            let mut out = by_id
                .or_else(|| previous.get(index))
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            for (sub, value) in row {
                let merged = if is_localized(sub) {
                    localized(out.get(sub), value, lang)
                } else {
                    value.clone()
                };
                out.insert(sub.clone(), merged);
            }
            Value::Object(out)
        });
    Value::Array(rows.collect())
}
