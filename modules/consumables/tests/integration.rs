//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;
use uuid::Uuid;

use consumables::{
    list_for_stay, list_items, list_open_count, render_home_card, render_host_main,
    render_host_stats, render_host_stay, replace_items, reset_test_store, seed_defaults, submit,
    update_config, update_status, ConsumableItemInput, ListForStayArgs, ReplaceItemsArgs,
    SubmitArgs, UpdateConfigArgs, UpdateStatusArgs, LEVEL_DEFAULT, STATUS_DEFAULT,
};
use portaki_sdk::prelude::EmptyArgs;
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::Text(_) if type_name == "Text" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::IndexedInput(_) if type_name == "IndexedInput" => true,
            Component::Grid(_) if type_name == "Grid" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::ChoiceList(_) if type_name == "ChoiceList" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::List(_) if type_name == "List" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::Pill(_) if type_name == "Pill" => true,
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
        Component::Field(inner) => inner.children.iter().collect(),
        Component::List(inner) => inner.children.iter().collect(),
        Component::ListItem(inner) => inner.children.iter().collect(),
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
fn home_card_renders_form_with_catalog() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Café".into(),
                        label_en: "Coffee".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "ChoiceList"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("Café") || json.contains("Coffee"));
        });
}

#[test]
#[serial]
fn submit_creates_open_report_and_lists_on_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Papier toilette".into(),
                        label_en: "Toilet paper".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");

            let items = list_items(ctx.clone()).expect("list items");
            assert_eq!(items.len(), 1);
            let item_id = items[0].id;

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_id,
                    level: LEVEL_DEFAULT.into(),
                    note: Some("Salle de bain".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].status, STATUS_DEFAULT);
            assert_eq!(rows[0].level, LEVEL_DEFAULT);
            assert!(rows[0].item_label.contains("Papier") || rows[0].item_label.contains("Toilet"));

            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
        });
}

#[test]
#[serial]
fn host_mark_restocked_clears_open_list() {
    reset_test_store();
    let mut report_id = Uuid::nil();

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        label: String::new(),
                        label_fr: "Savon".into(),
                        label_en: "Soap".into(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                    items_json: None,
                },
            )
            .expect("replace");
            let item_id = list_items(ctx.clone()).expect("items")[0].id;
            submit(
                ctx.clone(),
                SubmitArgs {
                    item_id,
                    level: "low".into(),
                    note: None,
                },
            )
            .expect("submit");
            report_id = list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0].id;
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let open = list_open_count(ctx.clone()).expect("open count");
            assert_eq!(open.open_count, 1);

            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id,
                    status: "restocked".into(),
                },
            )
            .expect("restock");

            let open = list_open_count(ctx.clone()).expect("open after");
            assert_eq!(open.open_count, 0);

            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "IndexedInput"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.emptyRecent"));
        });
}

#[test]
#[serial]
fn seed_defaults_fills_empty_catalog() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed");
            let items = list_items(ctx.clone()).expect("items");
            assert_eq!(items.len(), 8);

            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed again");
            assert_eq!(list_items(ctx).expect("items").len(), 8);
        });
}

#[test]
#[serial]
fn update_config_replaces_catalog() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    items: vec![ConsumableItemInput {
                        label: "Coffee pods".into(),
                        label_fr: String::new(),
                        label_en: String::new(),
                        sort_order: 0,
                        low_threshold: 0,
                    }],
                },
            )
            .expect("updateConfig");
            let items = list_items(ctx).expect("items");
            assert_eq!(items.len(), 1);
        });
}

#[test]
#[serial]
fn host_main_and_stats_render() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let main = render_host_main(ctx.clone());
            assert!(contains_component_type(&main, "Page"));
            assert!(contains_component_type(&main, "IndexedInput"));
            assert!(contains_component_type(&main, "InfoBanner"));
            assert!(contains_component_type(&main, "Button"));

            let stats = render_host_stats(ctx);
            assert!(contains_component_type(&stats, "Card"));
            let json = serde_json::to_string(&stats).expect("stats json");
            assert!(json.contains("stats.catalog"));
            assert!(json.contains("stats.open"));
        });
}

#[test]
#[serial]
fn host_stay_empty_when_no_reports() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx);
            assert!(contains_component_type(&surface, "Page"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(!json.contains("host.stay.listTitle"));
            assert!(!contains_component_type(&surface, "Card"));
        });
}

#[test]
#[serial]
fn submit_rejects_unknown_item() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let err = submit(
                ctx,
                SubmitArgs {
                    item_id: Uuid::new_v4(),
                    level: "missing".into(),
                    note: None,
                },
            );
            assert!(err.is_err());
        });
}
