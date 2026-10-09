//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use issue_report::{
    render_guest_form, render_home_card, reset_test_store, submit, Category, SubmitArgs,
};

/// Un signalement déjà envoyé, pour que la carte montre son suivi.
#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    // Les aperçus sont pris la veille de l'arrivée, et le formulaire ne s'ouvre par défaut que
    // pendant le séjour : la démo est un logement qui l'a ouvert à toutes les périodes, sans quoi
    // elle ne montrerait que « contactez votre hôte ».
    let (card, form) = previews::guest(root)
        .with_config(&serde_json::json!({
            "phase_before": true, "phase_during": true, "phase_after": true
        }))
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    category: Category::Appliance,
                    summary: "Le four ne chauffe plus".into(),
                    details: Some("Le voyant s'allume mais la température ne monte pas.".into()),
                    photo: None,
                },
            )
            .expect("submit");
            (
                render_home_card(ctx.clone()).expect("guest surface"),
                render_guest_form(ctx).expect("guest surface"),
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
