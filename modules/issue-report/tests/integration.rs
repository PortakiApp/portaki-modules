//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use issue_report::{
    list_for_stay, list_recent, render_home_card, render_host_main, reset_test_store, submit,
    SubmitArgs,
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
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::List(_) if type_name == "List" => true,
            Component::Pill(_) if type_name == "Pill" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::Stack(_) if type_name == "Stack" => true,
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
        Component::Card(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
        Component::List(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_renders_form_when_no_reports() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("form.category.label"));
        });
}

#[test]
#[serial]
fn submit_allows_multiple_reports_and_shows_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    category: "appliance".into(),
                    summary: "Oven broken".into(),
                    details: Some("Won't heat".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].category, "appliance");
            assert_eq!(rows[0].summary, "Oven broken");

            submit(
                ctx.clone(),
                SubmitArgs {
                    category: "noise".into(),
                    summary: "Loud neighbors".into(),
                    details: None,
                },
            )
            .expect("submit second");

            let rows = list_for_stay(ctx.clone()).expect("list after second");
            assert_eq!(rows.len(), 2);

            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
            assert!(contains_component_type(&surface, "ListItem"));
        });
}

#[test]
#[serial]
fn host_main_lists_recent_after_guest_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    category: "cleanliness".into(),
                    summary: "Bathroom not clean".into(),
                    details: None,
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let recent = list_recent(ctx.clone()).expect("listRecent");
            assert_eq!(recent.len(), 1);

            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "Page"));
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "List"));
            assert!(contains_component_type(&surface, "ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.banner"));
            assert!(json.contains("host.main.status.open"));
            assert!(json.contains("danger-triangle") || json.contains("sparkles"));
        });
}
