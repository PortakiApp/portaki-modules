//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use portaki_sdk::capability;
use sections::{
    render_explore_sheet, render_home_card, reset_test_store, save_section, SaveSectionArgs,
    SectionLocaleInput,
};
use serial_test::serial;

/// Le mot que l'hôte écrit à ses voyageurs, sur le logement d'exemple du livret.
fn seed(ctx: portaki_sdk::Context) {
    save_section(
        ctx,
        SaveSectionArgs {
            id: None,
            sort_order: Some(0),
            locales: vec![
                SectionLocaleInput {
                    lang: "fr".into(),
                    title: "Bienvenue à L'Islette".into(),
                    body_markdown: "Nous sommes heureux de vous accueillir.\n\nFaites comme chez vous : le café est dans le placard au-dessus de la machine, et les serviettes de plage sont dans le coffre de l'entrée.".into(),
                },
                SectionLocaleInput {
                    lang: "en".into(),
                    title: "Welcome to L'Islette".into(),
                    body_markdown: "We are glad to have you.\n\nMake yourself at home: the coffee is in the cupboard above the machine, and the beach towels are in the hallway chest.".into(),
                },
            ],
            title: String::new(),
            body_markdown: String::new(),
            lang: String::new(),
        },
    )
    .expect("save");
}

/// Aucune surface n'a de chemin : le module n'entre pas au catalogue des aperçus, mais le livret
/// montre son mot dès l'accueil — c'est donc la démo qui en a besoin.
#[test]
#[serial]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let context = previews::guest(root).with_capabilities(&[capability::core::STORAGE]);
    let (card, sheet) = context.run(|ctx| {
        seed(ctx.clone());
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
