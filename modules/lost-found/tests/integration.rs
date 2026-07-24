//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;
use uuid::Uuid;

use lost_found::{
    build_email_context, list_for_stay, list_recent, render_home_card, render_host_create,
    render_host_main, render_host_stay, reset_test_store, submit, submit_found, update_config,
    update_status, EmailContextArgs, ListForStayArgs, SubmitArgs, SubmitFoundArgs,
    UpdateConfigArgs, UpdateStatusArgs, STATUS_DEFAULT,
};
use portaki_sdk::prelude::EmailTemplateKey;
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
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::Select(_) if type_name == "Select" => true,
            Component::Pill(_) if type_name == "Pill" => true,
            Component::HeaderTitle(_) if type_name == "HeaderTitle" => true,
            Component::TextArea(_) if type_name == "TextArea" => true,
            Component::RichTextEditor(_) if type_name == "RichTextEditor" => true,
            Component::FieldHint(_) if type_name == "FieldHint" => true,
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
        Component::ListItem(inner) => inner.children.iter().collect(),
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
            assert!(json.contains("form.kind.label"));
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
                    kind: "lost".into(),
                    item_description: "Blue scarf".into(),
                    contact_hint: Some("guest@example.com".into()),
                    details: Some("Left in living room".into()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].kind, "lost");
            assert_eq!(rows[0].item_description, "Blue scarf");
            assert_eq!(rows[0].status, STATUS_DEFAULT);

            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "found".into(),
                    item_description: "Room key".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit second");

            let rows =
                list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list after second");
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
                    kind: "found".into(),
                    item_description: "Umbrella".into(),
                    contact_hint: None,
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
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "InfoBanner"));
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "List"));
            assert!(contains_component_type(&surface, "ListItem"));
            assert!(contains_component_type(&surface, "Pill"));
            assert!(contains_component_type(&surface, "Select"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.banner"));
            assert!(json.contains("host.main.recentTitle"));
            assert!(json.contains("updateStatus") || json.contains("host.main.updateStatus"));
            // Create lives on stay surface — not the property editor.
            assert!(!json.contains("host.create.submit"));
        });
}

#[test]
#[serial]
fn host_update_status_changes_report() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![],
                    stay_id: Some(stay_id),
                    description: "Chargeur USB-C".into(),
                    status: Some("to_collect".into()),
                },
            )
            .expect("submitFound");

            let rows = list_for_stay(
                ctx.clone(),
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list");
            assert_eq!(rows.len(), 1);
            let report_id = rows[0].id;

            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id,
                    status: "sent".into(),
                },
            )
            .expect("updateStatus");

            let rows = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list after update");
            assert_eq!(rows[0].status, "sent");
        });
}

#[test]
#[serial]
fn email_context_includes_descriptions_when_declaration_exists() {
    reset_test_store();

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: "Écharpe bleue".into(),
                    contact_hint: None,
                    details: None,
                },
            )
            .expect("submit");

            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::LostFound),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert!(out.has_declaration);
            assert_eq!(out.lost_item_description.as_deref(), Some("Écharpe bleue"));
        });
}

#[test]
#[serial]
fn host_submit_found_creates_report_per_stay() {
    reset_test_store();
    let stay_a = Uuid::new_v4();
    let stay_b = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![stay_a, stay_b],
                    stay_id: None,
                    description: "Chargeur MacBook oublié dans le tiroir".into(),
                    status: None,
                },
            )
            .expect("submitFound");

            let for_a = list_for_stay(
                ctx.clone(),
                ListForStayArgs {
                    stay_id: Some(stay_a),
                },
            )
            .expect("list a");
            assert_eq!(for_a.len(), 1);
            assert_eq!(for_a[0].kind, "found");
            assert_eq!(for_a[0].status, STATUS_DEFAULT);

            let for_b = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_b),
                },
            )
            .expect("list b");
            assert_eq!(for_b.len(), 1);
            assert_eq!(for_b[0].status, "to_collect");
        });
}

#[test]
#[serial]
fn host_submit_found_always_defaults_status_to_collect() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            submit_found(
                ctx.clone(),
                SubmitFoundArgs {
                    stay_ids: vec![],
                    stay_id: Some(stay_id),
                    description: r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Doudou"}]}]}"#.into(),
                    status: Some("sent".into()),
                },
            )
            .expect("submitFound");

            let rows = list_for_stay(
                ctx,
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("list");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].status, STATUS_DEFAULT);
            assert!(rows[0].item_description.contains("Doudou"));
        });
}

#[test]
#[serial]
fn host_create_surface_renders_declare_form() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({
                "stayId": stay_id.to_string(),
                "guestName": "Marie Dupont",
                "stayDates": "12–15 juil.",
            });
            let surface = render_host_create(ctx);
            assert!(contains_component_type(&surface, "Page"));
            assert!(contains_component_type(&surface, "Form"));
            assert!(contains_component_type(&surface, "TextArea"));
            assert!(contains_component_type(&surface, "FieldHint"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("submitFound") || json.contains("host.create.submit"));
            assert!(json.contains("host.create.description.label") || json.contains("description"));
            // Modal chrome owns title / Annuler — form body has no HeaderTitle.
            assert!(!json.contains("HeaderTitle"));
            assert!(!json.contains("host.main.status.label"));
        });
}

#[test]
#[serial]
fn host_stay_surface_is_list_only() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx);
            assert!(contains_component_type(&surface, "Page"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.empty") || json.contains("host.stay.listTitle"));
            assert!(!json.contains("submitFound"));
            assert!(!json.contains("host.create.submit"));
            assert!(!json.contains("TextArea"));
        });
}

#[test]
#[serial]
fn host_main_editor_has_note_not_create_form() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(contains_component_type(&surface, "InfoBanner"));
            assert!(contains_component_type(&surface, "RichTextEditor"));
            assert!(contains_component_type(&surface, "EmptyState"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.hostNote.label") || json.contains("host_note"));
            assert!(json.contains("host.main.banner"));
            assert!(!json.contains("host.create.submit"));
            assert!(!json.contains("submitFound"));
        });
}

#[test]
#[serial]
fn host_note_shows_on_guest_card_and_email_context() {
    reset_test_store();
    let config_bytes = serde_json::to_vec(&serde_json::json!({
        "host_note": "Leave found items in the lobby closet."
    }))
    .expect("config json");

    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[portaki_sdk::capability::core::STORAGE])
        .with_kv("config", config_bytes)
        .run(|ctx| {
            let surface = render_home_card(ctx.clone());
            assert!(contains_component_type(&surface, "InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Leave found items in the lobby closet."));

            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::LostFound),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert_eq!(
                out.checkout_tips.as_deref(),
                Some("Leave found items in the lobby closet.")
            );
        });
}

#[test]
#[serial]
fn update_config_persists_host_note() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[portaki_sdk::capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    host_note: "Lobby closet.".into(),
                },
            )
            .expect("updateConfig");

            let surface = render_host_main(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Lobby closet."));
        });
}
