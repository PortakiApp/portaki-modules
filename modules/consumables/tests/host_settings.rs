//! La page de réglages (spec Consommables §2) : le tableau des produits et son panneau, les
//! réglages des demandes, les demandes reçues et la réponse au voyageur.

use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

use consumables::{
    list_for_stay, list_items, publish_readiness, render_guest_form, render_host_main,
    render_host_stay, reset_test_store, submit, update_config, update_status, ConsumableItemInput,
    ListForStayArgs, SubmitArgs, UpdateConfigArgs, UpdateStatusArgs,
};
use portaki_sdk::host::email::EmailAudience;
use portaki_test_utils::{MockContext, Property};

fn row(id: Option<Uuid>, label: Option<&str>, location: Option<&str>) -> ConsumableItemInput {
    ConsumableItemInput {
        id,
        label: label.map(str::to_string),
        label_fr: String::new(),
        label_en: String::new(),
        sort_order: 0,
        low_threshold: 0,
        emoji: String::new(),
        location: location.map(str::to_string),
    }
}

fn save(ctx: &portaki_sdk::prelude::Context, items: Vec<ConsumableItemInput>, extra: Value) {
    let mut args = json!({ "items": [] });
    args.as_object_mut()
        .unwrap()
        .extend(extra.as_object().cloned().unwrap_or_default());
    let mut args: UpdateConfigArgs = serde_json::from_value(args).expect("args");
    args.items = items;
    update_config(ctx.clone(), args).expect("updateConfig");
}

fn host_json(ctx: &portaki_sdk::prelude::Context) -> String {
    serde_json::to_string(&render_host_main(ctx.clone())).expect("json")
}

fn blocking(ctx: &portaki_sdk::prelude::Context, field: &str) -> Option<String> {
    publish_readiness(ctx.clone())
        .expect("readiness")
        .items
        .into_iter()
        .find(|check| check.id == format!("config.{field}"))
        .map(|check| check.hint.get("fr").to_string())
}

/// Liste vide : la liste de départ en un clic, et « Ajouter » ouvre une ligne et son panneau.
#[test]
#[serial]
fn an_empty_table_offers_the_starter_list_and_adds_a_row() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        let json = host_json(&ctx);
        assert!(json.contains("host.starter.add"), "{json}");
        assert!(json.contains("seedDefaults"), "{json}");
        assert!(json.contains(r#""rowName":"items""#), "{json}");
        assert!(json.contains("newItems"), "{json}");
        assert!(!json.contains("items.0.label"), "{json}");
    });
    let added = Uuid::new_v4();
    MockContext::host().run(|mut ctx| {
        ctx.input = json!({ "newItems": added.to_string() });
        let json = host_json(&ctx);
        assert!(json.contains(&added.to_string()), "{json}");
        assert!(json.contains("items.0.label"), "{json}");
        assert!(json.contains("host.product.location.placeholder"), "{json}");
        assert!(json.contains("1 / 30"), "{json}");
    });
}

/// Une ligne jamais ouverte n'envoie que son id : le produit garde nom, emoji et emplacement.
/// Une ligne absente est supprimée ; renommer touche la seule langue de l'hôte.
#[test]
#[serial]
fn rows_are_keyed_by_id_and_an_unopened_row_keeps_everything() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        save(
            &ctx,
            vec![
                row(
                    Some(Uuid::new_v4()),
                    Some("Café"),
                    Some("Placard au-dessus de l'évier"),
                ),
                row(Some(Uuid::new_v4()), Some("Thé"), None),
            ],
            json!({}),
        );
        let items = list_items(ctx.clone()).expect("items");
        assert_eq!(items.len(), 2);
        let (coffee, tea) = (items[0].id, items[1].id);

        // Le thé, jamais ouvert, passe devant ; le café est retiré.
        save(&ctx, vec![row(Some(tea), None, None)], json!({}));
        let items = list_items(ctx.clone()).expect("items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, tea);
        assert!(items[0].label_fr.contains("Thé"));
        assert!(!items.iter().any(|item| item.id == coffee));

        // Un emplacement vide l'efface ; un emplacement écrit s'affiche dans le tableau.
        save(
            &ctx,
            vec![row(Some(tea), Some("Thé vert"), Some("Tiroir"))],
            json!({}),
        );
        let json = host_json(&ctx);
        assert!(json.contains("🍵 Thé vert"), "suggested emoji: {json}");
        assert!(json.contains(r#""location":"Tiroir""#), "{json}");
        assert!(json.contains("host.filter.never"), "{json}");
    });
}

/// « Où le trouver » sous la tuile du voyageur, et 80 caractères au plus.
#[test]
#[serial]
fn the_location_reaches_the_guest_and_is_bounded() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        save(
            &ctx,
            vec![row(
                Some(Uuid::new_v4()),
                Some("Sel"),
                Some("Placard au-dessus de l'évier"),
            )],
            json!({}),
        );
    });
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let form = serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap();
            assert!(form.contains("Placard au-dessus de l'évier"), "{form}");
            assert!(form.contains("🧂"), "suggested emoji: {form}");
        });
    MockContext::host().run(|ctx| {
        let id = list_items(ctx.clone()).expect("items")[0].id;
        save(
            &ctx,
            vec![row(Some(id), None, Some(&"p".repeat(81)))],
            json!({}),
        );
        assert_eq!(
            blocking(&ctx, "items.0.location").as_deref(),
            Some("80 caractères au maximum.")
        );
    });
}

/// Un nom effacé dans le panneau : « Donnez un nom au produit. », la ligne reste à corriger.
#[test]
#[serial]
fn a_cleared_name_is_flagged_not_dropped() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        let id = Uuid::new_v4();
        save(&ctx, vec![row(Some(id), Some("Huile"), None)], json!({}));
        save(&ctx, vec![row(Some(id), Some(""), None)], json!({}));
        assert_eq!(list_items(ctx.clone()).expect("items").len(), 1);
        assert_eq!(
            blocking(&ctx, "items.0.label").as_deref(),
            Some("Donnez un nom au produit.")
        );
    });
}

/// « Prévenir par » : au moins un canal, demandes ouvertes ; fermées, rien ne bloque.
#[test]
#[serial]
fn at_least_one_channel_while_requests_are_open() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        save(&ctx, Vec::new(), json!({ "notify": [] }));
        assert_eq!(
            blocking(&ctx, "notify").as_deref(),
            Some("Choisissez au moins un canal.")
        );
        let json = host_json(&ctx);
        assert!(json.contains("requests_enabled=true"), "{json}");
        assert!(json.contains("host.notify.sms"), "{json}");

        save(&ctx, Vec::new(), json!({ "notify": "[\"app\",\"sms\"]" }));
        assert_eq!(blocking(&ctx, "notify"), None);

        save(
            &ctx,
            Vec::new(),
            json!({ "notify": [], "requests_enabled": false }),
        );
        assert_eq!(blocking(&ctx, "notify"), None);
        assert!(host_json(&ctx).contains("host.received.disabled"));
    });
}

fn one_request(builder: portaki_test_utils::MockContextBuilder) -> (Uuid, usize) {
    builder.run_with(|ctx, host| {
        let item = list_items(ctx.clone()).expect("items")[0].id;
        submit(
            ctx.clone(),
            SubmitArgs {
                item_ids: vec![item],
                item_id: None,
                level: "missing".into(),
                note: None,
            },
        )
        .expect("submit");
        let report = list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0].id;
        (report, host.sent_emails().len())
    })
}

/// Sans « E-mail » coché, la demande n'écrit pas à l'hôte ; avec, si.
#[test]
#[serial]
fn the_host_email_follows_the_notify_setting() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        save(&ctx, vec![row(None, Some("Café"), None)], json!({}));
    });
    let app_only = serde_json::to_vec(&json!({ "notify": ["app"] })).unwrap();
    let guest = || MockContext::guest().with_property(Property::default());
    assert_eq!(one_request(guest().with_kv("settings", app_only)).1, 0);
    assert_eq!(one_request(guest()).1, 1);
}

/// La table des demandes, l'encart du séjour, et « Répondre » par e-mail au voyageur — une seule
/// fois pour une même réponse.
#[test]
#[serial]
fn requests_table_stay_summary_and_reply_by_email() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        save(&ctx, vec![row(None, Some("Café"), None)], json!({}));
    });
    let (report, _) = one_request(MockContext::guest().with_property(Property::default()));

    MockContext::host().run_with(|ctx, host| {
        let json = host_json(&ctx);
        assert!(json.contains("host.received.title"), "{json}");
        assert!(json.contains("i18n:status.open"), "{json}");
        assert!(json.contains("☕ Café"), "{json}");
        assert!(json.contains("host.main.sendReply"), "{json}");
        assert!(json.contains("host.filter.recent"), "{json}");

        let reply = |text: &str| {
            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id: report,
                    status: "open".into(),
                    host_reply: Some(text.into()),
                },
            )
            .expect("reply")
        };
        reply("Je passe ce soir.");
        reply("Je passe ce soir.");
        let emails = host.sent_emails();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].audience, EmailAudience::Guest);
        assert!(emails[0].email_id.starts_with("host-reply-"));
        assert!(emails[0].content.body.fr.contains("Je passe ce soir."));
    });

    let stay = MockContext::guest().run(|ctx| ctx.guest.unwrap().session_id);
    MockContext::host().run(|mut ctx| {
        ctx.input = json!({ "stayId": stay.to_string() });
        let json = serde_json::to_string(&render_host_stay(ctx)).unwrap();
        assert!(json.contains("host.stay.seeAll"), "{json}");
        assert!(json.contains("/modules/consumables"), "{json}");
    });
}
