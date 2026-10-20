//! La page de réglages (spec Consommables §2) : le tableau des produits, les réglages des
//! demandes, les demandes reçues et la réponse au voyageur. La fusion des lignes (par id, champ
//! absent gardé) est celle du SDK, testée là-bas.

mod sample;

use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

use consumables::{render_guest_form, render_host_main, render_host_stay, SubmitArgs};
use portaki_sdk::host::email::EmailAudience;
use portaki_test_utils::{MockContext, MockContextBuilder};

fn host_json(ctx: &portaki_sdk::prelude::Context) -> String {
    serde_json::to_string(&render_host_main(ctx.clone()).expect("render")).expect("json")
}

/// Liste vide : la liste de départ en un clic, et « Ajouter » ouvre une ligne et son panneau.
#[test]
#[serial]
fn an_empty_table_offers_the_starter_list_and_adds_a_row() {
    MockContext::host().run(|ctx| {
        let json = host_json(&ctx);
        assert!(
            json.contains("host.starter.add") && json.contains("seedDefaults"),
            "{json}"
        );
        assert!(json.contains(r#""rowName":"products""#), "{json}");
        assert!(!json.contains("products.0.label"), "{json}");
    });
    let added = Uuid::new_v4();
    MockContext::host().run(|mut ctx| {
        ctx.input = json!({ "productsNew": added.to_string() });
        let json = host_json(&ctx);
        assert!(json.contains(&added.to_string()), "{json}");
        assert!(json.contains("products.0.label"), "{json}");
        assert!(json.contains("host.product.location.placeholder"), "{json}");
        assert!(json.contains("1 / 30"), "{json}");
    });
}

/// Le tableau : emoji proposé devant le nom, l'emplacement, et le filtre « jamais demandé ».
#[test]
#[serial]
fn the_table_shows_the_emoji_the_place_and_the_filter() {
    let rows = json!([{ "id": sample::id(0).to_string(), "label": { "fr": "Thé vert" },
                        "location": { "fr": "Tiroir" } }]);
    MockContext::host()
        .with_resource("products", rows, json!({}))
        .run(|ctx| {
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
    let rows = |place: &str| json!([{ "id": sample::id(0).to_string(), "label": { "fr": "Sel" }, "location": { "fr": place } }]);
    let form = MockContext::guest()
        .with_resource("products", rows("Placard au-dessus de l'évier"), json!({}))
        .run(|ctx| serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap());
    assert!(form.contains("Placard au-dessus de l'évier"), "{form}");
    assert!(form.contains("🧂"), "suggested emoji: {form}");
    MockContext::host()
        .with_resource("products", rows(&"p".repeat(81)), json!({}))
        .run(|ctx| {
            let field = format!("{}.location", sample::id(0));
            assert_eq!(
                sample::blocking(&ctx, &field).as_deref(),
                Some("80 caractères au maximum.")
            );
        });
}

/// « Prévenir par » : au moins un canal, demandes ouvertes ; fermées, rien ne bloque et les
/// réglages se masquent.
#[test]
#[serial]
fn at_least_one_channel_while_requests_are_open() {
    MockContext::host().run(|ctx| {
        sample::save(&ctx, json!({ "notify": [] }));
        assert_eq!(
            sample::blocking(&ctx, "notify").as_deref(),
            Some("Choisissez au moins un canal.")
        );
        assert!(host_json(&ctx).contains("requests_enabled=true"));

        sample::save(&ctx, json!({ "notify": "[\"app\",\"sms\"]" }));
        assert_eq!(sample::blocking(&ctx, "notify"), None);

        sample::save(&ctx, json!({ "notify": [], "requests_enabled": false }));
        assert_eq!(sample::blocking(&ctx, "notify"), None);
        assert!(host_json(&ctx).contains("host.received.disabled"));
    });
}

fn ask(ctx: &portaki_sdk::prelude::Context) -> Uuid {
    let args = SubmitArgs {
        item_ids: vec![sample::id(0)],
        item_id: None,
        level: "missing".into(),
        note: None,
    };
    sample::submit(ctx, args).expect("submit");
    sample::requests(ctx)[0].id
}

fn one_request(builder: MockContextBuilder) -> usize {
    builder.run_with(|ctx, host| {
        ask(&ctx);
        host.sent_emails().len()
    })
}

/// Sans « E-mail » coché, la demande n'écrit pas à l'hôte ; avec, si.
#[test]
#[serial]
fn the_host_email_follows_the_notify_setting() {
    let coffee = [("Café", "", "☕")];
    let app_only = sample::with_meta(MockContext::guest(), &coffee, json!({ "notify": ["app"] }));
    assert_eq!(one_request(app_only), 0);
    assert_eq!(one_request(sample::with(MockContext::guest(), &coffee)), 1);
}

/// La table des demandes, l'encart du séjour, et « Répondre » par e-mail au voyageur — une seule
/// fois pour une même réponse.
#[test]
#[serial]
fn requests_table_stay_summary_and_reply_by_email() {
    let coffee = [("Café", "", "☕")];
    sample::with(MockContext::guest(), &coffee).run_with(|guest, host| {
        let report = ask(&guest);
        let ctx = sample::as_host(&guest);
        let sent = host.sent_emails().len();
        let json = host_json(&ctx);
        assert!(json.contains("host.received.title"), "{json}");
        assert!(json.contains("i18n:status.open"), "{json}");
        assert!(json.contains("☕ Café"), "{json}");
        assert!(json.contains("host.main.sendReply"), "{json}");
        assert!(json.contains("host.filter.recent"), "{json}");

        let reply = |text: &str| {
            let args = json!({ "id": report, "text": text });
            consumables::__portaki_requests_note(ctx.clone(), serde_json::from_value(args).unwrap())
                .expect("reply")
        };
        reply("Je passe ce soir.");
        reply("Je passe ce soir.");
        let emails = &host.sent_emails()[sent..];
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].audience, EmailAudience::Guest);
        assert!(emails[0].email_id.starts_with("host-reply-"));
        assert!(emails[0].content.body.fr.contains("Je passe ce soir."));

        let mut stay = ctx.clone();
        stay.input = json!({ "stayId": guest.guest.as_ref().unwrap().session_id.to_string() });
        let json = serde_json::to_string(&render_host_stay(stay)).unwrap();
        assert!(json.contains("host.stay.seeAll"), "{json}");
        assert!(json.contains("/modules/consumables"), "{json}");
    });
}
