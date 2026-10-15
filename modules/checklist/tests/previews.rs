//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use checklist::{
    create_checklist, list_checklists, render_home_card, render_post_stay_card, reset_test_store,
    update_config, CreateChecklistArgs, UpdateConfigArgs,
};
use serial_test::serial;

/// La liste de départ que le module propose en un clic : celle qu'un hôte obtient sans rien
/// écrire, et donc celle qu'un aperçu doit montrer.
fn seed(ctx: &portaki_sdk::Context) {
    create_checklist(
        ctx.clone(),
        CreateChecklistArgs {
            template: "departure".into(),
            id: None,
        },
    )
    .expect("create");
}

#[test]
#[serial]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let context = previews::guest(root);
    let post_stay = context.run(|ctx| {
        seed(&ctx);
        render_post_stay_card(ctx).expect("post-stay")
    });
    /*
     * La carte d'accueil, rendue la veille du départ : la liste ouverte et cochable, « Départ
     * demain à 11:00 ».
     *
     * <p>À l'heure des aperçus — la veille d'une arrivée — la liste n'est pas ouverte, et la carte
     * se masque (un état vide sans texte). Le catalogue n'aurait rien montré du module, et la démo
     * « pendant le séjour » non plus : elle prend ce rendu en phase `stay`.
     *
     * <p>Réglée sur « Pendant tout le séjour », un choix d'hôte du formulaire : la liste reste
     * ouverte quelle que soit l'horloge de la phase `stay`, sans dépendre du jour où elle s'ouvre
     * par défaut.
     */
    reset_test_store();
    let during = || {
        previews::guest_at(root, previews::DURING).run(|ctx| {
            seed(&ctx);
            let list = list_checklists().expect("lists")[0].id;
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    id: list.to_string(),
                    trigger: "duringStay".into(),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("trigger");
            render_home_card(ctx).expect("card")
        })
    };
    let card = during();
    reset_test_store();
    let stay_card = during();
    /*
     * La carte d'après séjour, rendue après le départ.
     *
     * <p>À l'heure des aperçus — la veille d'une arrivée — elle dit « pas encore », et la démo du
     * livret servait ce rendu-là même quand sa clé demandait l'après. Les listes de départ restent
     * pourtant cochables une fois le séjour fini.
     *
     */
    reset_test_store();
    let after = previews::guest_at(root, previews::AFTER).run(|ctx| {
        seed(&ctx);
        render_post_stay_card(ctx).expect("post-stay")
    });

    previews::check_all_phased(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("home.card", card), ("post-stay.card", post_stay)],
        // Les deux surfaces ont un chemin : rien à ajouter pour la démo.
        Vec::new(),
        None,
        vec![
            ("stay", vec![("home.card", stay_card)]),
            ("post-stay", vec![("post-stay.card", after)]),
        ],
    );
}
