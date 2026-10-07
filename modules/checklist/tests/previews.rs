//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use checklist::{
    create_checklist, render_home_card, render_post_stay_card, reset_test_store,
    CreateChecklistArgs,
};
use serial_test::serial;

/// La liste de départ que le module propose en un clic : celle qu'un hôte obtient sans rien
/// écrire, et donc celle qu'un aperçu doit montrer.
fn seed(ctx: &portaki_sdk::Context) {
    create_checklist(
        ctx.clone(),
        CreateChecklistArgs {
            template: "departure".into(),
        },
    )
    .expect("create");
}

#[test]
#[serial]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let context = previews::guest(root);
    let (card, post_stay) = context.run(|ctx| {
        seed(&ctx);
        (
            render_home_card(ctx.clone()).expect("card"),
            render_post_stay_card(ctx).expect("post-stay"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("post-stay.card", post_stay)],
        // Les deux surfaces ont un chemin : rien à ajouter pour la démo.
        Vec::new(),
        None,
    );
}
