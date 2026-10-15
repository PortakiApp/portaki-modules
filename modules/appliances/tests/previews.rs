//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use appliances::{
    render_explore_detail, render_explore_item, render_home_card, reset_test_store, save_appliance,
    ApplianceStatus, SaveApplianceArgs,
};
use portaki_sdk::prelude::Context;
use serde_json::{json, Value};

fn paragraph(text: &str) -> Value {
    json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] })
}

fn doc(blocks: Vec<Value>) -> String {
    json!({ "type": "doc", "content": blocks }).to_string()
}

/// Trois appareils courants d'une location, dont un seul mis en avant.
fn seed(ctx: Context) {
    let devices = [
        SaveApplianceArgs {
            id: Some("plaques".into()),
            name: "Plaques à induction".into(),
            emoji: "🍳".into(),
            description: doc(vec![
                paragraph("Appuyez 2 secondes sur la touche marche, puis choisissez le foyer."),
                paragraph("Seules les casseroles à fond aimanté chauffent."),
            ]),
            featured: true,
            order: Some(0),
            location: "Cuisine".into(),
            manual_url: "https://example.com/notice-plaques.pdf".into(),
            safety_note: "La surface reste chaude quelques minutes après l'arrêt.".into(),
            status: ApplianceStatus::Active,
            model: String::new(),
            video_url: String::new(),
            // Une étape avec sa photo : la fiche d'un appareil montre ce que l'hôte a photographié.
            steps: Some(json!([
                {
                    "text": "Posez une casserole à fond aimanté au centre du foyer.",
                    "image": "portaki-file:plaques-induction-1"
                },
                { "text": "Appuyez 2 secondes sur la touche marche, puis choisissez la puissance." }
            ])),
        },
        SaveApplianceArgs {
            id: Some("lave-linge".into()),
            name: "Lave-linge".into(),
            emoji: "🌀".into(),
            description: doc(vec![paragraph(
                "Programme « Coton 40° » pour un lavage courant, lessive dans le placard du dessous.",
            )]),
            featured: false,
            order: Some(1),
            location: "Salle de bain".into(),
            manual_url: String::new(),
            safety_note: "Merci de ne pas lancer de machine après 22 h.".into(),
            status: ApplianceStatus::Active,
            model: String::new(),
            video_url: String::new(),
            steps: None,
        },
        SaveApplianceArgs {
            id: Some("tv".into()),
            name: "Télévision".into(),
            emoji: "📺".into(),
            description: doc(vec![paragraph(
                "Allumez avec la télécommande noire ; la source « HDMI 1 » donne accès aux applications.",
            )]),
            featured: false,
            order: Some(2),
            location: "Salon".into(),
            manual_url: String::new(),
            safety_note: String::new(),
            status: ApplianceStatus::Active,
            model: String::new(),
            video_url: String::new(),
            steps: None,
        },
    ];
    for device in devices {
        save_appliance(ctx.clone(), device).expect("save appliance");
    }
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    // La démo installe un module qui reçoit les signalements : la fiche montre donc son bouton,
    // ouvert au chemin que ce module déclare pour `report.issue`.
    let reporter: portaki_sdk::host::module::ModulePeer =
        serde_json::from_value(json!({ "moduleId": "issue-report", "path": "issue-report/form" }))
            .expect("peer");
    let (detail, item, home, variants) = previews::guest(root)
        .with_peers(appliances::REPORT_ISSUE, vec![reporter])
        .run(|ctx| {
            seed(ctx.clone());
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
