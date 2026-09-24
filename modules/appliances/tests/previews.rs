//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use appliances::{
    render_explore_detail, render_explore_item, reset_test_store, save_appliance, ApplianceStatus,
    SaveApplianceArgs,
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
    let (detail, item) = previews::guest(root).run(|ctx| {
        seed(ctx.clone());
        let mut item_ctx = ctx.clone();
        item_ctx.input = json!({ "deviceId": "plaques" });
        (render_explore_detail(ctx), render_explore_item(item_ctx))
    });
    previews::check(
        root,
        vec![("explore.detail", detail), ("explore.item", item)],
    );
}
