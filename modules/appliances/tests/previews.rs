//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;
mod sample;

use appliances::{render_explore_detail, render_explore_item, render_home_card};
use serde_json::json;

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    // La démo installe un module qui reçoit les signalements : la fiche montre donc son bouton,
    // ouvert au chemin que ce module déclare pour `report.issue`.
    let reporter: portaki_sdk::host::module::ModulePeer =
        serde_json::from_value(json!({ "moduleId": "issue-report", "path": "issue-report/form" }))
            .expect("peer");
    let (detail, item, home, variants) = sample::seeded(previews::guest(root))
        .with_peers(appliances::REPORT_ISSUE, vec![reporter])
        .run(|ctx| {
            let item = |id: &str| {
                let mut item_ctx = ctx.clone();
                item_ctx.input = json!({ "deviceId": id });
                render_explore_item(item_ctx).expect("render")
            };
            // Chaque fiche : la démo ouvre celle de l'appareil tapé, pas toujours les plaques.
            let variants: Vec<_> = ["plaques", "lave-linge", "tv"]
                .into_iter()
                .map(|id| ("explore.item", json!({ "deviceId": id }), item(id)))
                .collect();
            (
                render_explore_detail(ctx.clone()).expect("render"),
                item("plaques"),
                render_home_card(ctx).expect("render"),
                variants,
            )
        });
    previews::check_all_with_variants(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail), ("explore.item", item)],
        vec![("home.card", home)],
        None,
        Vec::new(),
        variants,
    );
}
