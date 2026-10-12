//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;
use uuid::Uuid;

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};

use sections::{
    delete_section, list_sections, render_explore_sheet, render_home_card, render_host_main,
    reorder, reset_test_store, save_section, DeleteSectionArgs, ListSectionsArgs, ReorderArgs,
    SaveSectionArgs, SectionLocaleInput,
};

fn seed_two_sections(ctx: portaki_sdk::Context) -> (Uuid, Uuid) {
    let first = save_section(
        ctx.clone(),
        SaveSectionArgs {
            id: None,
            sort_order: Some(0),
            locales: vec![
                SectionLocaleInput {
                    lang: "fr".into(),
                    title: "Bienvenue".into(),
                    body_markdown: "Bienvenue à L'Islette !".into(),
                },
                SectionLocaleInput {
                    lang: "en".into(),
                    title: "Welcome".into(),
                    body_markdown: "Welcome to L'Islette!".into(),
                },
            ],
            title: String::new(),
            body_markdown: String::new(),
            lang: String::new(),
        },
    )
    .expect("save first");
    let second = save_section(
        ctx,
        SaveSectionArgs {
            id: None,
            sort_order: Some(1),
            locales: vec![SectionLocaleInput {
                lang: "fr".into(),
                title: "L'appartement".into(),
                body_markdown: "2 chambres, terrasse vue mer.".into(),
            }],
            title: String::new(),
            body_markdown: String::new(),
            lang: String::new(),
        },
    )
    .expect("save second");
    (first.id, second.id)
}

#[test]
#[serial]
fn home_card_empty_without_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_and_sheet_with_sections() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_sections(ctx.clone());
            let card = render_home_card(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
            assert!(SurfaceAssertions::new(&card).contains_type("Markdown"));
            let sheet = render_explore_sheet(ctx).expect("render");
            assert!(SurfaceAssertions::new(&sheet).contains_type("Markdown"));
            let json = serde_json::to_string(&sheet).expect("json");
            assert!(json.contains("L'appartement"));
        });
}

#[test]
#[serial]
fn host_main_master_detail() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_sections(ctx.clone());
            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("List"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            assert!(SurfaceAssertions::new(&surface).contains_type("RichTextEditor"));
            assert!(SurfaceAssertions::new(&surface).contains_type("FieldHint"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.list.add"));
            assert!(json.contains("host.body.hint"));
            assert!(json.contains("Bienvenue"));
        });
}

#[test]
#[serial]
fn list_reorder_delete() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let (a, b) = seed_two_sections(ctx.clone());
            let listed = list_sections(
                ctx.clone(),
                ListSectionsArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("list");
            assert_eq!(listed.len(), 2);
            assert_eq!(listed[0].id, a);

            reorder(
                ctx.clone(),
                ReorderArgs {
                    ordered_ids: vec![b, a],
                },
            )
            .expect("reorder");
            let listed = list_sections(
                ctx.clone(),
                ListSectionsArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("list after reorder");
            assert_eq!(listed[0].id, b);

            delete_section(ctx.clone(), DeleteSectionArgs { id: b }).expect("delete");
            let listed = list_sections(
                ctx,
                ListSectionsArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("list after delete");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].id, a);
        });
}
