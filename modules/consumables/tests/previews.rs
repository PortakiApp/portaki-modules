//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;
mod sample;

use consumables::{render_guest_form, render_home_card};

/// Le catalogue par défaut du module, celui qu'un hôte obtient en un clic.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let (card, form) = sample::seeded(previews::guest(root)).run(|ctx| {
        (
            render_home_card(ctx.clone()).expect("render"),
            render_guest_form(ctx).expect("render"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("guest.form", form)],
        // Toutes les surfaces ont un chemin : rien à ajouter pour la démo.
        Vec::new(),
        None,
    );
}
