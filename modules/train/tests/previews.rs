//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use train::{render_explore_detail, render_upcoming_card};

/// Le module n'a pas encore de configuration : gare et horaires sont ceux qu'il embarque.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root);
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("render"));
    let upcoming = context.run(|ctx| render_upcoming_card(ctx).expect("render"));
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail), ("upcoming.card", upcoming)],
    );
}
