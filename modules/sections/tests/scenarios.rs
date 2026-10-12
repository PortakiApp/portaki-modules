//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use sections::{reset_test_store, save_section, SaveSectionArgs, SectionLocaleInput};

/// Deux sections éditoriales, l'une traduite, l'autre en français seulement.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    builder.clone().run(|ctx| {
        let sections = [
            vec![
                ("fr", "Bienvenue", "Bienvenue à **L'Islette** !"),
                ("en", "Welcome", "Welcome to **L'Islette**!"),
            ],
            vec![("fr", "L'appartement", "2 chambres, terrasse vue mer.")],
        ];
        for (order, locales) in sections.into_iter().enumerate() {
            let args = SaveSectionArgs {
                id: None,
                sort_order: Some(order as i32),
                locales: locales
                    .into_iter()
                    .map(|(lang, title, body)| SectionLocaleInput {
                        lang: lang.into(),
                        title: title.into(),
                        body_markdown: body.into(),
                    })
                    .collect(),
                title: String::new(),
                body_markdown: String::new(),
                lang: String::new(),
            };
            save_section(ctx.clone(), args).expect("save section");
        }
    });
    builder
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        setup,
        // Des identifiants de sections qu'un compte neuf n'a pas.
        &["deleteSection", "reorder"],
    );
}
