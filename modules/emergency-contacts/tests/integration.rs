//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use emergency_contacts::{render_explore_detail, render_home_card, render_host_main};
use portaki_sdk::host::module::ModuleStatus;
use portaki_sdk::sdui::GeoPoint;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "SAMU" }, "phone": "15" },
            { "label": "Pompiers", "phone": "18" }
        ],
        "host_visible_phone": "+33 6 12 34 56 78"
    })
}

/// Chaque rangée mène par un pictogramme (§2.11).
///
/// L'hôte n'en avait pas — la ligne qu'on cherche en premier était la seule sans repère — et un
/// contact sans catégorie non plus, ce qui donnait une liste en dents de scie.
#[test]
#[serial]
fn every_row_leads_with_a_glyph() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "contacts": [{ "label": "Pharmacie de garde", "phone": "3237" }]
        }))
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(json.contains("\"leading\":\"users\""), "{json}");
            assert!(json.contains("\"leading\":\"phone\""), "{json}");
        });
}

#[test]
#[serial]
fn home_card_still_calls_the_country_numbers_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("\"112\""), "{json}");
        });
}

#[test]
#[serial]
fn home_card_renders_contacts() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // L'appel est porté par la ligne, pas par un `Pressable` autour : c'est ce que la
            // maquette donne à chaque rangée, et un niveau de moins à lire.
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
            assert!(json.contains("tel:"));
        });
}

#[test]
#[serial]
fn detail_includes_emergency_banner() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Pompiers"));
            // Le 112 est une tuile, plus un lien en bas de page : on compose avant de lire.
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            assert!(
                json.contains("\"title\":\"112\""),
                "le 112 est là, en tuile"
            );
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&{
            // Sur plages : les heures ne sont dessinées que là.
            let mut config = sample_config();
            config["host_availability"] = json!("hours");
            config
        })
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            // Les deux champs d'avant `host_hours` : lus, plus dessinés.
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &["host_hours_from", "host_hours_to"],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("+33 6 12 34 56 78"));
            assert!(json.contains("Pompiers"));
        });
}

/// The SDK renders the inactive state; the surface itself is not called.
#[test]
#[serial]
fn an_inactive_module_shows_the_sdk_state() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_module_status(ModuleStatus {
            active: false,
            workspace_enabled: true,
            incomplete: false,
            requires_config: true,
            missing_required_keys: Vec::new(),
        })
        .run(|ctx| {
            let surface = portaki_sdk::guest_shell::render(ctx, "home.card", render_home_card);
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

/// La ligne `rows` du tableau des contacts.
fn table_rows(surface: &portaki_sdk::sdui::surface::Surface) -> Vec<Value> {
    fn find(value: &Value) -> Option<Vec<Value>> {
        match value {
            Value::Object(o) if o.get("type") == Some(&json!("DataTable")) => {
                o.get("rows")?.as_array().cloned()
            }
            Value::Object(o) => o.values().find_map(find),
            Value::Array(items) => items.iter().find_map(find),
            _ => None,
        }
    }
    find(&serde_json::to_value(surface).unwrap()).expect("a DataTable")
}

/// Les contacts sont un tableau ordonnable : une ligne par contact saisi (les emplacements vides
/// de l'ancien formulaire n'en sont pas), « N / 20 », une fenêtre par ligne, et « Ajouter » tant
/// qu'on est sous 20 — la ligne demandée par « Ajouter » apparaît avec son id.
#[test]
#[serial]
fn the_contacts_are_a_sortable_table() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "contacts": [
                { "id": "paulette", "label": "Paulette", "phone": "+33698765432", "note": "Double des clés" },
                { "label": "", "phone": "" }
            ]
        }))
        .run(|mut ctx| {
            ctx.input = json!({ "newContacts": "neuf" });
            let surface = render_host_main(ctx).expect("host main");
            let rows = table_rows(&surface);
            assert_eq!(rows.len(), 2, "{rows:?}");
            assert_eq!(rows[0]["id"], "paulette");
            assert_eq!(rows[0]["note"], "Double des clés");
            assert_eq!(rows[1]["id"], "neuf");
            let json = serde_json::to_string(&surface).unwrap();
            assert!(json.contains(r#""sortable":true"#), "{json}");
            assert!(json.contains(r#""panel":"dialog""#), "{json}");
            assert!(json.contains(r#""subtitle":"2 / 20""#), "{json}");
            assert!(json.contains("contacts.1.phone"), "{json}");
            assert!(json.contains("newContacts"), "{json}");
        });

    let rows: Vec<Value> = (0..emergency_contacts::MAX_CONTACTS)
        .map(|i| json!({ "id": format!("c{i}"), "label": format!("Contact {i}"), "phone": "18" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "contacts": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains(r#""subtitle":"20 / 20""#), "{json}");
            assert!(
                !json.contains("newContacts"),
                "plus d'« Ajouter » à 20 : {json}"
            );
        });
}

/// « Même numéro que vous » : une pastille sur le contact qui répète le numéro affiché, sans rien
/// bloquer ; aucune quand l'hôte ne montre pas son numéro.
#[test]
#[serial]
fn a_contact_with_the_host_number_wears_a_pill() {
    for (show, expected) in [(true, "i18n:host.contact.duplicate"), (false, "")] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({
                "show_host": show,
                "host_visible_phone": "+33 6 12 34 56 78",
                "contacts": [
                    { "id": "julie", "label": "Julie", "phone": "+33612345678" },
                    { "id": "paulette", "label": "Paulette", "phone": "+33698765432" }
                ]
            }))
            .run(|ctx| {
                let rows = table_rows(&render_host_main(ctx).expect("host main"));
                assert_eq!(rows[0]["duplicate"], expected, "{rows:?}");
                assert_eq!(rows[1]["duplicate"], "", "{rows:?}");
            });
    }
}

/// Le bloc de l'hôte se masque avec « Afficher mon numéro », la plage avec « 24 h/24 » ; la
/// plage d'avant (deux champs) remplit le `TimeRange`.
#[test]
#[serial]
fn the_host_block_hides_with_its_switch() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "host_availability": "hours",
            "host_hours_from": "09:00",
            "host_hours_to": "20:00"
        }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains(r#""visibleWhen":"show_host=true""#), "{json}");
            assert!(
                json.contains(r#""visibleWhen":"host_availability=hours""#),
                "{json}"
            );
            assert!(json.contains(r#""layout":"segmented""#), "{json}");
            assert!(
                json.contains(r#""name":"host_hours","value":"09:00-20:00""#),
                "{json}"
            );
        });
}

/// A host writing in English: the French label and note stay, and so do the note, the category
/// and the id the form does not carry; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["contacts.label", "contacts.note"]
    );
    let stored = json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15",
              "note": { "fr": "Gratuit", "en": "Free" }, "category": "medical" },
            { "label": "", "phone": "" },
            { "id": "pompiers", "label": { "fr": "Pompiers" }, "phone": "18" }
        ],
        "host_visible_phone": "+33 6"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            // Stored order; the blank slot is not a row. The table carries the ids (the dashboard
            // sends `contacts.N.id` itself), the dialogs the fields.
            let rows = table_rows(&surface);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0]["id"], "samu");
            assert_eq!(rows[1]["id"], "pompiers");
            let sent = config_save::form_args(&surface);
            assert_eq!(sent["contacts"][0]["label"], "Ambulance");
            assert_eq!(sent["contacts"][1]["phone"], "18");

            let stored = json!({ "contacts": [stored["contacts"][0], stored["contacts"][2]] });
            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["contacts"][0], stored["contacts"][0]);
            assert_eq!(saved["contacts"][1]["label"]["fr"], "Pompiers");
        });
}

/// Le numéro de l'hôte vient de son profil quand il ne l'a pas saisi ici.
///
/// Le champ du module existait parce que la plateforme ne portait pas le téléphone de l'hôte ;
/// elle le porte depuis `ctx.host`. Un hôte qui a rempli son compte n'a plus à le retaper, et le
/// §2.16 ne lui demande que ses contacts, sa pharmacie et son hôpital.
#[test]
#[serial]
fn the_host_row_falls_back_to_the_platform_profile() {
    let config = json!({ "contacts": [{ "label": "Pompiers", "phone": "18" }] });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            assert!(json.contains("tel:+33612345678"), "{json}");
        });
}

/// Le numéro saisi ici gagne : c'est un choix délibéré, par exemple une ligne dédiée.
#[test]
#[serial]
fn a_number_the_host_typed_here_wins_over_the_profile() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33600000000".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            // Lu en paires à la française, composé en E.164.
            assert!(json.contains("\"06 12 34 56 78\""), "{json}");
            assert!(json.contains("tel:+33612345678"), "{json}");
            assert!(!json.contains("+33600000000"), "{json}");
        });
}

/// Sans contact ni numéro — ni ici, ni au profil — il reste le 112 (§2.16).
///
/// Ce module ne se tait jamais : « aucun contact hôte → tuiles pays seules, **jamais d'état
/// vide**, "composez le 112" minimum ». Les numéros d'urgence ne viennent pas de l'hôte, ils se
/// calculent du pays du logement — un hôte qui n'a rien rempli n'est pas une raison de laisser un
/// voyageur sans numéro.
#[test]
#[serial]
fn nothing_anywhere_still_shows_the_country_numbers() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("\"112\""), "{json}");
        });
}

/// Le profil seul suffit à ouvrir la carte : un logement sans contact saisi montre quand même
/// qui appeler.
#[test]
#[serial]
fn the_profile_alone_is_enough_to_show_the_card() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            assert!(serde_json::to_string(&surface)
                .expect("json")
                .contains("tel:+33612345678"));
        });
}

/// Les numéros d'urgence sont ceux du **logement**, pas de la langue du voyageur.
///
/// Le module lisait `ctx.locale`, qui est celle du lecteur dès qu'il a choisi une langue. Un
/// francophone en Espagne recevait donc le 15 et le 18 — qui ne sonnent nulle part là-bas — et un
/// anglophone en France perdait les deux. On ne compose pas un numéro mort en urgence.
#[test]
#[serial]
fn the_numbers_follow_the_property_not_the_reader() {
    for (property_locale, reader, expect_national) in
        [("fr-FR", "en-US", true), ("es-ES", "fr-FR", false)]
    {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_property(Property {
                locale: property_locale.into(),
                ..Property::default()
            })
            .run(|ctx| {
                let mut ctx = ctx;
                ctx.locale = reader.to_string();
                let json =
                    serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
                assert!(json.contains("\"112\""), "{json}");
                assert_eq!(
                    json.contains("\"15\""),
                    expect_national,
                    "{property_locale} lu en {reader} : {json}"
                );
                assert_eq!(json.contains("\"18\""), expect_national, "{json}");
            });
    }
}

/// La pharmacie et l'hôpital placés partent sur la Carte du livret ; sans position, rien.
#[test]
#[serial]
fn placed_health_places_reach_the_booklet_map() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "pharmacy": "Pharmacie du port",
            "pharmacy_lat": "43.58",
            "pharmacy_lng": 7.12,
            "hospital": "Hôpital d'Antibes"
        }))
        .run(|ctx| {
            let markers = emergency_contacts::map_markers(ctx)
                .expect("markers")
                .markers;
            let json = serde_json::to_string(&markers).unwrap();
            assert_eq!(markers.len(), 1, "{json}");
            assert!(json.contains("Pharmacie du port"), "{json}");
            // La catégorie range le repère dans le pratique, avec le pictogramme santé.
            assert_eq!(markers[0].category.as_deref(), Some("pharmacy"), "{json}");
        });
}

/// La distance du logement suit chaque place placée (§2.3 : « Rangée + distance ») : sur la
/// rangée qu'on appelle, et dans la phrase quand la place n'a pas de numéro. Sans position — de la
/// place ou du logement — pas de distance.
#[test]
#[serial]
fn placed_places_show_their_distance_from_the_property() {
    let config = json!({
        "pharmacy": "Pharmacie du port",
        "pharmacy_phone": "+33 4 93 34 00 00",
        "pharmacy_lat": 43.58,
        "pharmacy_lng": 7.12,
        "hospital": "Hôpital d'Antibes",
        "hospital_lat": 43.60,
        "hospital_lng": 7.11,
        "doctor": "Dr Martin",
        "doctor_phone": "+33 4 93 00 00 00"
    });
    let render = |home: Option<GeoPoint>| {
        let mut json = String::new();
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .with_coordinates(home)
            .run(|ctx| {
                json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                    .expect("json");
            });
        json
    };
    let placed = render(Some(GeoPoint::new(43.58, 7.12)));
    // La pharmacie est sur le logement : 50 m, jamais « 0 m ». Le médecin n'a pas de position.
    // (La phrase de l'hôpital, sans numéro, ne se lit pas ici : le `t!` du mock rend la clé seule.)
    assert!(
        placed.contains("04 93 34 00 00 · guest.distance.metres"),
        "{placed}"
    );
    assert_eq!(placed.matches("guest.distance").count(), 1, "{placed}");
    let unplaced = render(None);
    assert!(!unplaced.contains("guest.distance"), "{unplaced}");
    assert!(!unplaced.contains(" · "), "{unplaced}");
}

/// Les horaires de la pharmacie se lisent dans le livret : « Ouverte jusqu'à … » dans une plage,
/// « Fermée » en dehors, à l'heure du logement ; sans horaires, rien.
#[test]
#[serial]
fn the_pharmacy_hours_say_whether_it_is_open() {
    use chrono::TimeZone;
    let render = |hours: &str, now: chrono::DateTime<chrono::Utc>| {
        let mut json = String::new();
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({
                "pharmacy": "Pharmacie du Port",
                "pharmacy_phone": "+33297412210",
                "pharmacy_hours": hours
            }))
            .with_now(now)
            .run(|ctx| {
                json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            });
        json
    };
    // Lundi 12 octobre 2026.
    let monday = |h| chrono::Utc.with_ymd_and_hms(2026, 10, 12, h, 0, 0).unwrap();
    let week = "mon=09:00-19:30;tue=09:00-19:30";
    let open = render(week, monday(9));
    assert!(open.contains("guest.pharmacy.openUntil"), "{open}");
    let closed = render(week, monday(20));
    assert!(closed.contains("guest.pharmacy.closed"), "{closed}");
    let none = render("", monday(9));
    assert!(!none.contains("guest.pharmacy."), "{none}");
}

/// La pharmacie de garde (le 3237) est une rangée en France, même sans pharmacie saisie ; ailleurs,
/// rien. Le 114 se joint par SMS.
#[test]
#[serial]
fn the_on_duty_pharmacy_and_the_sms_number_follow_the_country() {
    for (locale, french) in [("fr-FR", true), ("es-ES", false)] {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_property(Property {
                locale: locale.into(),
                ..Property::default()
            })
            .run(|ctx| {
                let json =
                    serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
                assert_eq!(json.contains("guest.health.onDuty"), french, "{json}");
                assert_eq!(json.contains("tel:3237"), french, "{json}");
                assert_eq!(json.contains("sms:114"), french, "{json}");
                assert_eq!(json.contains("\"17\""), french, "{json}");
            });
    }
}
