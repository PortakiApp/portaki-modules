//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use consumables::{render_guest_form, render_home_card, reset_test_store, seed_defaults};
use portaki_sdk::prelude::EmptyArgs;

/// Le catalogue par défaut du module, celui qu'un hôte obtient en un clic.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let (card, form) = previews::guest(root).run(|ctx| {
        seed_defaults(ctx.clone(), EmptyArgs::default()).expect("seed defaults");
        (
            render_home_card(ctx.clone()).expect("render"),
            render_guest_form(ctx).expect("render"),
        )
    });
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("guest.form", form)],
    );
}
