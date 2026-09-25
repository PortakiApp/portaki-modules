//! Les sept cas pathologiques de la sandbox, rejoués sur un module, et ses exemples de dispatch.
//!
//! La grille « Scénarios » de l'espace développeur rend chaque surface sur sept séjours qui
//! cassent les modules en production : sans e-mail, arrivée passée, une nuit, trois mois, sans
//! nom, sans photo, et le cas normal. `portaki_test_utils::scenarios` porte les mêmes fixtures ;
//! [`check_surfaces`] les joue sur **toutes** les surfaces que le module déclare, par le même shim
//! que le binaire wasm, et ajoute ce qu'un rendu `Ok` ne dit pas : un texte cassé (`None`,
//! `null`, un `{placeholder}` resté tel quel) ou une clé `i18n:` qu'une langue du module n'a pas.
//!
//! [`check_examples`] joue les `example(…)` des queries et commands — ce que l'onglet Exécuter
//! propose — et refuse une entrée que l'opération ne sait pas lire.
//!
//! Inclus par `#[path]` depuis `modules/*/tests/scenarios.rs` : un fichier de test, pas une crate.
//! Le test doit nommer la crate du module (`use my_module as _;` au moins) : une crate que le
//! binaire ne nomme pas n'est pas liée, et ses déclarations n'existent pas.

use std::collections::BTreeMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use portaki_sdk::wasm::registry::{self, HandlerKind};
use portaki_test_utils::scenarios::{self, Scenario};
use portaki_test_utils::MockContextBuilder;
use serde_json::{json, Map, Value};

/// Ce que le module attend de l'installation pour rendre autre chose qu'un état vide — sa config
/// d'exemple, ses capacités. Appliqué au contexte du cas, voyageur comme hôte.
pub type Setup = fn(MockContextBuilder) -> MockContextBuilder;

/// Rend chaque surface déclarée sur chaque cas, contexte préparé par `setup`.
///
/// Échoue une fois, en nommant chaque cas et chaque surface fautifs : erreur (même rendue en état
/// d'erreur par le shell voyageur), panique, texte cassé ou clé i18n manquante.
pub fn check_surfaces(module_root: &str, setup: Setup) {
    let bundles = bundles(module_root);
    let surfaces: Vec<_> = registry::declarations()
        .filter(|declaration| declaration.kind == HandlerKind::Surface)
        .collect();
    assert!(
        !surfaces.is_empty(),
        "aucune surface déclarée : le test nomme-t-il la crate du module ?"
    );
    scenarios::check_each(|scenario| {
        let failures: Vec<String> = surfaces
            .iter()
            .filter_map(|surface| {
                context_for(scenario, surface.context == "guest", setup)
                    .run_with(|ctx, host| {
                        let tree = (surface.dispatch)(ctx, json!({}));
                        // Le shell voyageur rend l'état d'erreur sur un `Err` : il ne reste
                        // que son journal pour le dire.
                        match host
                            .logs()
                            .into_iter()
                            .find(|l| l.message.ends_with("_render_failed"))
                        {
                            Some(line) => Err(line.fields["error"].to_string()),
                            None => tree.map_err(|error| error.to_string()),
                        }
                    })
                    .and_then(|tree| sound(&bundles, &tree))
                    .err()
                    .map(|error| format!("{} {}: {error}", surface.context, surface.name))
            })
            .collect();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    });
}

/// Le contexte d'un cas, voyageur ou hôte, préparé par `setup`.
pub fn context_for(scenario: &Scenario, guest: bool, setup: Setup) -> MockContextBuilder {
    setup(if guest {
        scenario.guest()
    } else {
        scenario.host()
    })
}

/// Opérations appelées par la plateforme, jamais par un développeur : pas d'exemple.
const PLATFORM_HOOKS: &[&str] = &["legacyConfig", "onConfigUpdated"];

/// Joue chaque `example(…)` des queries et commands sur le cas `normal`.
///
/// Une entrée que l'opération ne sait pas désérialiser, ou une panique, fait échouer. Une erreur
/// métier non : un exemple n'a pas les données du compte qui l'exécutera. `without` nomme les
/// opérations laissées sans exemple exprès ; toute autre en réclame un.
pub fn check_examples(emissions: &str, setup: Setup, without: &[&str]) {
    let normal = scenarios::get("normal");
    let mut problems = Vec::new();
    for declared in operations(Path::new(emissions)) {
        let name = declared["name"].as_str().unwrap_or_default();
        let examples = declared["examples"].as_array().cloned().unwrap_or_default();
        if examples.is_empty() {
            // `legacyConfig` est la reprise de config que `#[portaki_sdk::config]` génère ;
            // `onConfigUpdated`, le signal que la plateforme envoie après une sauvegarde.
            if !without.contains(&name) && !PLATFORM_HOOKS.contains(&name) {
                problems.push(format!("{name}: no example"));
            }
            continue;
        }
        let kind = match declared["kind"].as_str() {
            Some("query") => HandlerKind::Query,
            _ => HandlerKind::Command,
        };
        let Some(handler) = registry::declarations().find(|d| d.kind == kind && d.name == name)
        else {
            problems.push(format!("{name}: not linked"));
            continue;
        };
        let guest = declared["guest"].as_bool().unwrap_or(false);
        // Les champs que `#[params]` a émis : serde tait une clé inconnue, pas ce test.
        let fields: Option<Vec<String>> = declared["args"]
            .as_str()
            .and_then(registry::params_shape)
            .and_then(|shape| {
                let fields = shape["fields"].as_array()?.iter();
                Some(
                    fields
                        .filter_map(|f| f["name"].as_str().map(str::to_string))
                        .collect(),
                )
            });
        for example in examples {
            let label = example["label"].as_str().unwrap_or_default();
            let unknown: Vec<&String> = example["input"]
                .as_object()
                .into_iter()
                .flat_map(|input| input.keys())
                .filter(|key| fields.as_ref().is_some_and(|fields| !fields.contains(key)))
                .collect();
            if !unknown.is_empty() {
                problems.push(format!("{name} « {label} »: unknown field(s) {unknown:?}"));
            }
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                context_for(&normal, guest, setup)
                    .run(|ctx| (handler.dispatch)(ctx, example["input"].clone()))
            }));
            match outcome {
                Err(_) => problems.push(format!("{name} « {label} »: panicked")),
                Ok(Err(error)) if error.to_string().contains("wasm_params_invalid") => {
                    problems.push(format!("{name} « {label} »: {error}"))
                }
                Ok(_) => {}
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Les déclarations `query-*.json` et `command-*.json` que les macros ont écrites.
fn operations(emissions: &Path) -> Vec<Value> {
    fs::read_dir(emissions)
        .expect("déclarations du module — compilé avec portaki-sdk-macros ?")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with("query-") || name.starts_with("command-")
        })
        .map(|path| {
            serde_json::from_str(&fs::read_to_string(path).expect("lecture")).expect("json")
        })
        .collect()
}

/// Le rendu ne montre ni texte cassé ni clé i18n absente d'une langue du module.
fn sound(bundles: &BTreeMap<String, Map<String, Value>>, tree: &Value) -> Result<(), String> {
    let mut problems = Vec::new();
    let mut stack = vec![tree];
    while let Some(value) = stack.pop() {
        match value {
            Value::String(text) => {
                if let Some(key) = text.strip_prefix("i18n:") {
                    for (locale, texts) in bundles {
                        if !texts.contains_key(key) {
                            problems.push(format!("`{key}` missing from {locale}"));
                        }
                    }
                } else if let Some(reason) = broken(text) {
                    problems.push(format!("{reason} in {text:?}"));
                }
            }
            Value::Array(items) => stack.extend(items),
            Value::Object(fields) => stack.extend(fields.values()),
            _ => {}
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join(", "))
    }
}

/// Ce qu'un voyageur ne doit jamais lire : une valeur absente formatée telle quelle, ou un
/// gabarit que personne n'a rempli.
fn broken(text: &str) -> Option<&'static str> {
    if text
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| matches!(word, "None" | "null" | "undefined" | "NaN"))
    {
        return Some("absent value");
    }
    if text.contains("Some(") {
        return Some("debug-formatted option");
    }
    let placeholder = text.split('{').skip(1).any(|rest| {
        rest.split_once('}').is_some_and(|(name, _)| {
            !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    });
    placeholder.then_some("unfilled placeholder")
}

/// `i18n/<locale>.json` → ses textes.
fn bundles(module_root: &str) -> BTreeMap<String, Map<String, Value>> {
    fs::read_dir(Path::new(module_root).join("i18n"))
        .expect("i18n/ du module")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let locale = path.file_stem().unwrap().to_string_lossy().to_string();
            let texts = serde_json::from_str(&fs::read_to_string(&path).expect("lecture"))
                .expect("bundle JSON");
            (locale, texts)
        })
        .collect()
}
