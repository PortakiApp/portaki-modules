//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::json;

use appliances::{
    get_content, render_explore_detail, render_explore_item, render_home_card, replace_devices,
    reset_test_store, save_appliance, ApplianceStatus, GetContentArgs, ReplaceDeviceSlot,
    ReplaceDevicesArgs, SaveApplianceArgs,
};

fn seed_two_devices(ctx: portaki_sdk::prelude::Context) {
    save_appliance(
        ctx.clone(),
        SaveApplianceArgs {
            id: Some("tv".into()),
            name: "Télévision".into(),
            emoji: "📺".into(),
            description: json!({
                "type": "doc",
                "content": [{
                    "type": "bulletList",
                    "content": [{
                        "type": "listItem",
                        "content": [{
                            "type": "paragraph",
                            "content": [{ "type": "text", "text": "Allumez avec la télécommande." }]
                        }]
                    }]
                }]
            })
            .to_string(),
            featured: true,
            order: Some(0),
            location: "Salon · Samsung 55\"".into(),
            manual_url: "https://example.com/tv-manual".into(),
            safety_note: String::new(),
            status: ApplianceStatus::Active,
        },
    )
    .expect("save tv");
    save_appliance(
        ctx,
        SaveApplianceArgs {
            id: Some("washer".into()),
            name: "Lave-linge".into(),
            emoji: "🌀".into(),
            description: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "ECO 30°" }]
                }]
            })
            .to_string(),
            featured: false,
            order: Some(1),
            location: "Salle de bain · Bosch".into(),
            manual_url: String::new(),
            safety_note: "Pas de machine après 21 h.".into(),
            status: ApplianceStatus::Active,
        },
    )
    .expect("save washer");
}

#[test]
#[serial]
fn home_card_empty_without_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_featured_only_and_detail_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let card = render_home_card(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
            assert!(SurfaceAssertions::new(&card).contains_type("ListItem"));
            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("Télévision"));
            assert!(!card_json.contains("Lave-linge"));
            assert!(card_json.contains("\"type\":\"openOverlay\""));
            assert!(card_json.contains("explore.detail"));
            assert!(card_json.contains("appliances/tv"));

            let detail = render_explore_detail(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("Card"));
            assert!(SurfaceAssertions::new(&detail).contains_type("ListItem"));
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("Télévision"));
            assert!(detail_json.contains("Lave-linge"));
            assert!(detail_json.contains("appliances/washer"));
        });
}

/// « Voir les N appareils » (§2.4), avec le compte de la liste et non celui des tuiles.
///
/// Les aperçus ne couvrent que `explore.detail` et `explore.item` : la carte d'accueil n'y est pas,
/// donc ce bouton n'a que ce test pour le tenir.
#[test]
#[serial]
fn the_home_card_says_how_many_appliances_there_are() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let card = render_home_card(ctx.clone()).expect("render");
            let json = serde_json::to_string(&card).expect("json");

            assert!(SurfaceAssertions::new(&card).contains_type("Button"));
            // Le nombre est glissé par le service de traduction de l'hôte, que le contexte de test
            // ne fournit pas : il rend ici la clé. Ce que ce test tient, c'est que le bouton existe
            // et porte bien ce libellé-là — jamais vide, jamais celui du repli sans nombre.
            assert!(json.contains("home.card.seeAll"), "{json}");
            assert!(!json.contains("\"label\":\"\""), "{json}");
        });
}

/// La liste complète groupe par pièce (§2.4), et range sous « Autres » ce que l'hôte n'a pas placé.
#[test]
#[serial]
fn the_full_list_is_grouped_by_room() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let detail = render_explore_detail(ctx.clone()).expect("render");
            let json = serde_json::to_string(&detail).expect("json");

            // Deux pièces différentes dans la fixture : deux cartes titrées, comme la maquette
            // les dessine. Avant, un seul bloc avec des intertitres `Eyebrow`.
            assert_eq!(json.matches("\"type\":\"Card\"").count(), 2, "{json}");
            assert!(
                !SurfaceAssertions::new(&detail).contains_type("Eyebrow"),
                "{json}"
            );
        });
}

#[test]
#[serial]
fn explore_item_uses_device_id_and_howto_steps() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());

            let mut tv_ctx = ctx.clone();
            tv_ctx.input = json!({ "deviceId": "tv" });
            let tv = render_explore_item(tv_ctx).expect("render");
            assert!(SurfaceAssertions::new(&tv).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&tv).contains_type("Eyebrow"));
            assert!(SurfaceAssertions::new(&tv).contains_type("Button"));
            // La notice n'est plus un lien nu : elle est une rangée de la carte « Notices » (§3.1).
            assert!(SurfaceAssertions::new(&tv).contains_type("Card"));
            let tv_json = serde_json::to_string(&tv).expect("json");
            assert!(tv_json.contains("Télévision"));
            assert!(tv_json.contains("Allumez avec la télécommande."));
            assert!(tv_json.contains("https://example.com/tv-manual"));
            assert!(tv_json.contains("openHostChat"));
            assert!(!tv_json.contains("Lave-linge"));

            let mut washer_ctx = ctx.clone();
            washer_ctx.input = json!({ "deviceId": "washer" });
            let washer = render_explore_item(washer_ctx).expect("render");
            assert!(SurfaceAssertions::new(&washer).contains_type("InfoBanner"));
            let washer_json = serde_json::to_string(&washer).expect("json");
            assert!(washer_json.contains("Lave-linge"));
            assert!(washer_json.contains("Pas de machine après 21 h."));
            assert!(washer_json.contains("ECO 30"));
        });
}

/// Une description sans liste part dans `RichText` **telle quelle** : `content` est un champ
/// TipTap, et c'est le livret qui convertit.
///
/// Le module y posait du HTML pré-rendu. Depuis que le livret refuse d'injecter ce qui n'est pas du
/// TipTap, ce HTML s'affichait littéralement — « <p>Appuyez… </p> », balises comprises, dans la
/// carte « Mode d'emploi ».
#[test]
#[serial]
fn explore_item_sends_tiptap_not_html_to_rich_text() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let description = json!({
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{ "type": "text", "text": "Appuyez 2 secondes." }]
                    },
                    {
                        "type": "paragraph",
                        "content": [{
                            "type": "text",
                            "text": "Fond aimanté.",
                            "marks": [{ "type": "bold" }]
                        }]
                    }
                ]
            })
            .to_string();
            save_appliance(
                ctx.clone(),
                SaveApplianceArgs {
                    id: Some("plaques".into()),
                    name: "Plaques à induction".into(),
                    emoji: String::new(),
                    description: description.clone(),
                    featured: false,
                    order: None,
                    location: String::new(),
                    manual_url: String::new(),
                    safety_note: String::new(),
                    status: ApplianceStatus::Active,
                },
            )
            .expect("save");

            let mut item_ctx = ctx.clone();
            item_ctx.input = json!({ "deviceId": "plaques" });
            let item = render_explore_item(item_ctx).expect("render");
            let content =
                rich_text_content(&item).expect("un RichText dans la carte mode d'emploi");

            assert_eq!(content, description);
            assert!(!content.contains("<p>"), "{content}");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&content)
                    .expect("TipTap")
                    .get("type")
                    .and_then(|t| t.as_str()),
                Some("doc"),
                "{content}"
            );
        });
}

/// Une description vide ne laisse pas une carte qui ne porte que son chapeau.
#[test]
#[serial]
fn explore_item_hides_howto_card_for_an_empty_description() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_appliance(
                ctx.clone(),
                SaveApplianceArgs {
                    id: Some("vide".into()),
                    name: "Grille-pain".into(),
                    emoji: String::new(),
                    description: json!({ "type": "doc", "content": [{ "type": "paragraph" }] })
                        .to_string(),
                    featured: false,
                    order: None,
                    location: String::new(),
                    manual_url: String::new(),
                    safety_note: String::new(),
                    status: ApplianceStatus::Active,
                },
            )
            .expect("save");

            let mut item_ctx = ctx.clone();
            item_ctx.input = json!({ "deviceId": "vide" });
            let item = render_explore_item(item_ctx).expect("render");
            assert!(
                !SurfaceAssertions::new(&item).contains_type("Eyebrow"),
                "{}",
                serde_json::to_string(&item).expect("json")
            );
        });
}

/// Le `content` du premier `RichText` de l'arbre.
fn rich_text_content(surface: &impl serde::Serialize) -> Option<String> {
    fn walk(node: &serde_json::Value) -> Option<String> {
        if node.get("type").and_then(|t| t.as_str()) == Some("RichText") {
            return node
                .get("content")
                .and_then(|c| c.as_str())
                .map(str::to_string);
        }
        match node {
            serde_json::Value::Object(map) => map.values().find_map(walk),
            serde_json::Value::Array(items) => items.iter().find_map(walk),
            _ => None,
        }
    }
    walk(&serde_json::to_value(surface).expect("json"))
}

#[test]
#[serial]
fn explore_item_missing_device_id_is_not_found() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let item = render_explore_item(ctx).expect("render");
            let json = serde_json::to_string(&item).expect("json");
            assert!(json.contains("explore.item.notFound"));
            assert!(!json.contains("Télévision"));
        });
}

#[test]
#[serial]
fn get_content_returns_devices() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                    device_id: None,
                },
            )
            .expect("get");
            assert_eq!(view.devices.len(), 2);
            assert!(view.devices.iter().any(|d| d.name.contains("Télévision")));
        });
}

#[test]
#[serial]
fn migrates_legacy_payload_on_read() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let legacy = json!({
                "safety_notice": "Coupez l'eau en cas de fuite.",
                "devices": [{
                    "id": "tv",
                    "icon": "📺",
                    "title": "Télévision",
                    "subtitle": "Salon",
                    "steps": ["Allumez", "HDMI 1"],
                    "tip": "Remote on stand"
                }]
            })
            .to_string();
            appliances::store_save_legacy_for_tests(legacy).expect("seed legacy");
            let view = get_content(
                ctx.clone(),
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                    device_id: None,
                },
            )
            .expect("get");
            assert_eq!(view.devices.len(), 1);
            assert_eq!(view.devices[0].name, "Télévision");
            assert_eq!(view.devices[0].emoji, "📺");
            assert!(!view.devices[0].featured);
            assert!(view.devices[0].description.contains("bulletList"));
            assert!(view.safety_notice.contains("Coupez l'eau"));
            let card = render_home_card(ctx).expect("render");
            // featured=false after migration → empty featured card children, still Card
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
        });
}

/// La carte « Notices » : lien seul, papier seul, les deux, ou masquée (§3.1 et §9).
#[test]
#[serial]
fn the_manuals_card_appears_only_when_there_is_a_manual() {
    for (manual_url, paper, shown) in [
        (
            "https://example.com/m.pdf",
            "Boîte rouge, étagère du salon",
            true,
        ),
        ("https://example.com/m.pdf", "", true),
        ("", "Boîte rouge, étagère du salon", true),
        ("", "", false),
    ] {
        reset_test_store();
        MockContext::guest()
            .with_property(Property::default())
            .with_capabilities(&[capability::core::STORAGE])
            .run(|ctx| {
                replace_devices(
                    ctx.clone(),
                    ReplaceDevicesArgs {
                        safety_notice: String::new(),
                        paper_manuals_location: Some(paper.to_string()),
                        devices: vec![ReplaceDeviceSlot {
                            id: "tv".into(),
                            name: "Télévision".into(),
                            manual_url: manual_url.to_string(),
                            ..ReplaceDeviceSlot::default()
                        }],
                    },
                )
                .expect("save");

                let mut item_ctx = ctx.clone();
                item_ctx.input = json!({ "deviceId": "tv" });
                let json_out =
                    serde_json::to_string(&render_explore_item(item_ctx).expect("render")).unwrap();

                assert_eq!(
                    json_out.contains("explore.item.manuals"),
                    shown,
                    "lien: {manual_url:?}, papier: {paper:?} — {json_out}"
                );
                assert_eq!(
                    json_out.contains("explore.item.manual.paper"),
                    !paper.is_empty()
                );
            });
    }
}

/// L'emplacement des notices papier vaut pour tous les appareils, pas pour un seul.
#[test]
#[serial]
fn the_paper_location_is_shared_by_every_appliance() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            replace_devices(
                ctx.clone(),
                ReplaceDevicesArgs {
                    safety_notice: String::new(),
                    paper_manuals_location: Some("Boîte rouge".into()),
                    devices: vec![
                        ReplaceDeviceSlot {
                            id: "tv".into(),
                            name: "Télévision".into(),
                            ..ReplaceDeviceSlot::default()
                        },
                        ReplaceDeviceSlot {
                            id: "washer".into(),
                            name: "Lave-linge".into(),
                            ..ReplaceDeviceSlot::default()
                        },
                    ],
                },
            )
            .expect("save");

            for device in ["tv", "washer"] {
                let mut item_ctx = ctx.clone();
                item_ctx.input = json!({ "deviceId": device });
                let out =
                    serde_json::to_string(&render_explore_item(item_ctx).expect("render")).unwrap();
                assert!(out.contains("Boîte rouge"), "{device}: {out}");
            }
        });
}

/// Le numéro d'une étape est un repère en tête de rangée, pas le texte de l'étape.
#[test]
#[serial]
fn a_step_reads_its_text_not_its_number() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let mut item_ctx = ctx.clone();
            item_ctx.input = json!({ "deviceId": "tv" });
            let out =
                serde_json::to_string(&render_explore_item(item_ctx).expect("render")).unwrap();

            // Le titre porte la consigne, le repère porte le rang.
            assert!(
                out.contains("\"title\":\"Allumez avec la télécommande.\""),
                "{out}"
            );
            assert!(out.contains("\"index\":1"), "{out}");
            assert!(!out.contains("\"title\":\"1\""), "{out}");
        });
}
