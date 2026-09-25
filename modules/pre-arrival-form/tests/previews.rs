//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use pre_arrival_form::{render_guest_form, render_home_card};

/// Les questions par défaut, la veille de l'arrivée : le formulaire est ouvert, rien
/// n'est encore rempli.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root);
    let card = context.clone().run(render_home_card).expect("home card");
    let form = context.run(render_guest_form).expect("form");
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("guest.form", form)],
    );
}
