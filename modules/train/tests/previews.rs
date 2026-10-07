//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use train::{render_explore_detail, render_explore_item, render_home_card, render_upcoming_card};

/// Les aperçus du catalogue montrent un vrai tableau : la gare vient de la configuration
/// d'exemple, les départs du fournisseur mocké.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root)
        .with_config(&serde_json::json!({ "station": "Antibes" }))
        .with_connector_response(
            "sncf",
            "find_place",
            include_str!("fixtures/sncf-places.json"),
        )
        .with_connector_response(
            "sncf",
            "departures",
            include_str!("fixtures/sncf-departures.json"),
        );
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("render"));
    let item = context.clone().run(|mut ctx| {
        ctx.input = serde_json::json!({ "departureId": "20261004-0812-17654" });
        render_explore_item(ctx).expect("render")
    });
    let upcoming = context
        .clone()
        .run(|ctx| render_upcoming_card(ctx).expect("render"));
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.detail", detail),
            ("explore.item", item),
            ("upcoming.card", upcoming),
        ],
        vec![("home.card", home)],
        None,
    );
}
