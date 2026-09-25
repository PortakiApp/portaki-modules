//! Integration-style unit tests with `portaki-test-utils`.

use std::collections::BTreeSet;

use portaki_sdk::capability;
use portaki_sdk::host::with_host;
use serial_test::serial;

use local_guide::{
    render_explore_detail, render_home_card, render_host_main, render_upcoming_card,
    MAX_CURATED_LINKS, PARTNER_ID, PARTNER_QUERY_PARAM,
};
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

const FR_BUNDLE: &str = include_str!("../i18n/fr-FR.json");
const EN_BUNDLE: &str = include_str!("../i18n/en-US.json");

/// Locales embarquées par le module. Toute assertion « dans chaque langue » part d'ici.
const BUNDLES: [(&str, &str); 2] = [("fr-FR", FR_BUNDLE), ("en-US", EN_BUNDLE)];

fn sample_config() -> serde_json::Value {
    json!({
        "spots": [{
            "id": "bike", "title": { "fr": "Holiday Bikes", "en": "Holiday Bikes" },
            "category": "Location vélos", "distance": "900 m", "tag": "1j offert",
            "url": "https://example.com",
            "detail": { "fr": "Vélos ville et électriques.", "en": "City and e-bikes." }
        }],
        "disclaimer": "Suggestions non partenaires"
    })
}

fn surface_json(surface: &Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

fn bundle(raw: &str) -> serde_json::Map<String, serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(raw)
        .expect("bundle json")
        .as_object()
        .expect("bundle object")
        .clone()
}

/// Toutes les clés `i18n:` référencées par une surface sérialisée.
fn i18n_refs(json: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    let mut rest = json;
    while let Some(position) = rest.find("i18n:") {
        rest = &rest[position + "i18n:".len()..];
        let end = rest.find('"').unwrap_or(rest.len());
        let key = &rest[..end];
        if !key.is_empty() {
            refs.insert(key.to_string());
        }
    }
    refs
}

#[test]
#[serial]
fn home_card_empty_without_config_or_address() {
    // Sans adresse géocodée, la section activités n'a pas de ville : le module retombe
    // sur son état vide historique.
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        assert!(
            SurfaceAssertions::new(&render_home_card(ctx.clone()).expect("surface"))
                .contains_type("EmptyState")
        );
    });
}

#[test]
#[serial]
fn home_card_renders_spots_with_pill() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            let json = surface_json(&surface);
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn upcoming_card_renders_spot_count() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // Compact card must not embed the full spot list.
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = surface_json(&surface);
            assert!(json.contains("upcoming.card"));
            assert!(json.contains("i18n:nav.local-guide"));
        });
}

#[test]
#[serial]
fn upcoming_card_empty_without_config_or_address() {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        assert!(
            SurfaceAssertions::new(&render_upcoming_card(ctx.clone()).expect("surface"))
                .contains_type("EmptyState")
        );
    });
}

#[test]
#[serial]
fn detail_includes_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
        });
}

// --- Carte -------------------------------------------------------------------------------

/// Config avec un spot situé et un spot sans position.
fn located_config() -> serde_json::Value {
    json!({
        "spots": [
            {
                "id": "plage", "title": { "fr": "Plage du Midi" },
                "address": "Plage du Midi, Cannes", "lat": 43.548, "lng": 7.005
            },
            { "id": "boulangerie", "title": { "fr": "Boulangerie" } }
        ]
    })
}

#[test]
#[serial]
fn the_detail_surface_maps_the_located_spots_and_the_property() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&located_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Map"));
            let json = surface_json(&surface);
            assert!(json.contains("Plage du Midi"), "{json}");
            // Le logement ferme la carte, avec le marqueur qui lui est propre.
            assert!(json.contains("\"property\""), "{json}");
            // Le spot sans position n'a pas de marqueur, mais reste dans la liste.
            assert!(json.contains("Boulangerie"), "{json}");
        });
}

/// Logement non géocodé : les lieux restent sur la carte, sans repère du logement ni repli.
#[test]
#[serial]
fn without_a_property_position_the_map_has_no_property_marker() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_coordinates(None)
        .with_config(&located_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Map"));
            let json = surface_json(&surface);
            assert!(json.contains("Plage du Midi"), "{json}");
            assert!(!json.contains("\"property\""), "{json}");
        });
}

#[test]
#[serial]
fn the_home_card_stays_a_list_without_a_map() {
    // La carte d'accueil est une vignette : la carte appartient à la feuille détaillée.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&located_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

#[test]
#[serial]
fn a_config_written_before_the_map_renders_no_map() {
    // Les installations existantes n'ont aucune coordonnée : elles gardent leur liste,
    // sans carte vide ni carte centrée sur l'Atlantique.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

#[test]
#[serial]
fn null_island_never_reaches_the_map() {
    // Ce que rend un formulaire dont les deux champs de position sont restés vides.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": [{ "id": "s1", "title": { "fr": "Plage" }, "lat": 0.0, "lng": 0.0 }]
        }))
        .run(|ctx| {
            assert!(
                !SurfaceAssertions::new(&render_explore_detail(ctx).expect("surface"))
                    .contains_type("Map")
            );
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": located_config()["spots"],
            "disclaimer": "Suggestions non partenaires",
            "activities_enabled": true,
            "activities_destination": "Antibes",
            "activities_intro": "Les incontournables",
            "activities": [{ "url": "https://gyg.me/aBcD12", "label": "Court" }],
            "tiqets_enabled": true,
            "tiqets_radius_km": "20",
            "tiqets_min_rating": "4"
        }))
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            let json = surface_json(&surface);
            assert!(json.contains("Plage du Midi, Cannes"), "{json}");
            assert!(json.contains("Les incontournables"), "{json}");
            // Pas de bouton Enregistrer : le tableau de bord enregistre à la saisie, et la
            // plateforme refuserait les arguments figés qu'il enverrait.
            assert!(!SurfaceAssertions::new(&surface).contains_type("Button"));
        });
}

/// A host writing in English: the French texts stay, and so do the url, the note and the ids the
/// form does not carry; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "activities.label",
            "activities_intro",
            "disclaimer",
            "spots.detail",
            "spots.note",
            "spots.title"
        ]
    );
    let stored = json!({
        "spots": [
            { "id": "bike", "title": { "fr": "Vélos", "en": "Bikes" }, "category": "Location",
              "url": "https://example.com", "note": { "fr": "Réservez", "en": "Book ahead" },
              "detail": { "fr": "Vélos électriques", "en": "E-bikes" } },
            { "title": "", "category": "", "distance": "", "tag": "", "detail": "", "address": "" },
            { "id": "beach", "title": { "fr": "Plage" }, "detail": { "fr": "Sable fin" },
              "address": "Bd du Midi, Cannes", "lat": 43.548, "lng": 7.005 }
        ],
        "disclaimer": { "fr": "Suggestions", "en": "Suggestions (en)" },
        "activities_enabled": true,
        "activities_intro": { "fr": "Les incontournables", "en": "Must-dos" },
        "activities": [
            { "id": "suquet", "url": "https://gyg.me/aBcD12", "label": { "fr": "Suquet", "en": "Old town" } }
        ],
        "tiqets_radius_km": "20"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["spots"][0]["id"], "bike");
            assert_eq!(sent["spots"][0]["title"], "Bikes");
            assert!(sent["spots"][1].get("id").is_none());
            assert_eq!(sent["spots"][2]["id"], "beach");
            assert_eq!(sent["spots"][2]["title"], "Plage");
            assert_eq!(sent["spots"].as_array().unwrap().len(), 6);
            assert_eq!(sent["activities"][0]["id"], "suquet");
            assert_eq!(sent["activities"][0]["label"], "Old town");
            assert!(sent["activities"][1].get("id").is_none());
            assert_eq!(sent["activities_intro"], "Must-dos");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            let bike = &saved["spots"][0];
            for key in ["title", "note", "detail", "url", "category"] {
                assert_eq!(bike[key], stored["spots"][0][key], "{key}");
            }
            assert_eq!(saved["spots"][2]["title"]["fr"], "Plage");
            assert_eq!(saved["spots"][2]["detail"]["fr"], "Sable fin");
            assert_eq!(
                saved["activities"][0]["label"],
                stored["activities"][0]["label"]
            );
            assert_eq!(saved["activities_intro"], stored["activities_intro"]);
            assert_eq!(saved["disclaimer"], stored["disclaimer"]);
        });
}

// --- Activités & billets -----------------------------------------------------------------

#[test]
#[serial]
fn the_destination_comes_from_the_property_city() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "activities_enabled": true }))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = surface_json(&surface);
            assert!(
                json.contains("https://www.getyourguide.com/s/?q=Cannes"),
                "{json}"
            );
            assert!(json.contains("i18n:guest.activities.title"));
        });
}

#[test]
#[serial]
fn the_host_destination_override_wins() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities_enabled": true, "activities_destination": "Antibes"
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("q=Antibes"), "{json}");
            assert!(!json.contains("q=Cannes"), "{json}");
        });
}

#[test]
#[serial]
fn a_bare_module_renders_no_activities_section() {
    // Opt-in : l'adresse du logement suffirait à bâtir la recherche, mais tant que
    // l'hôte n'a rien activé, aucun lien d'affiliation n'atteint le voyageur.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn a_config_without_the_activities_key_renders_no_section() {
    // Le cas des installations existantes : publier cette version ne doit pas les
    // transformer en vitrine d'affiliation.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(!json.contains("i18n:guest.activities.title"), "{json}");
            // Les bons plans, eux, s'affichent toujours.
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

#[test]
#[serial]
fn without_a_destination_the_section_is_absent() {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "activities_enabled": true }))
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        let surface = render_explore_detail(ctx.clone()).expect("surface");
        let json = surface_json(&surface);
        assert!(!json.contains("getyourguide"), "{json}");
        assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
    });
}

#[test]
#[serial]
fn a_disabled_section_disappears_and_leaves_the_spots() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": [{ "id": "s1", "title": { "fr": "Plage" } }],
            "activities_enabled": false, "activities_destination": "Antibes"
        }))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(!json.contains("i18n:guest.activities.title"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

#[test]
#[serial]
fn curated_links_render_normalized_capped_and_in_order() {
    let links: Vec<serde_json::Value> = (0..MAX_CURATED_LINKS + 2)
        .map(|index| json!({ "url": format!("https://www.getyourguide.com/tour-{index:02}") }))
        .collect();
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "activities_enabled": true, "activities": links }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            let first = json.find("tour-00").expect("first link");
            let last = json.find("tour-09").expect("last kept link");
            assert!(first < last, "l'ordre de l'hôte est conservé");
            // Le cap coupe au-delà de dix, sans réordonner ce qui reste.
            assert!(!json.contains("tour-10"), "{json}");
            assert!(!json.contains("tour-11"), "{json}");
        });
}

#[test]
#[serial]
fn every_link_reaching_the_guest_carries_our_partner_id() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities_enabled": true,
            "activities_destination": "Antibes",
            "activities": [
                // Un identifiant collé par l'hôte ne survit pas au rendu.
                { "url": "https://www.getyourguide.com/paris-l16/?partner_id=someone" },
                // La plateforme garde ce que l'hôte a saisi : un domaine étranger n'est
                // arrêté qu'au rendu.
                { "url": "https://viator.com/paris" },
                { "url": "https://gyg.me/aBcD12" }
            ]
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(!json.contains("someone"), "{json}");
            assert!(!json.contains("viator"), "{json}");
            assert!(json.contains(&format!(
                "https://www.getyourguide.com/s/?q=Antibes&{PARTNER_QUERY_PARAM}={PARTNER_ID}"
            )));
            assert!(json.contains(&format!(
                "https://www.getyourguide.com/paris-l16/?{PARTNER_QUERY_PARAM}={PARTNER_ID}"
            )));
            // Le lien court repart intact : son identifiant est déjà dans le chemin.
            assert!(json.contains("https://gyg.me/aBcD12"));
        });
}

#[test]
#[serial]
fn the_host_sheet_flags_a_url_it_would_refuse() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities": [{ "url": "https://viator.com/paris" }]
        }))
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx).expect("host main"));
            assert!(json.contains("i18n:host.activities.error.badUrl"), "{json}");
            assert!(json.contains("i18n:host.section.activities"));
        });
}

#[test]
#[serial]
fn the_search_label_names_the_destination() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_translation(
            "guest.activities.searchLabel",
            "Voir les activités à {destination}",
        )
        .with_config(&json!({
            "activities_enabled": true, "activities_destination": "Antibes"
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("Voir les activités à Antibes"), "{json}");
        });
}

#[test]
#[serial]
fn a_pasted_destination_url_becomes_the_link_and_names_its_place() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_translation(
            "guest.activities.searchLabel",
            "Voir les activités à {destination}",
        )
        .with_config(&json!({
            "activities_enabled": true,
            // L'hôte a collé la page de destination : elle est juste quel que soit
            // le pays depuis lequel le voyageur se connecte, là où `?q=` ne l'est pas.
            "activities_destination": "https://www.getyourguide.com/cannes-l15/"
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(
                json.contains("https://www.getyourguide.com/cannes-l15/"),
                "{json}"
            );
            assert!(!json.contains("/s/?q="), "{json}");
            assert!(json.contains("Voir les activités à Cannes"), "{json}");
        });
}

#[test]
#[serial]
fn a_destination_url_trades_the_hosts_partner_id_for_ours() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities_enabled": true,
            "activities_destination": "https://www.getyourguide.com/cannes-l15/?partner_id=someone"
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(
                json.contains(&format!(
                    "https://www.getyourguide.com/cannes-l15/?{PARTNER_QUERY_PARAM}={PARTNER_ID}"
                )),
                "{json}"
            );
            assert!(!json.contains("someone"), "{json}");
        });
}

#[test]
#[serial]
fn a_short_destination_link_is_kept_and_labelled_from_the_address() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_translation(
            "guest.activities.searchLabel",
            "Voir les activités à {destination}",
        )
        .with_config(&json!({
            "activities_enabled": true, "activities_destination": "https://gyg.me/aBcD12"
        }))
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            // Le lien court repart intact : son identifiant est déjà dans le chemin.
            assert!(json.contains("https://gyg.me/aBcD12"), "{json}");
            assert!(!json.contains(PARTNER_QUERY_PARAM), "{json}");
            // Aucun slug à lire : la ville de l'adresse nomme le bouton.
            assert!(json.contains("Voir les activités à Cannes"), "{json}");
        });
}

#[test]
#[serial]
fn a_short_destination_link_without_an_address_gets_the_neutral_label() {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities_enabled": true, "activities_destination": "https://gyg.me/aBcD12"
        }))
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_explore_detail(ctx.clone()).expect("surface"));
        assert!(json.contains("https://gyg.me/aBcD12"), "{json}");
        assert!(json.contains("i18n:guest.activities.browseLabel"), "{json}");
    });
}

#[test]
#[serial]
fn the_host_sheet_flags_a_destination_url_it_would_refuse() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "activities_destination": "https://viator.com/paris"
        }))
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx).expect("host main"));
            assert!(json.contains("i18n:host.activities.error.badUrl"), "{json}");
        });
}

#[test]
#[serial]
fn the_affiliate_disclosure_ships_in_every_locale() {
    for (locale, raw) in BUNDLES {
        let text = bundle(raw)
            .get("guest.activities.disclosure")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        assert!(!text.trim().is_empty(), "{locale}: mention absente");
        assert!(
            text.to_lowercase().contains("commission"),
            "{locale}: la mention doit nommer la commission — {text}"
        );
    }

    // Et elle est bien rendue, sous la liste, dès que des liens s'affichent.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "activities_enabled": true }))
        .run(|ctx| {
            for json in [
                surface_json(&render_explore_detail(ctx.clone()).expect("surface")),
                surface_json(&render_home_card(ctx.clone()).expect("surface")),
            ] {
                assert!(
                    json.contains("i18n:guest.activities.disclosure"),
                    "mention absente d'une surface — {json}"
                );
            }
        });
}

#[test]
#[serial]
fn every_i18n_key_a_surface_uses_exists_in_every_locale() {
    let mut refs = BTreeSet::new();

    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": [{ "id": "s1", "title": { "fr": "Plage" }, "url": "https://example.com" }],
            "disclaimer": "Suggestions non partenaires",
            "activities_enabled": true,
            "activities": [{ "url": "https://gyg.me/aBcD12" }]
        }))
        .run(|ctx| {
            refs.extend(i18n_refs(&surface_json(
                &render_home_card(ctx.clone()).expect("surface"),
            )));
            refs.extend(i18n_refs(&surface_json(
                &render_explore_detail(ctx.clone()).expect("surface"),
            )));
            refs.extend(i18n_refs(&surface_json(
                &render_upcoming_card(ctx).expect("surface"),
            )));
        });

    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            refs.extend(i18n_refs(&surface_json(
                &render_host_main(ctx).expect("host main"),
            )));
        });

    assert!(refs.contains("guest.activities.disclosure"));
    assert!(refs.contains("host.section.activities"));

    // Les clés passées par `t!` sont résolues avant sérialisation : elles ne portent pas
    // le préfixe `i18n:` et n'apparaissent donc pas dans `refs`.
    let translated_at_runtime = ["guest.activities.searchLabel", "guest.upcoming.spotCount"];

    for (locale, raw) in BUNDLES {
        let keys = bundle(raw);
        for key in refs.iter().map(String::as_str).chain(translated_at_runtime) {
            assert!(keys.contains_key(key), "{locale}: clé manquante — {key}");
        }
    }

    // Les deux bundles déclarent exactement le même jeu de clés.
    let fr: BTreeSet<String> = bundle(FR_BUNDLE).keys().cloned().collect();
    let en: BTreeSet<String> = bundle(EN_BUNDLE).keys().cloned().collect();
    assert_eq!(
        fr.difference(&en).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "clés présentes en fr-FR seulement"
    );
    assert_eq!(
        en.difference(&fr).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "clés présentes en en-US seulement"
    );
}
