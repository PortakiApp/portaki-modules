//! Ce que les appareils font de plus que la collection du SDK : la carte, la liste par pièce, la
//! fiche, les règles de publication, l'éditeur. La fusion et l'import sont ceux du SDK.

mod sample;

use appliances::{
    __portaki_devices_publish_readiness, render_explore_detail, render_explore_item,
    render_home_card, render_host_main, stats_summary,
};
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_sdk::prelude::*;
use portaki_test_utils::{MockContext, MockContextBuilder, SurfaceAssertions};
use serde_json::{json, Value};

fn tree(surface: &Surface) -> String {
    serde_json::to_string(surface).unwrap()
}

fn item(builder: MockContextBuilder, id: &str) -> String {
    builder.run(|mut ctx| {
        ctx.input = json!({ "deviceId": id });
        tree(&render_explore_item(ctx).unwrap())
    })
}

fn readiness(devices: Value, meta: Value) -> Vec<String> {
    sample::with(MockContext::host(), devices, meta).run(|ctx| {
        __portaki_devices_publish_readiness(ctx)
            .unwrap()
            .items
            .into_iter()
            .map(|i| i.id)
            .collect()
    })
}

#[test]
fn no_appliance_is_the_empty_state() {
    let surface = MockContext::guest().run(|ctx| render_home_card(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
}

#[test]
fn the_card_tiles_the_featured_and_counts_them_all() {
    let card = sample::seeded(MockContext::guest()).run(|ctx| render_home_card(ctx).unwrap());
    assert_eq!(SurfaceAssertions::new(&card).count_type("ListItem"), 1);
    let card = tree(&card);
    assert!(
        card.contains("Plaques à induction") && !card.contains("Télévision"),
        "{card}"
    );
    assert!(card.contains("home.card.seeAll"), "{card}");
}

#[test]
fn nothing_featured_shows_the_first_ones() {
    let devices =
        json!([ { "id": "a", "name": { "fr": "Four" } }, { "id": "b", "name": { "fr": "Télé" } } ]);
    let card = sample::with(
        MockContext::guest(),
        devices,
        json!({ "featured_limit": 2 }),
    )
    .run(|ctx| render_home_card(ctx).unwrap());
    assert_eq!(SurfaceAssertions::new(&card).count_type("ListItem"), 2);
    assert!(tree(&card).contains("🔥"), "the name proposes the emoji");
}

#[test]
fn the_list_is_one_card_per_room_in_the_hosts_order() {
    let list = sample::with(
        MockContext::guest(),
        sample::devices(),
        json!({ "room_order": ["Salon", "Cuisine"] }),
    )
    .run(|ctx| tree(&render_explore_detail(ctx).unwrap()));
    let salon = list.find("\"title\":\"Salon\"").unwrap();
    let cuisine = list.find("\"title\":\"Cuisine\"").unwrap();
    assert!(salon < cuisine, "{list}");
}

#[test]
fn a_hidden_or_unnamed_appliance_stays_off_the_booklet() {
    let devices = json!([
        { "id": "a", "name": { "fr": "Four" }, "hidden": true },
        { "id": "b", "name": { "fr": "" }, "emoji": "📺" },
        { "id": "c", "name": { "fr": "Bouilloire" } }
    ]);
    let list = sample::with(MockContext::guest(), devices, json!({}))
        .run(|ctx| tree(&render_explore_detail(ctx).unwrap()));
    assert!(list.contains("Bouilloire") && !list.contains("Four") && !list.contains("📺"));
}

#[test]
fn the_item_shows_safety_steps_with_their_diagram_and_the_manuals() {
    let page = item(sample::seeded(MockContext::guest()), "plaques");
    assert!(
        page.contains("InfoBanner") && page.contains("La surface reste chaude"),
        "{page}"
    );
    assert!(page.contains("portaki-file:plaques-induction-1") && page.contains("\"index\":2"));
    assert!(page.contains("explore.item.manuals") && page.contains("notice-plaques.pdf"));
    assert!(
        !page.contains("explore.item.contactHost"),
        "no report module, no button"
    );
}

#[test]
fn without_steps_the_free_text_lists_become_steps() {
    let description = json!({ "type": "doc", "content": [ { "type": "orderedList", "content": [
        { "type": "listItem", "content": [ { "type": "paragraph", "content": [ { "type": "text", "text": "Ouvrir le hublot" } ] } ] }
    ] } ] }).to_string();
    let devices = json!([ { "id": "l", "name": { "fr": "Lave-linge" }, "description": { "fr": description } } ]);
    let page = item(sample::with(MockContext::guest(), devices, json!({})), "l");
    assert!(
        page.contains("\"title\":\"Ouvrir le hublot\"") && page.contains("\"index\":1"),
        "{page}"
    );
}

#[test]
fn no_manual_at_all_says_so_and_the_pdf_wins_over_the_link() {
    let devices = json!([
        { "id": "a", "name": { "fr": "Four" } },
        { "id": "b", "name": { "fr": "Télé" }, "manual_file": "portaki-file:notice", "manual_url": "https://example.com/n" }
    ]);
    let page = item(
        sample::with(MockContext::guest(), devices.clone(), json!({})),
        "a",
    );
    assert!(page.contains("explore.item.noManual") && !page.contains("explore.item.howto"));
    let page = item(sample::with(MockContext::guest(), devices, json!({})), "b");
    assert!(
        page.contains("portaki-file:notice") && !page.contains("example.com/n"),
        "{page}"
    );
}

#[test]
fn an_unknown_appliance_is_not_found() {
    let page = item(sample::seeded(MockContext::guest()), "nope");
    assert!(page.contains("explore.item.notFound"));
}

#[test]
fn a_report_module_adds_the_button() {
    let reporter: portaki_sdk::host::module::ModulePeer =
        serde_json::from_value(json!({ "moduleId": "issue-report", "path": "issue-report/form" }))
            .unwrap();
    let builder =
        sample::seeded(MockContext::guest()).with_peers(appliances::REPORT_ISSUE, vec![reporter]);
    let page = item(builder, "tv");
    assert!(page.contains("issue-report/form") && page.contains("\"category\":\"appliance\""));
}

#[test]
fn a_video_the_booklet_cannot_play_blocks_publication_and_stays_off() {
    let devices =
        json!([ { "id": "a", "name": { "fr": "Four" }, "video_url": "https://example.com/v" } ]);
    assert!(readiness(devices.clone(), json!({})).contains(&"resources.devices.a.video_url".into()));
    let page = item(sample::with(MockContext::guest(), devices, json!({})), "a");
    assert!(!page.contains("explore.item.video"));
}

#[test]
fn more_featured_than_the_card_shows_blocks_publication() {
    let devices = json!([
        { "id": "a", "name": { "fr": "Four" }, "featured": true },
        { "id": "b", "name": { "fr": "Télé" }, "featured": true },
        { "id": "c", "name": { "fr": "Bouilloire" }, "featured": true }
    ]);
    let ids = readiness(devices, json!({ "featured_limit": 2 }));
    assert!(
        ids.contains(&"resources.devices.featured_limit".into()),
        "{ids:?}"
    );
}

#[test]
fn the_editor_is_the_home_card_then_the_table_with_steps() {
    let surface = sample::seeded(MockContext::host()).run(|ctx| render_host_main(ctx).unwrap());
    let page = tree(&surface);
    for wanted in [
        "host.section.homeCard",
        r#""kind":"check""#,
        r#""name":"🍳 Plaques à induction""#,
        r#""name":"devices.0.steps.1.text""#,
        r#""name":"devices.0.steps.0.id""#,
        r#""accept":"application/pdf""#,
        "host.search.placeholder",
    ] {
        assert!(page.contains(wanted), "{wanted}");
    }
    assert!(
        !page.contains("host.manual.missing"),
        "every sample has a manual"
    );
}

#[test]
fn the_stats_tile_counts_the_manuals_opened() {
    let summary = MockContext::host().run(|ctx| {
        let args: StatsSummaryArgs = serde_json::from_value(json!({
            "propertyId": "5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10", "period": 30, "key": "appliances-views",
            "views": { "opens": 9, "surfaces": { "explore.item": 7 } }
        }))
        .unwrap();
        stats_summary(ctx, args).unwrap()
    });
    assert_eq!(serde_json::to_value(summary).unwrap()["value"], "7");
}
