//! Ce que le règlement fait de plus que la collection du SDK : la carte, la page, le modèle,
//! l'éditeur, les e-mails. La fusion, l'import et les contrôles sont ceux du SDK, testés là-bas.

mod sample;

use portaki_sdk::prelude::*;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use rules::{apply_template, render_explore_detail, render_home_card, render_host_main, Rules};
use serde_json::json;

#[test]
fn the_home_card_lists_the_first_rules_and_offers_the_rest() {
    let surface = sample::seeded(MockContext::guest()).run(|ctx| render_home_card(ctx).unwrap());
    let tree = serde_json::to_value(&surface).unwrap().to_string();
    assert_eq!(SurfaceAssertions::new(&surface).count_type("ListItem"), 4);
    assert!(
        tree.contains("Calme après 22 h") && tree.contains("22:00 – 08:00"),
        "{tree}"
    );
    assert!(
        SurfaceAssertions::new(&surface).contains_type("Button"),
        "five rules, four shown"
    );
}

#[test]
fn the_card_limit_is_the_hosts() {
    let surface = MockContext::guest()
        .with_resource("rules", sample::rules(), json!({ "card_limit": 2 }))
        .run(|ctx| render_home_card(ctx).unwrap());
    assert_eq!(SurfaceAssertions::new(&surface).count_type("ListItem"), 2);
}

#[test]
fn the_page_groups_by_theme_in_the_hosts_order() {
    let surface =
        sample::seeded(MockContext::guest()).run(|ctx| render_explore_detail(ctx).unwrap());
    let tree = serde_json::to_value(&surface).unwrap().to_string();
    assert!(
        tree.contains("rule.theme.noise") && tree.contains("rule.theme.pets"),
        "{tree}"
    );
    assert_eq!(SurfaceAssertions::new(&surface).count_type("ListItem"), 5);
}

#[test]
fn no_rule_is_the_empty_state() {
    let surface = MockContext::guest().run(|ctx| render_home_card(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
}

#[test]
fn the_template_fills_an_empty_rulebook_once() {
    MockContext::host()
        .with_translation("host.template.silence.title", "Calme après 22 h")
        .run(|ctx| {
            apply_template(ctx.clone(), EmptyArgs {}).unwrap();
            let rules = Rules::load(&ctx).unwrap();
            assert_eq!(rules.len(), 5);
            assert_eq!(rules.rows()[0].title.get("fr"), "Calme après 22 h");
            apply_template(ctx.clone(), EmptyArgs {}).unwrap();
            assert_eq!(
                Rules::load(&ctx).unwrap().len(),
                5,
                "a second click writes nothing"
            );
        });
}

#[test]
fn the_editor_marks_the_status_and_draws_the_card_line() {
    let surface = sample::seeded(MockContext::host()).run(|ctx| render_host_main(ctx).unwrap());
    let tree = serde_json::to_value(&surface).unwrap().to_string();
    assert!(tree.contains(r#""glyph":"!""#), "{tree}");
    assert!(tree.contains(r#""dividerAfter":4"#), "{tree}");
    assert!(tree.contains(r#""name":"rules.4.title""#), "{tree}");
}

#[test]
fn the_arrival_email_carries_three_rules() {
    let vars = sample::seeded(MockContext::guest()).run(|ctx| {
        rules::email_vars(
            ctx,
            EmailContextArgs {
                template_key: Some(EmailTemplateKey::Arrival),
                ..Default::default()
            },
        )
        .unwrap()
    });
    let text = vars
        .get(EmailVar::HouseRulesTeaser)
        .unwrap_or_default()
        .to_string();
    assert!(
        text.contains("Calme après 22 h — Merci de penser au voisinage"),
        "{text}"
    );
    assert!(!text.contains("Animaux"), "three rules only");
}
