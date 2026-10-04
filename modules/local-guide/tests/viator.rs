//! Section « Activités (Viator) », contre une réponse enregistrée — jamais un appel réel.
//!
//! Le fixture est une recherche libre réelle du bac à sable Viator (Antibes, en français,
//! note ≥ 4), réduite à quatre produits.

use std::collections::BTreeSet;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::capability::{self, CapabilityId};
use portaki_sdk::host::{with_host, HostBackend};
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use serial_test::serial;

use local_guide::{
    render_explore_detail, render_home_card, render_host_main, VIATOR_FRESH_SECS,
    VIATOR_STALE_MAX_SECS,
};

const RECORDED: &str = include_str!("fixtures/viator-freetext.json");
const FR_BUNDLE: &str = include_str!("../i18n/fr-FR.json");
const EN_BUNDLE: &str = include_str!("../i18n/en-US.json");

/// Le lien du fixture : Viator y a posé les paramètres d'affiliation de la clé appelante.
const AFFILIATE_URL: &str = "https://shop.live.rc.viator.com/fr-FR/tours/Antibes/Antibes-2-hour-walking-tour-in-the-old-town/d21941-479450P1?mcid=42383&pid=P00322642&medium=api&api_version=2.0";

/// La variante de l'image de couverture la plus proche de 720 px, pour le premier produit.
const COVER_IMAGE: &str =
    "https://hare-media-cdn.tripadvisor.com/media/attractions-splice-spp-720x480/10/24/33/0b.jpg";

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-09-21T10:00:00Z")
        .expect("date")
        .with_timezone(&Utc)
}

/// Ce qu'enregistre le formulaire hôte : le sélecteur envoie son choix en texte.
fn enabled() -> Value {
    json!({ "viator_enabled": true, "viator_min_rating": "4" })
}

fn surface_json(surface: &Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

fn storage() -> [CapabilityId; 1] {
    [capability::core::STORAGE]
}

fn guest() -> MockContext {
    MockContext::guest()
        .with_capabilities(&storage())
        .with_now(now())
}

/// Une entrée de cache telle que le module l'écrit, datée de `age` avant [`now`]. Le contexte
/// de test est à « Cannes, France ».
/// Une entrée de cache **telle que la version précédente l'écrivait** : sans `gallery` ni
/// `description`. Elle doit rester lisible, sinon chaque livret rappellerait Viator pour rien.
fn cached(age: Duration, title: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "search_term": "Cannes",
        "min_rating": 4,
        "lang": "fr",
        "fetched_at": (now() - age).timestamp(),
        "products": [{
            "code": "1",
            "title": title,
            "image_url": null,
            "price": null,
            "currency": null,
            "rating": null,
            "rating_count": 0,
            "duration_minutes": null,
            "free_cancellation": false,
            "machine_translated": false,
            "product_url": "https://www.viator.com/tours/p1?pid=P1"
        }]
    }))
    .expect("cache")
}

#[test]
#[serial]
fn the_detail_lists_the_recorded_products_with_their_affiliate_links() {
    guest()
        .with_translation("guest.viator.priceFrom", "À partir de {price}")
        .with_translation("guest.viator.rating", "★ {rating} ({count} avis)")
        .with_config(&enabled())
        .with_connector_response("viator", "search_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));

            assert!(json.contains("Le Petit Train d'Antibes"), "{json}");
            // Le lien de Viator, tel quel : il porte les paramètres d'affiliation.
            assert!(json.contains(AFFILIATE_URL), "{json}");
            assert!(json.contains(COVER_IMAGE), "{json}");
            assert!(
                json.contains("À partir de 13 € · ★ 4,1 (48 avis) · 45 min"),
                "{json}"
            );
            assert!(json.contains("À partir de 25 € · ★ 5,0 (63 avis) · 2 h"));
            assert!(json.contains("i18n:guest.viator.freeCancellation"));
            assert!(json.contains("i18n:guest.viator.machineTranslated"));
            assert!(json.contains("i18n:guest.viator.book"));
            assert!(json.contains("i18n:guest.viator.attribution"));
            assert!(json.contains("i18n:guest.viator.disclosure"));

            let calls = host.connector_calls();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].connector_id, "viator");
            assert_eq!(calls[0].operation, "search_products");
            let args: Value = serde_json::from_str(&calls[0].args_json).expect("args");
            // La ville de l'adresse du logement, comme la section GetYourGuide.
            assert_eq!(args["searchTerm"], "Cannes");
            assert_eq!(args["lang"], "fr");
            assert_eq!(args["currency"], "EUR");
            assert_eq!(args["searchTypes"][0]["pagination"]["count"], 12);
            assert_eq!(args["productFiltering"]["rating"]["from"], 4);
        });
}

#[test]
#[serial]
fn the_home_card_is_a_preview_of_three_with_thumbnails() {
    guest()
        .with_config(&enabled())
        .with_connector_response("viator", "search_products", RECORDED)
        .run(|ctx| {
            let json = surface_json(&render_home_card(ctx).expect("surface"));
            assert!(json.contains("Le Petit Train d'Antibes"), "{json}");
            assert!(json.contains(AFFILIATE_URL));
            // Le quatrième produit reste pour le détail.
            assert!(!json.contains("Excursion en voilier"), "{json}");
            // La photo est une vignette dans l'emplacement `leading` de la tuile (§2.13), pas
            // un bandeau plein format : celui-là reste dans la feuille.
            assert!(json.contains("\"leading\":{\"image\""), "{json}");
            assert!(!json.contains("\"type\":\"Image\""), "{json}");
        });
}

#[test]
#[serial]
fn the_section_is_off_until_the_host_turns_it_on() {
    guest()
        .with_connector_response("viator", "search_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(!json.contains("guest.viator.title"), "{json}");
            assert!(host.connector_calls().is_empty());
        });
}

#[test]
#[serial]
fn a_fresh_cache_is_served_without_calling_viator() {
    guest()
        .with_config(&enabled())
        .with_kv(
            "viator_cache.fr",
            cached(Duration::seconds(VIATOR_FRESH_SECS - 60), "Depuis le cache"),
        )
        .with_connector_response("viator", "search_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("Depuis le cache"), "{json}");
            assert!(host.connector_calls().is_empty());
        });
}

#[test]
#[serial]
fn a_day_old_cache_is_refreshed() {
    guest()
        .with_config(&enabled())
        .with_kv(
            "viator_cache.fr",
            cached(Duration::seconds(VIATOR_FRESH_SECS + 60), "Depuis le cache"),
        )
        .with_connector_response("viator", "search_products", RECORDED)
        .run_with(|ctx, host| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("Le Petit Train d'Antibes"), "{json}");
            assert!(!json.contains("Depuis le cache"));
            assert_eq!(host.connector_calls().len(), 1);
        });
}

#[test]
#[serial]
fn when_viator_fails_a_cache_under_seven_days_is_still_shown() {
    guest()
        .with_config(&enabled())
        .with_kv(
            "viator_cache.fr",
            cached(Duration::days(3), "Gardé trois jours"),
        )
        .with_connector_error(
            "viator",
            "search_products",
            "connector_pool_quota_exhausted",
        )
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("Gardé trois jours"), "{json}");
        });
}

#[test]
#[serial]
fn a_cache_older_than_seven_days_is_never_shown() {
    guest()
        .with_config(&enabled())
        .with_kv(
            "viator_cache.fr",
            cached(Duration::seconds(VIATOR_STALE_MAX_SECS + 60), "Périmé"),
        )
        .with_connector_error("viator", "search_products", "connector_egress_failed")
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(!json.contains("Périmé"), "{json}");
            assert!(!json.contains("guest.viator.title"), "{json}");
        });
}

#[test]
#[serial]
fn a_viator_failure_without_cache_still_renders_the_rest_of_the_booklet() {
    guest()
        .with_config(&json!({
            "spots": [{ "id": "s1", "title": { "fr": "Plage" } }],
            "viator_enabled": true
        }))
        .with_connector_error("viator", "search_products", "connector_egress_failed")
        .run(|ctx| {
            let json = surface_json(&render_explore_detail(ctx).expect("surface"));
            assert!(json.contains("Plage"), "{json}");
            assert!(!json.contains("guest.viator.title"), "{json}");
            assert!(!json.contains("guest.error.title"), "{json}");
        });
}

/// Sans la clé de Portaki, l'appel échoue ; la section se tait, et l'écran de l'hôte le dit
/// tant que le constat a moins d'un jour.
#[test]
#[serial]
fn without_portakis_key_the_section_is_silent_and_the_host_is_told() {
    let (ctx, host) = guest()
        .with_config(&enabled())
        .with_connector_error("viator", "search_products", "connector_credential_missing")
        .build();
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_explore_detail(ctx.clone()).expect("surface"));
        assert!(!json.contains("guest.viator.title"), "{json}");
    });
    assert!(backend.kv_get("viator_key_missing").expect("kv").is_some());
    MockContext::host()
        .with_config(&enabled())
        .with_kv("viator_key_missing", b"1".to_vec())
        .run(|ctx| {
            let json = surface_json(&render_host_main(ctx).expect("host main"));
            assert!(
                json.contains("i18n:host.viator.status.missingKey"),
                "{json}"
            );
        });
}

/// Un appel qui aboutit efface le constat : la clé a été posée depuis.
#[test]
#[serial]
fn a_successful_call_clears_the_missing_key_note() {
    let (ctx, host) = guest()
        .with_config(&enabled())
        .with_kv("viator_key_missing", b"1".to_vec())
        .with_connector_response("viator", "search_products", RECORDED)
        .build();
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        assert!(
            surface_json(&render_explore_detail(ctx.clone()).expect("surface"))
                .contains("Le Petit Train d'Antibes")
        );
    });
    assert!(backend.kv_get("viator_key_missing").expect("kv").is_none());
}

/// Sans ville lisible dans l'adresse, rien n'est cherché, et l'écran de l'hôte le dit.
#[test]
#[serial]
fn a_property_without_city_is_never_searched() {
    let (mut ctx, host) = guest()
        .with_config(&enabled())
        .with_connector_response("viator", "search_products", RECORDED)
        .build();
    ctx.property.address = None;
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_explore_detail(ctx.clone()).expect("surface"));
        assert!(!json.contains("guest.viator.title"), "{json}");
    });
    assert!(backend.connector_calls().is_empty());

    let (mut ctx, host) = MockContext::host().with_config(&enabled()).build();
    ctx.property.address = Some("06400".into());
    with_host(host, ctx.clone(), || {
        let json = surface_json(&render_host_main(ctx.clone()).expect("host main"));
        assert!(
            json.contains("i18n:host.viator.status.missingCity"),
            "{json}"
        );
    });
}

#[test]
#[serial]
fn an_unsupported_booklet_language_asks_viator_in_english() {
    let (mut ctx, host) = guest()
        .with_config(&enabled())
        .with_connector_response("viator", "search_products", RECORDED)
        .build();
    ctx.locale = "uk-UA".to_string();
    let backend = host.clone();
    with_host(host, ctx.clone(), || {
        render_explore_detail(ctx.clone()).expect("surface");
    });
    let args: Value = serde_json::from_str(&backend.connector_calls()[0].args_json).expect("args");
    assert_eq!(args["lang"], "en");
    assert_eq!(args["searchTerm"], "Cannes");
}

#[test]
#[serial]
fn the_host_sheet_says_why_nothing_would_show() {
    MockContext::host().with_config(&enabled()).run(|ctx| {
        let json = surface_json(&render_host_main(ctx).expect("host main"));
        assert!(json.contains("i18n:host.section.viator"), "{json}");
        assert!(json.contains("i18n:host.viator.status.ready"), "{json}");
    });
    MockContext::host().run(|ctx| {
        let json = surface_json(&render_host_main(ctx).expect("host main"));
        assert!(json.contains("i18n:host.viator.status.off"), "{json}");
    });
}

/// Toutes les clés `i18n:` des surfaces Viator existent en fr et en en.
#[test]
#[serial]
fn every_viator_key_exists_in_both_bundles() {
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
    guest()
        .with_config(&enabled())
        .with_connector_response("viator", "search_products", RECORDED)
        .run(|ctx| {
            collect(
                &surface_json(&render_explore_detail(ctx.clone()).expect("surface")),
                &mut refs,
            );
            collect(
                &surface_json(&render_home_card(ctx).expect("surface")),
                &mut refs,
            );
        });
    MockContext::host().with_config(&enabled()).run(|ctx| {
        collect(
            &surface_json(&render_host_main(ctx).expect("host main")),
            &mut refs,
        )
    });
    assert!(refs.contains("guest.viator.disclosure"));
    assert!(refs.contains("host.viator.minRating.4"));

    let runtime_keys = [
        "guest.viator.priceFrom",
        "guest.viator.rating",
        "connector.viator.name",
        "host.viator.status.off",
        "host.viator.status.missingKey",
        "host.viator.status.missingCity",
    ];
    for raw in [FR_BUNDLE, EN_BUNDLE] {
        let bundle: Value = serde_json::from_str(raw).expect("bundle");
        for key in refs.iter().map(String::as_str).chain(runtime_keys) {
            assert!(bundle.get(key).is_some(), "clé manquante — {key}");
        }
    }
}
