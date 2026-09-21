//! Section « Billets & activités (Tiqets) », contre une réponse enregistrée — jamais un appel réel.

use std::collections::BTreeSet;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::capability::{self, CapabilityId};
use portaki_sdk::host::with_host;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use serial_test::serial;

use local_guide::{
    get_config, render_explore_detail, render_home_card, render_host_main, render_upcoming_card,
    update_config, UpdateConfigArgs, FRESH_SECS, STALE_MAX_SECS,
};

const RECORDED: &str = include_str!("fixtures/tiqets-products-nearby.json");
const FR_BUNDLE: &str = include_str!("../i18n/fr-FR.json");
const EN_BUNDLE: &str = include_str!("../i18n/en-US.json");

/// Le lien du fixture : l'API Tiqets y a posé le code d'affiliation de la clé appelante.
const AFFILIATE_URL: &str = "https://www.tiqets.com/fr/attractions-amsterdam-c75061/billets-pour-musee-van-gogh-p974079/?partner=portaki-fixture";

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-09-21T10:00:00Z")
        .expect("date")
        .with_timezone(&Utc)
}

fn config(tiqets: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({ "tiqets": tiqets })).expect("config")
}

fn enabled() -> Vec<u8> {
    config(json!({ "enabled": true, "radius_km": 20 }))
}

fn surface_json(surface: &Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

fn guest(capabilities: &[CapabilityId]) -> MockContext {
    MockContext::guest()
        .with_capabilities(capabilities)
        .with_now(now())
}

fn pool() -> [CapabilityId; 2] {
    [capability::core::STORAGE, capability::external::TIQETS_POOL]
}

/// Une entrée de cache telle que le module l'écrit, datée de `age` avant [`now`].
fn cached(age: Duration, title: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "lat": 43.5513,
        "lng": 7.0128,
        "radius_km": 20,
        "min_rating": null,
        "lang": "fr",
        "fetched_at": (now() - age).timestamp(),
        "products": [{
            "id": "1",
            "title": title,
            "tagline": null,
            "city": null,
            "image": null,
            "price": null,
            "currency": null,
            "rating": null,
            "rating_count": 0,
            "distance_km": null,
            "product_url": "https://www.tiqets.com/fr/p1/?partner=portaki-fixture",
            "lat": null,
            "lng": null
        }]
    }))
    .expect("cache")
}

#[test]
#[serial]
fn the_detail_lists_the_recorded_products_with_their_affiliate_links() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx));

            assert!(json.contains("Musée Van Gogh : billet d'entrée"), "{json}");
            // Le lien de Tiqets, tel quel : il porte le code d'affiliation.
            assert!(json.contains(AFFILIATE_URL), "{json}");
            assert!(json.contains("fixture-medium.jpg"), "{json}");
            // Crédit de l'image, exigé par Tiqets là où l'image est montrée.
            assert!(json.contains("Photo : Fixture Studio"), "{json}");
            assert!(json.contains("Le plus grand ensemble"), "{json}");
            assert!(json.contains("i18n:guest.tiqets.book"));
            assert!(json.contains("i18n:guest.tiqets.attribution"));
            assert!(json.contains("i18n:guest.tiqets.disclosure"));
            // Hors saison : pas de billet à vendre, pas de ligne.
            assert!(!json.contains("Produit hors saison"), "{json}");

            let calls = host.connector_calls();
            assert_eq!(calls.len(), 1);
            let args: Value = serde_json::from_str(&calls[0].args_json).expect("args");
            assert_eq!(args["lat"], 43.5513);
            assert_eq!(args["lng"], 7.0128);
            assert_eq!(args["max_distance"], 20);
            assert_eq!(args["lang"], "fr");
            assert_eq!(args["currency"], "EUR");
            assert!(args.get("min_rating").is_none());
        });
}

#[test]
#[serial]
fn price_and_rating_read_in_the_guest_language() {
    guest(&pool())
        .with_translation("guest.tiqets.priceFrom", "Dès {price}")
        .with_translation("guest.tiqets.rating", "★ {rating} ({count} avis)")
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(json.contains("Dès 22 € · ★ 4,6 (18234 avis)"), "{json}");
        });
}

#[test]
#[serial]
fn the_home_card_is_a_preview_without_images() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run(|ctx| {
            let json = surface_json(&render_home_card(ctx));
            assert!(json.contains("Musée Van Gogh"), "{json}");
            assert!(json.contains(AFFILIATE_URL));
            assert!(!json.contains("fixture-medium.jpg"), "{json}");
        });
}

#[test]
#[serial]
fn the_section_is_off_until_the_host_turns_it_on() {
    guest(&pool())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(!json.contains("guest.tiqets.title"), "{json}");
            assert!(host.connector_calls().is_empty());
        });
}

#[test]
#[serial]
fn without_pool_or_own_key_nothing_is_called() {
    guest(&[capability::core::STORAGE])
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(!json.contains("guest.tiqets.title"), "{json}");
            assert!(host.connector_calls().is_empty());
        });
}

#[test]
#[serial]
fn the_hosts_own_key_is_enough() {
    guest(&[capability::core::STORAGE, capability::external::TIQETS_BYOK])
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run(|ctx| {
            assert!(surface_json(&render_explore_detail(ctx)).contains("Musée Van Gogh"));
        });
}

#[test]
#[serial]
fn a_property_without_position_is_never_searched() {
    let (mut ctx, host) = guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .build();
    ctx.property.lat = 0.0;
    ctx.property.lng = 0.0;
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_explore_detail(ctx.clone()));
        assert!(!json.contains("guest.tiqets.title"), "{json}");
    });
    assert!(backend.connector_calls().is_empty());
}

#[test]
#[serial]
fn a_fresh_cache_is_served_without_calling_tiqets() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_kv(
            "tiqets_cache.fr",
            cached(Duration::seconds(FRESH_SECS - 60), "Depuis le cache"),
        )
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(json.contains("Depuis le cache"), "{json}");
            assert!(host.connector_calls().is_empty());
        });
}

#[test]
#[serial]
fn a_day_old_cache_is_refreshed() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_kv(
            "tiqets_cache.fr",
            cached(Duration::seconds(FRESH_SECS + 60), "Depuis le cache"),
        )
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(json.contains("Musée Van Gogh"), "{json}");
            assert!(!json.contains("Depuis le cache"));
            assert_eq!(host.connector_calls().len(), 1);
        });
}

#[test]
#[serial]
fn when_tiqets_fails_a_stale_cache_under_fourteen_days_is_still_shown() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_kv(
            "tiqets_cache.fr",
            cached(Duration::days(3), "Gardé trois jours"),
        )
        .with_connector_error(
            "tiqets",
            "nearby_products",
            "connector_pool_quota_exhausted",
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(json.contains("Gardé trois jours"), "{json}");
        });
}

#[test]
#[serial]
fn a_cache_older_than_fourteen_days_is_never_shown() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_kv(
            "tiqets_cache.fr",
            cached(Duration::seconds(STALE_MAX_SECS + 60), "Périmé"),
        )
        .with_connector_error("tiqets", "nearby_products", "connector_egress_failed")
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(!json.contains("Périmé"), "{json}");
            assert!(!json.contains("guest.tiqets.title"), "{json}");
        });
}

#[test]
#[serial]
fn a_tiqets_failure_without_cache_still_renders_the_rest_of_the_booklet() {
    guest(&pool())
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" } }],
                "tiqets": { "enabled": true }
            }))
            .expect("config"),
        )
        .with_connector_error("tiqets", "nearby_products", "connector_egress_failed")
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx));
            assert!(json.contains("Plage"), "{json}");
            assert!(!json.contains("guest.tiqets.title"), "{json}");
            assert!(!json.contains("guest.error.title"), "{json}");
        });
}

#[test]
#[serial]
fn an_unsupported_booklet_language_asks_tiqets_in_english() {
    let (mut ctx, host) = guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .build();
    ctx.locale = "uk-UA".to_string();
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        render_explore_detail(ctx.clone());
    });
    let args: Value = serde_json::from_str(&backend.connector_calls()[0].args_json).expect("args");
    assert_eq!(args["lang"], "en");
}

#[test]
#[serial]
fn the_host_sheet_says_why_nothing_would_show() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", enabled())
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx));
            assert!(json.contains("i18n:host.section.tiqets"), "{json}");
            assert!(
                json.contains("i18n:host.tiqets.status.missingKey"),
                "{json}"
            );
        });
    MockContext::host()
        .with_capabilities(&pool())
        .with_kv("config", enabled())
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx));
            assert!(json.contains("i18n:host.tiqets.status.ready"), "{json}");
        });
    MockContext::host().with_capabilities(&pool()).run(|ctx| {
        let json = surface_json(&render_host_main(ctx));
        assert!(json.contains("i18n:host.tiqets.status.off"), "{json}");
    });
}

#[test]
#[serial]
fn update_config_saves_the_settings_and_drops_the_cache() {
    MockContext::host()
        .with_capabilities(&pool())
        .with_kv(
            "tiqets_cache.fr",
            cached(Duration::hours(1), "Ancien rayon"),
        )
        .run_with(|ctx, host| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    tiqets_enabled: Some(true),
                    tiqets_radius_km: "40".into(),
                    tiqets_min_rating: "4".into(),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("update");
            let saved = get_config(ctx.clone()).expect("config");
            assert!(saved.tiqets.enabled);
            assert_eq!(saved.tiqets.radius_km, 40);
            assert_eq!(saved.tiqets.min_rating, 4);
            assert!(portaki_sdk::host::kv::get("tiqets_cache.fr")
                .expect("kv")
                .is_none());
            let _ = host;
        });
}

#[test]
#[serial]
fn an_older_caller_leaves_the_tiqets_settings_alone() {
    MockContext::host()
        .with_capabilities(&pool())
        .with_kv(
            "config",
            config(json!({ "enabled": true, "radius_km": 5, "min_rating": 3 })),
        )
        .run(|ctx| {
            update_config(ctx.clone(), UpdateConfigArgs::default()).expect("update");
            let saved = get_config(ctx).expect("config");
            assert!(saved.tiqets.enabled);
            assert_eq!(saved.tiqets.radius_km, 5);
            assert_eq!(saved.tiqets.min_rating, 3);
        });
}

#[test]
#[serial]
fn the_upcoming_card_does_not_embed_the_tiqets_list() {
    guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run(|ctx| {
            let json = surface_json(&render_upcoming_card(ctx));
            assert!(!json.contains("Musée Van Gogh"), "{json}");
        });
}

/// Toutes les clés `i18n:` des surfaces Tiqets existent en fr et en en.
#[test]
#[serial]
fn every_tiqets_key_exists_in_both_bundles() {
    let mut refs = BTreeSet::new();
    let collect = |json: &str, refs: &mut BTreeSet<String>| {
        let mut rest = json;
        while let Some(position) = rest.find("i18n:") {
            rest = &rest[position + "i18n:".len()..];
            let end = rest.find('"').unwrap_or(rest.len());
            if end > 0 {
                refs.insert(rest[..end].to_string());
            }
        }
    };
    guest(&pool())
        .with_kv("config", enabled())
        .with_connector_response("tiqets", "nearby_products", RECORDED)
        .run(|ctx| {
            collect(
                &surface_json(&render_explore_detail(ctx.clone())),
                &mut refs,
            );
            collect(&surface_json(&render_home_card(ctx)), &mut refs);
        });
    for capabilities in [vec![capability::core::STORAGE], pool().to_vec()] {
        MockContext::host()
            .with_capabilities(&capabilities)
            .with_kv("config", enabled())
            .run(|ctx| collect(&surface_json(&render_host_main(ctx)), &mut refs));
    }
    assert!(refs.contains("guest.tiqets.disclosure"));

    let runtime_keys = [
        "guest.tiqets.priceFrom",
        "guest.tiqets.rating",
        "connector.tiqets.name",
        "capability.tiqets.purpose",
        "capability.tiqets.fallback",
        "capability.tiqets.byok.purpose",
        "capability.tiqets.byok.fallback",
        "host.tiqets.radius.5",
        "host.tiqets.radius.10",
        "host.tiqets.radius.20",
        "host.tiqets.radius.40",
        "host.tiqets.status.missingCoordinates",
    ];
    for raw in [FR_BUNDLE, EN_BUNDLE] {
        let bundle: Value = serde_json::from_str(raw).expect("bundle");
        for key in refs.iter().map(String::as_str).chain(runtime_keys) {
            assert!(bundle.get(key).is_some(), "clé manquante — {key}");
        }
    }
}
