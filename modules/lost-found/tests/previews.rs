//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use lost_found::{
    render_guest_form, render_home_card, render_post_stay_card, reset_test_store, submit,
    SubmitArgs,
};
use serde_json::json;

fn sample_config() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "host_note": "Les objets retrouvés sont gardés un mois ; l'envoi par la poste est à votre charge."
    }))
    .expect("config json")
}

/// Un objet déjà signalé par le voyageur, pour que la carte montre son suivi.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let (card, form, post_stay) = previews::guest(root)
        .with_kv("config", sample_config())
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: "Chargeur de téléphone blanc".into(),
                    contact_hint: None,
                    details: Some("Sans doute branché près du lit de la chambre 2.".into()),
                },
            )
            .expect("submit");
            (
                render_home_card(ctx.clone()).expect("guest surface"),
                render_guest_form(ctx.clone()).expect("guest surface"),
                render_post_stay_card(ctx).expect("guest surface"),
            )
        });
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("home.card", card),
            ("guest.form", form),
            ("post-stay.card", post_stay),
        ],
    );
}
