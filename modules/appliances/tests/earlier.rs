//! La reprise des appareils d'avant : des lignes `appliances_content` telles que l'ancien module
//! les a écrites (capturées depuis son code sur `main`, `tests/earlier/*.json`), relues une fois.

use appliances::{render_explore_detail, render_explore_item, Devices};
use portaki_test_utils::MockContext;
use serde_json::{json, Value};

fn stored(name: &str) -> Value {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_multilingual_bundle_becomes_the_collection() {
    MockContext::guest()
        .with_rows("AppliancesContent", vec![stored("bundle")])
        .run(|ctx| {
            let devices = Devices::load(&ctx).unwrap();
            let rows: Vec<(&str, &str, &str, bool)> = devices
                .rows()
                .iter()
                .map(|d| (d.id.as_str(), d.name.get("fr"), d.name.get("en"), d.hidden))
                .collect();
            assert_eq!(
                rows,
                [
                    ("tv", "Télévision", "TV", false),
                    ("plaques", "Plaques à induction", "Induction hob", false),
                    ("lave-linge", "Lave-linge", "Washing machine", false),
                    ("seche-linge", "Sèche-linge", "Dryer", true),
                ]
            );
            let hob = &devices.rows()[1];
            assert!(hob.featured);
            assert_eq!(hob.location.get("en"), "Kitchen");
            assert_eq!(hob.model, "Bosch PIE631");
            assert_eq!(hob.manual_file, "portaki-file:notice-plaques");
            assert_eq!(hob.video_url, "https://youtu.be/abc123");
            assert_eq!(
                hob.safety_note.get("en"),
                "The surface stays hot after use."
            );
            assert!(hob.description.get("en").contains("Only magnetic pans"));
            let steps: Vec<(&str, &str, &str)> = hob
                .steps
                .iter()
                .map(|s| (s.text.get("fr"), s.text.get("en"), s.image.as_str()))
                .collect();
            assert_eq!(
                steps,
                [
                    (
                        "Posez la casserole au centre du foyer.",
                        "Put the pan in the middle of the zone.",
                        "portaki-file:plaques-1"
                    ),
                    (
                        "Appuyez 2 secondes sur la touche marche.",
                        "Hold the power key for 2 seconds.",
                        ""
                    ),
                ]
            );
            // Un mode d'emploi vide (`{ paragraph }` seul) n'est pas repris comme un texte.
            assert!(devices.rows()[0].description.is_blank());
            let meta = devices.meta();
            assert_eq!(meta.featured_limit, 3);
            assert_eq!(
                meta.room_order,
                ["Salon", "Cuisine", "Living room", "Kitchen"]
            );
            assert_eq!(
                meta.paper_manuals_location.get("en"),
                "Red box, living-room shelf"
            );
        });
}

#[test]
fn the_booklet_shows_what_it_showed_in_each_language() {
    let list = |locale: &str| {
        MockContext::guest()
            .with_locale(locale)
            .with_rows("AppliancesContent", vec![stored("bundle")])
            .run(|ctx| serde_json::to_string(&render_explore_detail(ctx).unwrap()).unwrap())
    };
    let en = list("en-US");
    let (living, kitchen) = (en.find("Living room").unwrap(), en.find("Kitchen").unwrap());
    assert!(living < kitchen, "{en}");
    assert!(
        !en.contains("Dryer"),
        "a hidden appliance stays hidden: {en}"
    );
    let fr = list("fr-FR");
    assert!(
        fr.find("Salon").unwrap() < fr.find("Cuisine").unwrap(),
        "{fr}"
    );
}

#[test]
fn the_per_language_format_keeps_order_and_hidden() {
    MockContext::guest()
        .with_rows("AppliancesContent", vec![stored("dual")])
        .run(|ctx| {
            let devices = Devices::load(&ctx).unwrap();
            let rows: Vec<(&str, &str, bool)> = devices
                .rows()
                .iter()
                .map(|d| (d.id.as_str(), d.name.get("en"), d.hidden))
                .collect();
            assert_eq!(
                rows,
                [("clim", "Air conditioning", true), ("four", "Oven", false)]
            );
            assert_eq!(devices.meta().featured_limit, 0, "0 is the default card");
        });
}

#[test]
fn the_first_format_is_converted_as_the_old_module_read_it() {
    MockContext::guest()
        .with_rows("AppliancesContent", vec![stored("v1")])
        .run(|ctx| {
            let devices = Devices::load(&ctx).unwrap();
            let rows: Vec<(&str, &str, &str, &str)> = devices
                .rows()
                .iter()
                .map(|d| {
                    let (id, emoji) = (d.id.as_str(), d.emoji.as_str());
                    (id, d.name.get("en"), d.location.get("en"), emoji)
                })
                .collect();
            assert_eq!(
                rows,
                [
                    ("device-1", "TV", "Living room", "📺"),
                    ("device-2", "Internet box", "", ""),
                ]
            );
            assert_eq!(devices.rows()[1].manual_url, "https://example.com/box.pdf");
            let mut ctx = ctx;
            ctx.input = json!({ "deviceId": "device-1" });
            let item = serde_json::to_string(&render_explore_item(ctx).unwrap()).unwrap();
            assert!(
                item.contains("Allumez avec la télécommande noire"),
                "{item}"
            );
            assert!(item.contains("Source HDMI 1"), "{item}");
        });
}

#[test]
fn taken_over_once_and_never_over_the_collection() {
    MockContext::host()
        .with_rows("AppliancesContent", vec![stored("bundle")])
        .with_resource(
            "devices",
            json!([{ "id": "x", "name": { "fr": "Déjà là" } }]),
            json!({}),
        )
        .run(|ctx| assert_eq!(Devices::load(&ctx).unwrap().len(), 1));
    MockContext::host()
        .with_rows("AppliancesContent", vec![stored("bundle")])
        .run(|ctx| {
            let mut devices = Devices::load(&ctx).unwrap();
            devices.rows_mut().clear();
            Devices::save(&devices).unwrap();
            assert!(
                Devices::load(&ctx).unwrap().is_empty(),
                "an emptied guide stays empty"
            );
        });
}
