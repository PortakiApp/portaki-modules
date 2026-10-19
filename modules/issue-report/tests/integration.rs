//! Ce que le signalement fait de plus que le dépôt du SDK : sa règle d'admission, ses e-mails,
//! sa carte et son formulaire. Statuts, historique et boîte de réception sont testés dans le SDK.

use issue_report::{
    __portaki_reports_note as note, __portaki_reports_submit as submit, render_guest_form,
    render_home_card, render_host_reports, Reports,
};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

fn report() -> serde_json::Value {
    json!({ "category": "appliance", "summary": "Le four ne chauffe plus" })
}

#[test]
fn a_category_the_host_dropped_is_refused() {
    let refused = MockContext::guest()
        .with_config(&json!({ "categories": ["noise"] }))
        .run(|ctx| submit(ctx, report()).unwrap_err().to_string());
    assert!(refused.contains("category_not_offered"), "{refused}");
}

#[test]
fn outside_the_open_phases_the_form_is_closed() {
    let (refused, card) = MockContext::guest()
        .with_config(&json!({ "phases": ["after"] }))
        .run(|ctx| {
            (
                submit(ctx.clone(), report()).unwrap_err().to_string(),
                render_home_card(ctx).unwrap(),
            )
        });
    assert!(refused.contains("out_of_phase"), "{refused}");
    assert!(serde_json::to_string(&card)
        .unwrap()
        .contains("home.card.closed"));
}

#[test]
fn a_report_emails_the_host_and_shows_on_the_card() {
    let (ctx, host) = MockContext::guest().build();
    portaki_sdk::host::with_host(host.clone(), ctx.clone(), || {
        submit(ctx.clone(), report()).unwrap();
        let card = serde_json::to_string(&render_home_card(ctx.clone()).unwrap()).unwrap();
        assert!(
            card.contains("Le four ne chauffe plus") && card.contains("home.card.thanks"),
            "{card}"
        );
    });
    let sent = host.sent_emails();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].email_id.starts_with("submitted-"));
}

#[test]
fn the_hosts_reply_emails_the_guest_and_shows_under_the_report() {
    let (ctx, host) = MockContext::guest()
        .with_translation("home.card.reply", "Réponse : {reply}")
        .build();
    portaki_sdk::host::with_host(host.clone(), ctx.clone(), || {
        submit(ctx.clone(), report()).unwrap();
        let id = Reports::list(&ctx).unwrap()[0].id;
        let mut host_ctx = ctx.clone();
        host_ctx.guest = None;
        note(
            host_ctx,
            serde_json::from_value(json!({ "id": id, "text": "Le technicien passe demain." }))
                .unwrap(),
        )
        .unwrap();
        let card = serde_json::to_string(&render_home_card(ctx.clone()).unwrap()).unwrap();
        assert!(card.contains("Le technicien passe demain."), "{card}");
    });
    assert!(host
        .sent_emails()
        .iter()
        .any(|e| e.email_id.starts_with("host-reply-")));
}

#[test]
fn the_form_offers_the_hosts_categories_and_no_photo_when_off() {
    let form = MockContext::guest()
        .with_config(&json!({ "categories": ["appliance", "noise"], "photo_allowed": false }))
        .run(|ctx| render_guest_form(ctx).unwrap());
    let tree = serde_json::to_string(&form).unwrap();
    assert!(
        tree.contains("form.category.noise") && !tree.contains("form.category.wifi"),
        "{tree}"
    );
    assert!(!SurfaceAssertions::new(&form).contains_type("ImageUpload"));
}

#[test]
fn the_inbox_lists_open_reports_and_opens_one() {
    let (ctx, host) = MockContext::guest().build();
    let mut host_ctx = ctx.clone();
    host_ctx.guest = None;
    let (list, detail) = portaki_sdk::host::with_host(host, ctx.clone(), || {
        submit(ctx.clone(), report()).unwrap();
        let id = Reports::list(&ctx).unwrap()[0].id;
        let list = render_host_reports(host_ctx.clone()).unwrap();
        let mut open = host_ctx.clone();
        open.input = json!({ "reportsId": id });
        (list, render_host_reports(open).unwrap())
    });
    assert_eq!(SurfaceAssertions::new(&list).count_type("FeedItem"), 1);
    assert_eq!(SurfaceAssertions::new(&list).count_type("FilterChip"), 3);
    let detail = serde_json::to_string(&detail).unwrap();
    assert!(
        detail.contains("host.history.title") && detail.contains(r#""name":"text""#),
        "{detail}"
    );
}
