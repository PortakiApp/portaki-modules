//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use checklist::{
    complete_item, list_completions, list_items, render_home_card, render_host_main, replace_items,
    reset_test_store, uncomplete_item, update_config, ChecklistItemInput, ItemIdArgs,
    ReplaceItemsArgs, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::Text(_) if type_name == "Text" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::ChecklistItem(_) if type_name == "ChecklistItem" => true,
            Component::IndexedInput(_) if type_name == "IndexedInput" => true,
            Component::Grid(_) if type_name == "Grid" => true,
            Component::Pressable(_) if type_name == "Pressable" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
            Component::Button(_) if type_name == "Button" => true,
            _ => false,
        };
        if matches {
            return true;
        }
        for child in child_components(node) {
            if walk(child, type_name) {
                return true;
            }
        }
        false
    }
    walk(&surface.root, type_name)
}

fn child_components(node: &Component) -> Vec<&Component> {
    match node {
        Component::Stack(inner) => inner.children.iter().collect(),
        Component::Grid(inner) => inner.children.iter().collect(),
        Component::Card(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        Component::Pressable(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_empty_when_no_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.empty"));
        });
}

#[test]
#[serial]
fn home_card_renders_toggles_with_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![
                        ChecklistItemInput {
                            label: String::new(),
                            label_fr: "Fermer les volets".into(),
                            label_en: "Close shutters".into(),
                            sort_order: 0,
                        },
                        ChecklistItemInput {
                            label: String::new(),
                            label_fr: "Sortir les poubelles".into(),
                            label_en: "Take out bins".into(),
                            sort_order: 1,
                        },
                    ],
                    items_json: None,
                },
            )
            .expect("replace");

            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "ChecklistItem"));
            assert!(contains_component_type(&surface, "Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Fermer les volets") || json.contains("Close shutters"));
            assert!(json.contains("completeItem"));
        });
}

#[test]
#[serial]
fn complete_and_uncomplete_roundtrip() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ChecklistItemInput {
                        label: String::new(),
                        label_fr: "Clés".into(),
                        label_en: "Keys".into(),
                        sort_order: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let items = list_items(ctx.clone()).expect("list");
            assert_eq!(items.len(), 1);
            let item_id = items[0].id;

            complete_item(ctx.clone(), ItemIdArgs { item_id }).expect("complete");
            let done = list_completions(ctx.clone()).expect("completions");
            assert_eq!(done, vec![item_id]);

            uncomplete_item(ctx.clone(), ItemIdArgs { item_id }).expect("uncomplete");
            let done = list_completions(ctx).expect("completions after uncomplete");
            assert!(done.is_empty());
        });
}

#[test]
#[serial]
fn host_main_renders_form() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "Page"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Grid"));
            assert!(contains_component_type(&surface, "IndexedInput"));
            // Workspace header Save owns persistence — no in-form Enregistrer.
            assert!(!contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.item.empty"));
            // Empty list → length + 1 = one slot only.
            assert!(json.contains("items.0.label"));
            assert!(!json.contains("items.1.label"));
        });
}

#[test]
#[serial]
fn host_main_shows_filled_plus_one_slot() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ChecklistItemInput {
                        label: "Fermer les volets".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let json = serde_json::to_string(&render_host_main(ctx)).expect("surface json");
            assert!(json.contains("items.0.label"));
            assert!(json.contains("items.1.label"));
            assert!(!json.contains("items.2.label"));
        });
}

#[test]
#[serial]
fn update_config_replaces_items_from_form() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        update_config(
            ctx.clone(),
            UpdateConfigArgs {
                items: vec![
                    ChecklistItemInput {
                        label: "Clés".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                    },
                    ChecklistItemInput {
                        label: String::new(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 1,
                    },
                ],
            },
        )
        .expect("updateConfig");

        let items = list_items(ctx).expect("list");
        assert_eq!(items.len(), 1);
    });
}
