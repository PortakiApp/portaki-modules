//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;
mod sample;

use rules::{render_explore_detail, render_home_card};

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let (detail, home) = sample::seeded(previews::guest(root)).run(|ctx| {
        (
            render_explore_detail(ctx.clone()).expect("render"),
            render_home_card(ctx).expect("render"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
        vec![("home.card", home)],
        None,
    );
}
