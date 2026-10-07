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
const DEMO_FILE: &str = "demo.json";

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
        .with_property(Property {
            // Le logement de la démo du livret, et non le « Villa Azur » du SDK.
            //
            // Les aperçus sont servis par `/p/demo-guest`, dont le manifeste dit « L'Islette,
            // Cap d'Antibes ». Les modules y rendaient le nom et les coordonnées de la fixture du
            // SDK : la carte d'accès nommait son repère « Villa Azur » au milieu du livret de
            // L'Islette, l'avis demandait « votre séjour à Villa Azur », et le plan montrait
            // Cannes là où le bandeau annonçait Antibes.
            //
            // Ici et pas dans le SDK : `Property::default()` sert tous les tests des vingt-trois
            // modules, et le changer demanderait une release pour un nom de vitrine.
            name: "L'Islette".to_string(),
            // Le **nom** du livret, pas sa position. Les lieux d'exemple des modules sont posés
            // autour des coordonnées du SDK ; déplacer le logement sans eux mettait neuf
            // kilomètres entre les deux, et la fiche d'une boulangerie « du village » annonçait
            // « environ 108 min » à pied. Les distances se calculent, elles ne se déclarent pas.
            ..Property::default()
        })
        .with_stay(stay)
        .with_now(at(NOW))
}

/// L'instant des aperçus : la veille de l'arrivée.
pub const NOW: &str = "2026-05-31T10:00:00Z";

/// Le lendemain du départ — pour ce qu'un module ne montre qu'après.
///
/// `allow(dead_code)` parce que ce fichier est inclus par `#[path]` dans chaque module : ce dont
/// un seul se sert est mort pour les vingt autres. Pas d'équivalent « pendant le séjour » : le cas
/// qu'on croyait en avoir besoin, la checklist, ouvre à 48 h du départ, et « pas encore » est la
/// bonne réponse à mi-séjour.
#[allow(dead_code)]
pub const AFTER: &str = "2026-06-09T10:00:00Z";

/// Un contexte voyageur à une autre heure que celle des aperçus.
///
/// <p>La démo du livret sert un rendu figé, pris à [`NOW`], qui est d'avant une arrivée. Un module
/// dont le contenu dépend du moment — la checklist de départ — y disait donc « pas encore » quelle
/// que soit la phase demandée. Il publie désormais un rendu par phase, et le livret choisit.
#[allow(dead_code)]
pub fn guest_at(module_root: &str, instant: &str) -> portaki_test_utils::MockContextBuilder {
    guest(module_root).with_now(at(instant))
}

/// Un instant RFC 3339, en UTC.
pub fn at(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("date")
        .with_timezone(&Utc)
}

/// Les aperçus du catalogue **et** les surfaces de la démo, rendus une fois.
///
/// `rendered` compare `previews.json` : chaque surface voyageur que le livret sert — un
/// `#[surface(guest, path = …)]` — doit y être, et rien d'autre, sans quoi un aperçu mentirait
/// sur le module.
///
/// `extra` porte ce que cet invariant exclut par construction : les cartes d'accueil, qui n'ont
/// pas de chemin — et que la démo du livret montre en premier. L'invariant du catalogue reste
/// donc entier, et `demo.json` reçoit l'ensemble.
///
/// `emissions` est le dossier où les macros du module ont écrit ses déclarations en compilant :
/// `concat!(env!("OUT_DIR"), "/portaki-emissions")` depuis le test. Le manifeste n'est écrit que
/// par `portaki build`, qui ne tourne pas avant `cargo test`.
///
/// `markers` porte les points que le module pose sur la carte du livret (§3) — la réponse de sa
/// requête `mapMarkers`, rendue sous la même fixture que ses surfaces. `None` pour un module qui
/// n'en pose pas. La carte de la démo était vide : sa route renvoie une liste vide pour une clé
/// `demo-`, et le mécanisme central de §3 — « les modules poussent leurs lieux » — n'était exercé
/// nulle part dans la vitrine.
#[allow(dead_code)]
pub fn check_all(
    module_root: &str,
    emissions: &str,
    rendered: Vec<(&str, Surface)>,
    extra: Vec<(&str, Surface)>,
    markers: Option<Value>,
) {
    check_all_phased(module_root, emissions, rendered, extra, markers, Vec::new());
}

/// Comme [`check_all`], plus un jeu de surfaces par phase du séjour.
///
/// <p>`phases` nomme chaque jeu — `"stay"`, `"post-stay"` — et porte les mêmes surfaces rendues à
/// une autre heure (voir [`guest_at`]). Le livret de démonstration les sert quand sa clé demande
/// cette phase ; sans elles il montre le rendu de [`NOW`], d'avant l'arrivée.
#[allow(dead_code)]
pub fn check_all_phased(
    module_root: &str,
    emissions: &str,
    rendered: Vec<(&str, Surface)>,
    extra: Vec<(&str, Surface)>,
    markers: Option<Value>,
    phases: Vec<(&str, Vec<(&str, Surface)>)>,
) {
    let demo: Vec<(&str, Value)> = rendered
        .iter()
        .chain(extra.iter())
        .map(|(id, surface)| {
            (
                *id,
                serde_json::to_value(&surface.root).expect("arbre SDUI"),
            )
        })
        .collect();
    let phased: Vec<(&str, Vec<(&str, Value)>)> = phases
        .into_iter()
        .map(|(name, surfaces)| {
            let trees = surfaces
                .into_iter()
                .map(|(id, surface)| (id, serde_json::to_value(&surface.root).expect("arbre SDUI")))
                .collect();
            (name, trees)
        })
        .collect();
    check_demo(
        module_root,
        guest_routes(Path::new(emissions)),
        demo,
        markers,
        phased,
    );
    check_previews(module_root, emissions, rendered);
}

fn check_previews(module_root: &str, emissions: &str, rendered: Vec<(&str, Surface)>) {
    let root = Path::new(module_root);
    let bundle = fr_bundle(module_root);
    let declared = guest_routes(Path::new(emissions));

    // Un module dont aucune surface n'a de chemin n'entre pas au catalogue : pas de fichier
    // d'aperçus plutôt qu'un fichier vide, qui laisserait croire qu'il n'a rien à montrer.
    if declared.is_empty() && rendered.is_empty() {
        return;
    }

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
/// Les surfaces de la démo : **tout** ce que le module rend, cartes d'accueil comprises.
///
/// Pourquoi un second fichier à côté de `previews.json` : celui-là est tenu par un invariant —
/// un aperçu par surface *servie par un chemin*, et rien d'autre — qui exclut par construction
/// les cartes d'accueil. Or ce sont elles que la démo du livret montre en premier. Les y faire
/// entrer aurait affaibli l'invariant du catalogue pour servir un autre besoin.
///
/// `demo.json` n'a donc pas d'invariant de couverture : le module y met ce qu'il veut montrer. La
/// démo du livret (`portaki-guest`) le lit à la place des arbres qu'elle écrivait à la main, et
/// ces arbres étaient une troisième source de vérité qui dérivait sans que rien ne le dise.
///
/// Pour le régénérer :
///
/// ```sh
/// PORTAKI_UPDATE_DEMO=1 cargo test -p <module> --test previews
/// ```
fn check_demo(
    module_root: &str,
    routes: std::collections::BTreeMap<String, Value>,
    rendered: Vec<(&str, Value)>,
    markers: Option<Value>,
    phases: Vec<(&str, Vec<(&str, Value)>)>,
) {
    let root = Path::new(module_root);
    let bundle = fr_bundle(module_root);

    let mut ids_seen = Vec::new();
    let mut surfaces: Vec<Value> = rendered
        .into_iter()
        .map(|(surface_id, mut tree)| {
            stable_uuids(&mut tree, &mut ids_seen);
            let i18n: Map<String, Value> = i18n_refs(&tree)
                .into_iter()
                .filter_map(|key| bundle.get(&key).map(|value| (key, value.clone())))
                .collect();
            // Le chemin que le module déclare, quand il en déclare un. La démo du livret construit
            // ses routes avec, au lieu d'en tenir une liste à la main : celle-ci avait déjà dérivé
            // (`sections` y était annoncé sur une surface que le module n'a pas).
            let mut surface = json!({ "surfaceId": surface_id, "tree": tree, "i18n": i18n });
            if let Some(catalog) = routes.get(surface_id) {
                for (from, to) in [("path", "path"), ("label_key", "labelKey")] {
                    if let Some(value) = catalog.get(from).filter(|value| value.is_string()) {
                        surface[to] = value.clone();
                    }
                }
                // L'émission garde la forme Rust (`GuestRole::StatusCell`) ; c'est `portaki build`
                // qui la met sur le fil. La démo lit le même mot que la plateforme sert, sinon le
                // livret chercherait `status-cell` en face d'un nom de variante.
                if let Some(role) = catalog["role"].as_str().and_then(wire_role) {
                    surface["role"] = Value::String(role.to_string());
                }
            }
            surface
        })
        .collect();
    // Trié : le fichier ne doit pas bouger parce que le test a listé ses surfaces autrement.
    surfaces.sort_by(|a, b| a["surfaceId"].as_str().cmp(&b["surfaceId"].as_str()));

    let mut document = json!({ "locale": LOCALE, "surfaces": surfaces });
    if let Some(markers) = markers {
        document["markers"] = markers;
    }
    if !phases.is_empty() {
        let mut by_phase = Map::new();
        for (name, trees) in phases {
            let mut ids = Vec::new();
            let mut list: Vec<Value> = trees
                .into_iter()
                .map(|(surface_id, mut tree)| {
                    stable_uuids(&mut tree, &mut ids);
                    let i18n: Map<String, Value> = i18n_refs(&tree)
                        .into_iter()
                        .filter_map(|key| bundle.get(&key).map(|value| (key, value.clone())))
                        .collect();
                    json!({ "surfaceId": surface_id, "tree": tree, "i18n": i18n })
                })
                .collect();
            list.sort_by(|a, b| a["surfaceId"].as_str().cmp(&b["surfaceId"].as_str()));
            by_phase.insert(name.to_string(), Value::Array(list));
        }
        document["phases"] = Value::Object(by_phase);
    }
    let expected = serde_json::to_string_pretty(&document).expect("json") + "\n";
    let path = root.join(DEMO_FILE);
    if std::env::var_os("PORTAKI_UPDATE_DEMO").is_some() {
        fs::write(&path, &expected).expect("écrire demo.json");
        return;
    }
    let current = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == expected,
        "{DEMO_FILE} ne correspond plus au rendu — PORTAKI_UPDATE_DEMO=1 cargo test --test previews"
    );
}

/// `GuestRole::StatusCell` → `"status-cell"`, comme le fait `portaki build`.
fn wire_role(variant: &str) -> Option<&'static str> {
    let (vocabulary, variant) = variant.split_once("::")?;
    portaki_sdk::vocab::wire_of(vocabulary, variant)
}

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
