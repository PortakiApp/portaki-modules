//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use emergency_contacts::{render_explore_detail, render_home_card, render_host_main};
use portaki_sdk::host::module::ModuleStatus;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "SAMU" }, "phone": "15" },
            { "label": "Pompiers", "phone": "18" }
        ],
        "host_visible_phone": "+33 6 12 34 56 78"
    })
}

/// Chaque rangée mène par un pictogramme (§2.11).
///
/// L'hôte n'en avait pas — la ligne qu'on cherche en premier était la seule sans repère — et un
/// contact sans catégorie non plus, ce qui donnait une liste en dents de scie.
#[test]
#[serial]
fn every_row_leads_with_a_glyph() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "contacts": [{ "label": "Pharmacie de garde", "phone": "3237" }]
        }))
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(json.contains("\"leading\":\"users\""), "{json}");
            assert!(json.contains("\"leading\":\"phone\""), "{json}");
        });
}

#[test]
#[serial]
fn home_card_still_calls_the_country_numbers_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("\"112\""), "{json}");
        });
}

#[test]
#[serial]
fn home_card_renders_contacts() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // L'appel est porté par la ligne, pas par un `Pressable` autour : c'est ce que la
            // maquette donne à chaque rangée, et un niveau de moins à lire.
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
            assert!(json.contains("tel:"));
        });
}

#[test]
#[serial]
fn detail_includes_emergency_banner() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Pompiers"));
            // Le 112 est une tuile, plus un lien en bas de page : on compose avant de lire.
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            assert!(
                json.contains("\"title\":\"112\""),
                "le 112 est là, en tuile"
            );
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&{
            // Sur plages : les heures ne sont dessinées que là.
            let mut config = sample_config();
            config["host_availability"] = json!("hours");
            config
        })
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("+33 6 12 34 56 78"));
            assert!(json.contains("Pompiers"));
        });
}

/// The SDK renders the inactive state; the surface itself is not called.
#[test]
#[serial]
fn an_inactive_module_shows_the_sdk_state() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_module_status(ModuleStatus {
            active: false,
            workspace_enabled: true,
            incomplete: false,
            requires_config: true,
            missing_required_keys: Vec::new(),
        })
        .run(|ctx| {
            let surface = portaki_sdk::guest_shell::render(ctx, "home.card", render_home_card);
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

/// Les lignes suivent ce que l'hôte a saisi, et « Ajouter » en demande une de plus.
///
/// Six emplacements figés gelaient la liste à six : l'hôte voyait quatre cartes vides quand il
/// avait saisi deux contacts, et ne pouvait pas en saisir un septième.
#[test]
#[serial]
fn the_form_draws_the_contacts_the_host_has() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("contacts.0.label"), "{json}");
            assert!(!json.contains("contacts.1.label"), "{json}");
            assert!(json.contains(r#""contacts_count":2"#), "{json}");
        });

    let rows: Vec<Value> = (0..emergency_contacts::MAX_CONTACTS)
        .map(|i| json!({ "label": format!("Contact {i}"), "phone": "18" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "contacts": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            let last = emergency_contacts::MAX_CONTACTS - 1;
            assert!(json.contains(&format!("contacts.{last}.label")), "{json}");
            assert!(
                !json.contains(&format!("contacts.{}.label", last + 1)),
                "{json}"
            );
            assert!(
                json.contains(&format!(
                    r#""contacts_count":{}"#,
                    emergency_contacts::MAX_CONTACTS
                )),
                "{json}"
            );
        });
}

/// A host writing in English: the French label and note stay, and so do the note, the category
/// and the id the form does not carry; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["contacts.label", "contacts.note"]
    );
    let stored = json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15",
              "note": { "fr": "Gratuit", "en": "Free" }, "category": "medical" },
            { "label": "", "phone": "" },
            { "id": "pompiers", "label": { "fr": "Pompiers" }, "phone": "18" }
        ],
        "host_visible_phone": "+33 6"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["contacts"][0]["id"], "samu");
            assert_eq!(sent["contacts"][0]["label"], "Ambulance");
            assert!(sent["contacts"][1].get("id").is_none());
            assert_eq!(sent["contacts"][2]["id"], "pompiers");
            // Trois lignes stockées, trois dessinées : plus d'emplacements vides en bout de liste.
            assert_eq!(sent["contacts"].as_array().unwrap().len(), 3);

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["contacts"][0], stored["contacts"][0]);
            assert_eq!(saved["contacts"][2]["label"]["fr"], "Pompiers");
        });
}

/// Le numéro de l'hôte vient de son profil quand il ne l'a pas saisi ici.
///
/// Le champ du module existait parce que la plateforme ne portait pas le téléphone de l'hôte ;
/// elle le porte depuis `ctx.host`. Un hôte qui a rempli son compte n'a plus à le retaper, et le
/// §2.16 ne lui demande que ses contacts, sa pharmacie et son hôpital.
#[test]
#[serial]
fn the_host_row_falls_back_to_the_platform_profile() {
    let config = json!({ "contacts": [{ "label": "Pompiers", "phone": "18" }] });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            assert!(json.contains("tel:+33612345678"), "{json}");
        });
}

/// Le numéro saisi ici gagne : c'est un choix délibéré, par exemple une ligne dédiée.
#[test]
#[serial]
fn a_number_the_host_typed_here_wins_over_the_profile() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33600000000".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            assert!(json.contains("+33 6 12 34 56 78"), "{json}");
            assert!(!json.contains("+33600000000"), "{json}");
        });
}

/// Sans contact ni numéro — ni ici, ni au profil — il reste le 112 (§2.16).
///
/// Ce module ne se tait jamais : « aucun contact hôte → tuiles pays seules, **jamais d'état
/// vide**, "composez le 112" minimum ». Les numéros d'urgence ne viennent pas de l'hôte, ils se
/// calculent du pays du logement — un hôte qui n'a rien rempli n'est pas une raison de laisser un
/// voyageur sans numéro.
#[test]
#[serial]
fn nothing_anywhere_still_shows_the_country_numbers() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("\"112\""), "{json}");
        });
}

/// Le profil seul suffit à ouvrir la carte : un logement sans contact saisi montre quand même
/// qui appeler.
#[test]
#[serial]
fn the_profile_alone_is_enough_to_show_the_card() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            assert!(serde_json::to_string(&surface)
                .expect("json")
                .contains("tel:+33612345678"));
        });
}

/// Les numéros d'urgence sont ceux du **logement**, pas de la langue du voyageur.
///
/// Le module lisait `ctx.locale`, qui est celle du lecteur dès qu'il a choisi une langue. Un
/// francophone en Espagne recevait donc le 15 et le 18 — qui ne sonnent nulle part là-bas — et un
/// anglophone en France perdait les deux. On ne compose pas un numéro mort en urgence.
#[test]
#[serial]
fn the_numbers_follow_the_property_not_the_reader() {
    for (property_locale, reader, expect_national) in
        [("fr-FR", "en-US", true), ("es-ES", "fr-FR", false)]
    {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_property(Property {
                locale: property_locale.into(),
                ..Property::default()
            })
            .run(|ctx| {
                let mut ctx = ctx;
                ctx.locale = reader.to_string();
                let json =
                    serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
                assert!(json.contains("\"112\""), "{json}");
                assert_eq!(
                    json.contains("\"15\""),
                    expect_national,
                    "{property_locale} lu en {reader} : {json}"
                );
                assert_eq!(json.contains("\"18\""), expect_national, "{json}");
            });
    }
}

/// La pharmacie et l'hôpital placés partent sur la Carte du livret ; sans position, rien.
#[test]
#[serial]
fn placed_health_places_reach_the_booklet_map() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "pharmacy": "Pharmacie du port",
            "pharmacy_lat": "43.58",
            "pharmacy_lng": 7.12,
            "hospital": "Hôpital d'Antibes"
        }))
        .run(|ctx| {
            let markers = emergency_contacts::map_markers(ctx)
                .expect("markers")
                .markers;
            let json = serde_json::to_string(&markers).unwrap();
            assert_eq!(markers.len(), 1, "{json}");
            assert!(json.contains("Pharmacie du port"), "{json}");
        });
}
