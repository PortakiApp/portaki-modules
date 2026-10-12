//! La page de réglages et l'onglet (spec Appareils §2) : le tableau, ses panneaux, `updateConfig`,
//! et l'effet de chaque nouveau réglage dans le livret.

use portaki_sdk::capability;
use portaki_sdk::prelude::Context;
use serde_json::{json, Value};
use serial_test::serial;

use portaki_test_utils::{MockContext, Property};

use appliances::{
    get_content, publish_readiness, render_explore_detail, render_explore_item, render_host_main,
    reset_test_store, update_config, GetContentArgs, UpdateConfigArgs,
};

fn host(run: impl FnOnce(Context)) {
    reset_test_store();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|mut ctx| {
            ctx.locale = "fr-FR".into();
            run(ctx)
        });
}

fn save(ctx: &Context, args: Value) {
    let args: UpdateConfigArgs = serde_json::from_value(args).expect("args");
    update_config(ctx.clone(), args).expect("updateConfig");
}

fn devices(ctx: &Context) -> Vec<appliances::Appliance> {
    get_content(ctx.clone(), GetContentArgs::default())
        .expect("content")
        .devices
}

fn seed(ctx: &Context) {
    save(
        ctx,
        json!({
            "featuredLimit": 4,
            "paperManualsLocation": "Boîte rouge",
            "devices": [
                { "id": "four", "name": "Four", "location": "Cuisine", "featured": true,
                  "active": true, "model": "Bosch HBA", "safetyNote": "Pas de pyrolyse" },
                { "id": "tv", "name": "Télévision", "location": "Salon", "featured": false,
                  "active": true, "steps": [{ "text": "Allumez", "image": "portaki-file:schema" }] },
            ]
        }),
    );
}

fn host_json(ctx: &Context, input: Value) -> String {
    let mut ctx = ctx.clone();
    ctx.input = input;
    serde_json::to_string(&render_host_main(ctx)).expect("json")
}

/// Une ligne jamais ouverte n'envoie que son id : l'appareil garde tout ce qu'il avait.
#[test]
#[serial]
fn a_row_that_only_sends_its_id_keeps_everything() {
    host(|ctx| {
        seed(&ctx);
        save(
            &ctx,
            json!({ "devices": [{ "id": "tv" }, { "id": "four", "featured": false }] }),
        );
        let saved = devices(&ctx);
        assert_eq!(
            saved.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
            ["tv", "four"]
        );
        let four = &saved[1];
        assert_eq!(four.name, "Four");
        assert_eq!(four.model, "Bosch HBA");
        assert_eq!(four.safety_note, "Pas de pyrolyse");
        assert!(!four.featured);
        assert_eq!(saved[0].steps[0].image, "portaki-file:schema");
    });
}

/// Supprimer = la ligne n'est plus envoyée ; « Ajouter » laissé vide ne crée rien ; « Actif »
/// décoché masque l'appareil.
#[test]
#[serial]
fn rows_are_removed_added_and_deactivated() {
    host(|ctx| {
        seed(&ctx);
        save(
            &ctx,
            json!({ "devices": [
                { "id": "four", "active": "false" },
                { "id": "new-1", "name": "", "emoji": "", "location": "" },
                { "id": "new-2", "name": "Cafetière", "manualFile": "portaki-file:pdf" },
            ] }),
        );
        let saved = devices(&ctx);
        assert_eq!(
            saved.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
            ["four", "new-2"]
        );
        assert_eq!(saved[0].status, appliances::ApplianceStatus::Hidden);
        assert_eq!(saved[1].manual_file, "portaki-file:pdf");
    });
}

/// Trop d'appareils en avant : l'auto-save garde la saisie, la publication le refuse.
#[test]
#[serial]
fn too_many_featured_is_saved_and_blocks_publishing() {
    host(|ctx| {
        seed(&ctx);
        save(
            &ctx,
            json!({ "featuredLimit": 2, "devices": [
                { "id": "four", "featured": true },
                { "id": "tv", "featured": true },
                { "id": "x", "name": "Bouilloire", "featured": true },
            ] }),
        );
        assert_eq!(devices(&ctx).iter().filter(|d| d.featured).count(), 3);
        let readiness =
            serde_json::to_string(&publish_readiness(ctx.clone()).expect("ready")).unwrap();
        assert!(
            readiness.contains("Vous avez déjà mis 2 appareils en avant."),
            "{readiness}"
        );
    });
}

/// Un appareil sans nom : « Donnez un nom à l'appareil. » sous le champ et à la publication.
#[test]
#[serial]
fn an_unnamed_appliance_is_reported() {
    host(|ctx| {
        save(
            &ctx,
            json!({ "devices": [{ "id": "x", "name": "", "model": "Bosch" }] }),
        );
        let readiness =
            serde_json::to_string(&publish_readiness(ctx.clone()).expect("ready")).unwrap();
        assert!(
            readiness.contains("Donnez un nom à l'appareil."),
            "{readiness}"
        );
        let json = host_json(&ctx, json!({}));
        assert!(json.contains("Donnez un nom à l'appareil."), "{json}");
    });
}

/// Deux cartes, le tableau éditable, et le panneau de chaque ligne nommé `devices.<i>.<clé>`.
#[test]
#[serial]
fn the_settings_page_is_two_sections_and_an_editable_table() {
    host(|ctx| {
        seed(&ctx);
        let json = host_json(&ctx, json!({}));
        let tree: Value = serde_json::from_str(&json).unwrap();
        let text = tree.to_string();
        assert_eq!(text.matches(r#""type":"Section""#).count(), 2, "{text}");
        for needle in [
            r#""rowName":"devices""#,
            r#""name":"devices.1.manualFile""#,
            r#""accept":"application/pdf""#,
            r#""maxBytes":20971520"#,
            r#""name":"devices.1.steps.0.image""#,
            r#""value":"portaki-file:schema""#,
            r#""name":"devices.0.active""#,
            r#""name":"roomOrder""#,
            r#""name":"paperManualsLocation""#,
            r#""kind":"check""#,
            r#""max":4"#,
            r#""filterKey":"room""#,
            "\"2 / 60\"",
            "📺 Télévision",
        ] {
            assert!(text.contains(needle), "{needle} absent : {text}");
        }
        // Un appareil déjà mis en avant : pas d'invite à en choisir.
        assert!(!text.contains("host.featured.none"));
    });
}

/// « Ajouter » : une ligne de plus, son id gardé d'un rendu à l'autre ; « Ajouter une étape » de même.
#[test]
#[serial]
fn adding_a_row_and_a_step_renders_them() {
    host(|ctx| {
        seed(&ctx);
        let json = host_json(&ctx, json!({ "newDevices": "abc", "steps.four": 1 }));
        assert!(json.contains(r#""id":"abc""#), "{json}");
        assert!(json.contains("\"3 / 60\""), "{json}");
        assert!(json.contains(r#""name":"devices.2.name""#), "{json}");
        assert!(
            json.contains(r#""name":"devices.0.steps.0.text""#),
            "{json}"
        );
        // Aucun mis en avant : l'invite (cas §9 #3).
        save(
            &ctx,
            json!({ "devices": [{ "id": "four", "featured": false }, { "id": "tv" }] }),
        );
        assert!(host_json(&ctx, json!({})).contains("host.featured.none"));
    });
}

fn guest(ctx_seed: impl FnOnce(&Context), run: impl FnOnce(Context)) {
    reset_test_store();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|mut ctx| {
            ctx.locale = "fr-FR".into();
            ctx_seed(&ctx);
        });
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(run);
}

fn item_json(ctx: &Context, id: &str) -> String {
    let mut ctx = ctx.clone();
    ctx.input = json!({ "deviceId": id });
    serde_json::to_string(&render_explore_item(ctx).expect("item")).expect("json")
}

/// Le PDF et le lien : seul le PDF (§3) ; le schéma sous son étape ; sans rien, « Pas encore de
/// mode d'emploi ».
#[test]
#[serial]
fn the_pdf_wins_over_the_link_and_steps_show_their_diagram() {
    guest(
        |ctx| {
            reset_test_store();
            save(
                ctx,
                json!({ "devices": [
                    { "id": "four", "name": "Four", "manualFile": "portaki-file:pdf",
                      "manualUrl": "https://example.com/four.pdf",
                      "steps": [{ "text": "Tournez", "image": "portaki-file:schema" }] },
                    { "id": "clim", "name": "Climatisation" },
                ] }),
            );
        },
        |ctx| {
            let four = item_json(&ctx, "four");
            assert!(four.contains("portaki-file:pdf"), "{four}");
            assert!(four.contains("explore.item.manual.pdf"), "{four}");
            assert!(!four.contains("example.com/four.pdf"), "{four}");
            assert!(four.contains(r#""url":"portaki-file:schema""#), "{four}");
            let clim = item_json(&ctx, "clim");
            assert!(clim.contains("explore.item.noManual"), "{clim}");
            // L'emoji proposé par le nom.
            assert!(clim.contains("❄️"), "{clim}");
        },
    );
}

/// L'ordre des pièces choisi par l'hôte ; « Autres » reste en dernier.
#[test]
#[serial]
fn the_room_order_orders_the_list() {
    guest(
        |ctx| {
            reset_test_store();
            save(
                ctx,
                json!({
                    "roomOrder": r#"[{"label":"Salon"},{"label":"Cuisine"}]"#,
                    "devices": [
                        { "id": "a", "name": "Four", "location": "Cuisine" },
                        { "id": "b", "name": "Aspirateur" },
                        { "id": "c", "name": "Télévision", "location": "Salon" },
                    ]
                }),
            );
        },
        |ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            let salon = json.find("\"Salon\"").expect("salon");
            let cuisine = json.find("\"Cuisine\"").expect("cuisine");
            let other = json.find("explore.detail.room.other").expect("autres");
            assert!(salon < cuisine && cuisine < other, "{json}");
        },
    );
}
