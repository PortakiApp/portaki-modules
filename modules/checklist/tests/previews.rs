//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

mod sample;

use checklist::{render_home_card, render_post_stay_card};
use serial_test::serial;

/// La liste de départ que le module propose en un clic : celle qu'un hôte obtient sans rien
/// écrire, et donc celle qu'un aperçu doit montrer.
fn departure() -> serde_json::Value {
    sample::lists(&["departure"])
}

#[test]
#[serial]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root);
    let post_stay = sample::with(context, departure())
        .run(|ctx| render_post_stay_card(ctx).expect("post-stay"));
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
    let during = || {
        let mut lists = departure();
        lists[0]["guest_trigger"] = "duringStay".into();
        sample::with(previews::guest_at(root, previews::DURING), lists)
            .run(|ctx| render_home_card(ctx).expect("card"))
    };
    let card = during();
    let stay_card = during();
    /*
     * La carte d'après séjour, rendue après le départ.
     *
     * <p>À l'heure des aperçus — la veille d'une arrivée — elle dit « pas encore », et la démo du
     * livret servait ce rendu-là même quand sa clé demandait l'après. Les listes de départ restent
     * pourtant cochables une fois le séjour fini.
     *
     */
    let after = sample::with(previews::guest_at(root, previews::AFTER), departure())
        .run(|ctx| render_post_stay_card(ctx).expect("post-stay"));

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
