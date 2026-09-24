//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use issue_report::{render_guest_form, render_home_card, reset_test_store, submit, SubmitArgs};

/// Un signalement déjà envoyé, pour que la carte montre son suivi.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let (card, form) = previews::guest(root).run(|ctx| {
        submit(
            ctx.clone(),
            SubmitArgs {
                category: "appliance".into(),
                summary: "Le four ne chauffe plus".into(),
                details: Some("Le voyant s'allume mais la température ne monte pas.".into()),
                photo: None,
            },
        )
        .expect("submit");
        (render_home_card(ctx.clone()), render_guest_form(ctx))
    });
    previews::check(root, vec![("home.card", card), ("guest.form", form)]);
}
