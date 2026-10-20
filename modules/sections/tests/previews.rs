//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;
mod sample;

use sections::{render_explore_sheet, render_home_card};

/// Aucune surface n'a de chemin : le module n'entre pas au catalogue des aperçus, mais le livret
/// montre son mot dès l'accueil — c'est donc la démo qui en a besoin.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let (card, sheet) = sample::seeded(previews::guest(root)).run(|ctx| {
        (
            render_home_card(ctx.clone()).expect("card"),
            render_explore_sheet(ctx).expect("sheet"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        Vec::new(),
        vec![("home.card", card), ("explore.sheet", sheet)],
        None,
    );
}
