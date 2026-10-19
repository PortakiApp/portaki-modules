//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use issue_report::{render_guest_form, render_home_card};
use serde_json::json;

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    // Les aperçus sont pris la veille de l'arrivée : la démo ouvre le formulaire à toutes les
    // périodes, sans quoi elle ne montrerait que « contactez votre hôte ».
    let (ctx, host) = previews::guest(root)
        .with_config(&json!({ "phases": ["before", "during", "after"] }))
        .build();
    let (card, form) = portaki_sdk::host::with_host(host, ctx.clone(), || {
        issue_report::__portaki_reports_submit(
            ctx.clone(),
            json!({ "category": "appliance", "summary": "Le four ne chauffe plus",
                    "details": "Le voyant s'allume mais la température ne monte pas." }),
        )
        .expect("submit");
        (
            render_home_card(ctx.clone()).expect("guest surface"),
            render_guest_form(ctx.clone()).expect("guest surface"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("guest.form", form)],
        Vec::new(),
        None,
    );
}
