//! Integration-style unit tests with `portaki-test-utils`.

use std::collections::BTreeSet;

use portaki_sdk::capability;
use portaki_sdk::host::with_host;
use serial_test::serial;

use local_guide::{
    get_config, render_explore_detail, render_home_card, render_host_main, render_upcoming_card,
    update_config, ActivityInput, SpotInput, UpdateConfigArgs, ERR_ACTIVITIES_TOO_MANY,
    ERR_ACTIVITY_URL_NOT_GYG, MAX_CURATED_LINKS, PARTNER_ID, PARTNER_QUERY_PARAM,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

const FR_BUNDLE: &str = include_str!("../i18n/fr-FR.json");
const EN_BUNDLE: &str = include_str!("../i18n/en-US.json");

/// Locales embarquées par le module. Toute assertion « dans chaque langue » part d'ici.
const BUNDLES: [(&str, &str); 2] = [("fr-FR", FR_BUNDLE), ("en-US", EN_BUNDLE)];

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "spots_json": r#"[{"id":"bike","title":{"fr":"Holiday Bikes","en":"Holiday Bikes"},"category":"Location vélos","distance":"900 m","tag":"1j offert","url":"https://example.com","detail":{"fr":"Vélos ville et électriques.","en":"City and e-bikes."}}]"#,
        "disclaimer": "Suggestions non partenaires"
    }))
    .expect("config json")
}

fn config_bytes(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).expect("config json")
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

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Pill(_) if type_name == "Pill" => true,
            Component::Pressable(_) if type_name == "Pressable" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Link(_) if type_name == "Link" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Map(_) if type_name == "Map" => true,
            _ => false,
        };
        if matches {
            return true;
        }
        for child in child_components(node) {
            if walk(child, type_name) {
                return true;
            }
        }
        false
    }
    walk(&surface.root, type_name)
}

fn child_components(node: &Component) -> Vec<&Component> {
    match node {
        Component::Stack(inner) => inner.children.iter().collect(),
        Component::Card(inner) => inner.children.iter().collect(),
        Component::ListItem(inner) => inner.children.iter().collect(),
        Component::Pressable(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
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
        assert!(contains_component_type(
            &render_home_card(ctx.clone()),
            "EmptyState"
        ));
    });
}

#[test]
#[serial]
fn home_card_renders_spots_with_pill() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Pill"));
            let json = surface_json(&surface);
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn upcoming_card_renders_spot_count() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            // Compact card must not embed the full spot list.
            assert!(!contains_component_type(&surface, "ListItem"));
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
        assert!(contains_component_type(
            &render_upcoming_card(ctx.clone()),
            "EmptyState"
        ));
    });
}

#[test]
#[serial]
fn detail_includes_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Link"));
            assert!(contains_component_type(&surface, "InfoBanner"));
        });
}

// --- Carte -------------------------------------------------------------------------------

/// Config avec un spot situé et un spot sans position.
fn located_config_bytes() -> Vec<u8> {
    config_bytes(json!({
        "spots": [
            {
                "id": "plage", "title": { "fr": "Plage du Midi" },
                "address": "Plage du Midi, Cannes", "lat": 43.548, "lng": 7.005
            },
            { "id": "boulangerie", "title": { "fr": "Boulangerie" } }
        ]
    }))
}

#[test]
#[serial]
fn the_detail_surface_maps_the_located_spots_and_the_property() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", located_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Map"));
            let json = surface_json(&surface);
            assert!(json.contains("Plage du Midi"), "{json}");
            // Le logement ferme la carte, avec le marqueur qui lui est propre.
            assert!(json.contains("\"property\""), "{json}");
            // Le spot sans position n'a pas de marqueur, mais reste dans la liste.
            assert!(json.contains("Boulangerie"), "{json}");
        });
}

#[test]
#[serial]
fn the_home_card_stays_a_list_without_a_map() {
    // La carte d'accueil est une vignette : la carte appartient à la feuille détaillée.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", located_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(!contains_component_type(&surface, "Map"));
            assert!(contains_component_type(&surface, "ListItem"));
        });
}

#[test]
#[serial]
fn a_config_written_before_the_map_renders_no_map() {
    // Les installations existantes n'ont aucune coordonnée : elles gardent leur liste,
    // sans carte vide ni carte centrée sur l'Atlantique.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(!contains_component_type(&surface, "Map"));
            assert!(contains_component_type(&surface, "ListItem"));
        });
}

#[test]
#[serial]
fn null_island_never_reaches_the_map() {
    // Ce que rend un formulaire dont les deux champs de position sont restés vides.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" }, "lat": 0.0, "lng": 0.0 }]
            })),
        )
        .run(|ctx| {
            assert!(!contains_component_type(&render_explore_detail(ctx), "Map"));
        });
}

#[test]
#[serial]
fn the_picker_fields_save_the_position() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    spots: vec![SpotInput {
                        name: "Plage du Midi".into(),
                        address: "  Plage du Midi, Cannes  ".into(),
                        lat: Some(43.548),
                        lng: Some(7.005),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )
            .expect("enregistré");

            let config = get_config(ctx).expect("cfg");
            assert_eq!(config.spots[0].coords(), Some((43.548, 7.005)));
            assert_eq!(
                config.spots[0].address.as_deref(),
                Some("Plage du Midi, Cannes")
            );
        });
}

#[test]
#[serial]
fn saving_without_the_map_fields_keeps_the_stored_position() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", located_config_bytes())
        .run(|ctx| {
            // Un appelant plus ancien que la carte soumet le nom et rien d'autre : la
            // position enregistrée doit lui survivre.
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    spots: vec![SpotInput {
                        name: "Plage du Midi".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )
            .expect("enregistré");

            let config = get_config(ctx).expect("cfg");
            assert_eq!(config.spots[0].coords(), Some((43.548, 7.005)));
            assert_eq!(
                config.spots[0].address.as_deref(),
                Some("Plage du Midi, Cannes")
            );
        });
}

#[test]
#[serial]
fn update_config_roundtrip() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    disclaimer: "d".into(),
                    ..Default::default()
                },
            )
            .expect("ok");
            assert_eq!(get_config(ctx).expect("cfg").disclaimer.get("fr"), "d");
        });
}

// --- Activités & billets -----------------------------------------------------------------

#[test]
#[serial]
fn the_destination_comes_from_the_property_city() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({ "activities": { "enabled": true } })),
        )
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "enabled": true, "destination": "Antibes" }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
            let surface = render_explore_detail(ctx);
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(contains_component_type(&surface, "EmptyState"));
        });
}

#[test]
#[serial]
fn a_config_without_the_activities_key_renders_no_section() {
    // Le cas des installations existantes : publier cette version ne doit pas les
    // transformer en vitrine d'affiliation.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(!json.contains("i18n:guest.activities.title"), "{json}");
            // Les bons plans, eux, s'affichent toujours.
            assert!(contains_component_type(&surface, "ListItem"));
        });
}

#[test]
#[serial]
fn without_a_destination_the_section_is_absent() {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({ "activities": { "enabled": true } })),
        )
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        let surface = render_explore_detail(ctx.clone());
        let json = surface_json(&surface);
        assert!(!json.contains("getyourguide"), "{json}");
        assert!(contains_component_type(&surface, "EmptyState"));
    });
}

#[test]
#[serial]
fn a_disabled_section_disappears_and_leaves_the_spots() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" } }],
                "activities": { "enabled": false, "destination": "Antibes" }
            })),
        )
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            let json = surface_json(&surface);
            assert!(!json.contains("getyourguide"), "{json}");
            assert!(!json.contains("i18n:guest.activities.title"));
            assert!(contains_component_type(&surface, "ListItem"));
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
        .with_kv(
            "config",
            config_bytes(json!({ "activities": { "enabled": true, "links": links } })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": {
                    "enabled": true,
                    "destination": "Antibes",
                    "links": [
                        // Un identifiant collé par l'hôte ne survit pas au rendu.
                        { "url": "https://www.getyourguide.com/paris-l16/?partner_id=someone" },
                        { "url": "https://gyg.me/aBcD12" }
                    ]
                }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(!json.contains("someone"), "{json}");
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
fn a_foreign_url_is_refused_at_save() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let error = update_config(
                ctx,
                UpdateConfigArgs {
                    activities: vec![ActivityInput {
                        url: "https://viator.com/paris".into(),
                        label: "Pas GetYourGuide".into(),
                    }],
                    ..Default::default()
                },
            )
            .expect_err("refusé");
            assert!(
                error.to_string().contains(ERR_ACTIVITY_URL_NOT_GYG),
                "{error}"
            );
        });
}

#[test]
#[serial]
fn more_than_ten_links_is_refused_at_save() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let activities = (0..MAX_CURATED_LINKS + 1)
                .map(|index| ActivityInput {
                    url: format!("https://www.getyourguide.com/tour-{index:02}"),
                    label: String::new(),
                })
                .collect();
            let error = update_config(
                ctx,
                UpdateConfigArgs {
                    activities,
                    ..Default::default()
                },
            )
            .expect_err("refusé");
            assert!(
                error.to_string().contains(ERR_ACTIVITIES_TOO_MANY),
                "{error}"
            );
        });
}

#[test]
#[serial]
fn saving_normalizes_the_links_and_keeps_the_order() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    activities_enabled: Some(true),
                    activities_destination: "  Antibes  ".into(),
                    activities_intro: " Les incontournables ".into(),
                    activities: vec![
                        ActivityInput {
                            url: "www.getyourguide.com/paris-l16/?partner_id=ancien&lc=fr".into(),
                            label: " Paris ".into(),
                        },
                        // Créneau laissé vide : ignoré, ce n'est pas une faute.
                        ActivityInput::default(),
                        ActivityInput {
                            url: "https://gyg.me/aBcD12".into(),
                            label: "Court".into(),
                        },
                    ],
                    ..Default::default()
                },
            )
            .expect("enregistré");

            let config = get_config(ctx).expect("cfg");
            let activities = config.activities;
            assert!(activities.enabled);
            assert_eq!(activities.destination, "Antibes");
            assert_eq!(activities.intro.get("fr"), "Les incontournables");
            assert_eq!(activities.links.len(), 2);
            // Relevé en https, identifiant périmé retiré, reste de la query intact.
            assert_eq!(
                activities.links[0].url,
                "https://www.getyourguide.com/paris-l16/?lc=fr&partner_id=CLOQ42U"
            );
            assert_eq!(activities.links[0].label.get("fr"), "Paris");
            assert_eq!(activities.links[1].url, "https://gyg.me/aBcD12");
        });
}

#[test]
#[serial]
fn the_host_sheet_flags_a_url_it_would_refuse() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "links": [{ "url": "https://viator.com/paris" }] }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "enabled": true, "destination": "Antibes" }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": {
                    "enabled": true,
                    // L'hôte a collé la page de destination : elle est juste quel que soit
                    // le pays depuis lequel le voyageur se connecte, là où `?q=` ne l'est pas.
                    "destination": "https://www.getyourguide.com/cannes-l15/"
                }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": {
                    "enabled": true,
                    "destination": "https://www.getyourguide.com/cannes-l15/?partner_id=someone"
                }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "enabled": true, "destination": "https://gyg.me/aBcD12" }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "enabled": true, "destination": "https://gyg.me/aBcD12" }
            })),
        )
        .build();
    ctx.property.address = None;
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_explore_detail(ctx.clone()));
        assert!(json.contains("https://gyg.me/aBcD12"), "{json}");
        assert!(json.contains("i18n:guest.activities.browseLabel"), "{json}");
    });
}

#[test]
#[serial]
fn a_foreign_destination_url_is_refused_at_save() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let error = update_config(
                ctx,
                UpdateConfigArgs {
                    activities_destination: "https://viator.com/paris".into(),
                    ..Default::default()
                },
            )
            .expect_err("refusé");
            // Même faute que dans la liste, donc même erreur et même message.
            assert!(
                error.to_string().contains(ERR_ACTIVITY_URL_NOT_GYG),
                "{error}"
            );
        });
}

#[test]
#[serial]
fn saving_a_destination_url_normalizes_it_and_leaves_a_place_name_alone() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    activities_enabled: Some(true),
                    activities_destination:
                        "  www.getyourguide.com/cannes-l15/?partner_id=ancien  ".into(),
                    ..Default::default()
                },
            )
            .expect("enregistré");
            assert_eq!(
                get_config(ctx.clone()).expect("cfg").activities.destination,
                "https://www.getyourguide.com/cannes-l15/?partner_id=CLOQ42U"
            );

            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    activities_enabled: Some(true),
                    activities_destination: "  Antibes  ".into(),
                    ..Default::default()
                },
            )
            .expect("enregistré");
            assert_eq!(
                get_config(ctx).expect("cfg").activities.destination,
                "Antibes"
            );
        });
}

#[test]
#[serial]
fn the_host_sheet_flags_a_destination_url_it_would_refuse() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "activities": { "destination": "https://viator.com/paris" }
            })),
        )
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx));
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
        .with_kv(
            "config",
            config_bytes(json!({ "activities": { "enabled": true } })),
        )
        .run(|ctx| {
            for json in [
                surface_json(&render_explore_detail(ctx.clone())),
                surface_json(&render_home_card(ctx.clone())),
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
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" }, "url": "https://example.com" }],
                "disclaimer": "Suggestions non partenaires",
                "activities": {
                    "enabled": true,
                    "links": [{ "url": "https://gyg.me/aBcD12" }]
                }
            })),
        )
        .run(|ctx| {
            refs.extend(i18n_refs(&surface_json(&render_home_card(ctx.clone()))));
            refs.extend(i18n_refs(&surface_json(&render_explore_detail(
                ctx.clone(),
            ))));
            refs.extend(i18n_refs(&surface_json(&render_upcoming_card(ctx))));
        });

    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            refs.extend(i18n_refs(&surface_json(&render_host_main(ctx))));
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
