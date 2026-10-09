//! Integration-style unit tests with `portaki-test-utils`.

use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

use consumables::{
    list_for_stay, list_items, list_open_count, publish_readiness, render_guest_form,
    render_home_card, render_host_main, render_host_stats, render_host_stay, replace_items,
    reset_test_store, seed_defaults, stats_summary, submit, update_config, update_status,
    ConsumableItemInput, ListForStayArgs, ReplaceItemsArgs, SubmitArgs, UpdateConfigArgs,
    UpdateStatusArgs, GUEST_TEXT_EMAIL_MAX_CHARS, LEVEL_DEFAULT, STATUS_DEFAULT,
};
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_sdk::limits;
use portaki_sdk::prelude::EmptyArgs;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};

#[test]
#[serial]
fn home_card_empty_when_no_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.empty"));
        });
}

#[test]
#[serial]
fn home_card_opens_form_overlay_with_catalog() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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

            let surface = render_home_card(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("guest.form"));
            assert!(json.contains("package"));
            assert!(json.contains("home.card.openForm"));

            let form = render_guest_form(ctx).expect("render");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("ChoiceList"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("Café") || form_json.contains("Coffee"));
        });
}

/// Le §2.5 veut une grille à choix multiple : deux produits manquants font un seul envoi.
#[test]
#[serial]
fn one_submit_reports_every_product_the_guest_picked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![
                        ConsumableItemInput {
                            emoji: String::new(),
                            label: String::new(),
                            label_fr: "Papier toilette".into(),
                            label_en: "Toilet paper".into(),
                            sort_order: 0,
                            low_threshold: 0,
                        },
                        ConsumableItemInput {
                            emoji: String::new(),
                            label: String::new(),
                            label_fr: "Café".into(),
                            label_en: "Coffee".into(),
                            sort_order: 1,
                            low_threshold: 0,
                        },
                    ],
                    items_json: None,
                },
            )
            .expect("replace");

            let items = list_items(ctx.clone()).expect("list items");
            assert_eq!(items.len(), 2);

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: vec![items[0].id, items[1].id],
                    item_id: None,
                    level: LEVEL_DEFAULT.into(),
                    note: None,
                },
            )
            .expect("submit");

            // Un signalement par produit, et non un pour le lot : l'hôte clôt l'un sans clore
            // l'autre, et chaque e-mail garde le lien vers le sien.
            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows.len(), 2);
        });
}

/// Un formulaire déjà ouvert dans un téléphone envoie encore l'ancien champ au moment du déploiement.
#[test]
#[serial]
fn the_single_product_field_is_still_accepted() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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
            let item_id = list_items(ctx.clone()).expect("items")[0].id;

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: Vec::new(),
                    item_id: Some(item_id),
                    level: LEVEL_DEFAULT.into(),
                    note: None,
                },
            )
            .expect("submit");

            assert_eq!(
                list_for_stay(ctx, ListForStayArgs::default())
                    .expect("list")
                    .len(),
                1
            );
        });
}

/// Rien n'est coché d'avance : une case présélectionnée partirait au signalement sans qu'on le veuille.
#[test]
#[serial]
fn the_product_grid_starts_with_nothing_picked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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

            let form = render_guest_form(ctx).expect("render");
            let json = serde_json::to_string(&form).expect("json");

            assert!(json.contains("\"layout\":\"grid\""), "{json}");
            assert!(json.contains("\"multi\":true"), "{json}");
            assert!(json.contains("\"layout\":\"segmented\""), "{json}");
            // La liste des produits ne porte pas de `value` : aucune tuile n'est cochée.
            assert!(
                !json.contains("\"name\":\"itemIds\",\"type\":\"ChoiceList\",\"value\""),
                "{json}"
            );
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
                        emoji: String::new(),
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
                    item_ids: Vec::new(),
                    item_id: Some(item_id),
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

            let surface = render_home_card(ctx).expect("render");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
        });
}

fn stock_args() -> StatsSummaryArgs {
    StatsSummaryArgs {
        property_id: Uuid::nil(),
        period: 30,
        key: "stock".into(),
    }
}

#[test]
#[serial]
fn host_mark_restocked_clears_open_list() {
    reset_test_store();
    let mut report_id = Uuid::nil();
    let mut stay_id = Uuid::nil();
    let mut item_id = Uuid::nil();

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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
            item_id = list_items(ctx.clone()).expect("items")[0].id;
            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: Vec::new(),
                    item_id: Some(item_id),
                    level: "low".into(),
                    note: None,
                },
            )
            .expect("submit");
            let report = &list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0];
            report_id = report.id;
            stay_id = report.stay_id;
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let open = list_open_count(ctx.clone()).expect("open count");
            assert_eq!(open.open_count, 1);
            let tile = stats_summary(ctx.clone(), stock_args()).expect("tile");
            assert_eq!(tile.value, "1");
            assert!(tile.attention.is_none(), "running low is no stock-out");

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
            assert_eq!(
                stats_summary(ctx.clone(), stock_args())
                    .expect("tile")
                    .value,
                "0"
            );
            let stats = serde_json::to_string(&render_host_stats(ctx.clone())).expect("json");
            assert!(stats.contains("stats.stock.ok"));
            assert!(stats.contains("stats.row.restockedAt"));
            assert!(stats.contains("/modules/consumables"));
            // Sans séjours : les signalements par article ; un seul réassort, pas encore de rythme.
            assert!(stats.contains("stats.byItem.title"));
            assert!(stats.contains("stats.restock.empty"));

            // Deux départs dans la période, dont celui qui a signalé le savon : 50 %.
            let mut with_stays = ctx.clone();
            let now = portaki_sdk::host::time::now().expect("now");
            let stay = |id: Uuid, days: i64| {
                serde_json::json!({
                    "id": id,
                    "checkIn": now - chrono::Duration::days(days + 3),
                    "checkOut": now - chrono::Duration::days(days),
                    "status": "COMPLETED",
                })
            };
            with_stays.input = serde_json::json!({
                "periodDays": 30,
                "stays": [stay(stay_id, 0), stay(Uuid::new_v4(), 4)],
            });
            let stats = serde_json::to_string(&render_host_stats(with_stays)).expect("json");
            assert!(stats.contains("stats.perStay.title"));
            assert!(stats.contains("\"50 %\""));

            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("IndexedInput"));
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

/// L'emoji suit son produit, et un champ vide ne l'efface pas (§2.5).
///
/// Vide, il ne l'efface pas parce que l'hôte qui renomme un produit dans une autre langue ne doit
/// pas perdre le pictogramme qu'il avait choisi — le formulaire ne porte qu'une langue à la fois.
#[test]
#[serial]
fn an_emoji_follows_its_product_and_an_empty_field_does_not_erase_it() {
    reset_test_store();
    let save = |emoji: &str, label: &str| ConsumableItemInput {
        emoji: emoji.into(),
        label: label.into(),
        label_fr: String::new(),
        label_en: String::new(),
        sort_order: 0,
        low_threshold: 0,
    };
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    requests_enabled: None,
                    max_requests: None,
                    restock_delay: Default::default(),
                    items: vec![save("☕", "Café")],
                },
            )
            .expect("premier enregistrement");
            // Même produit, nouveau nom, emoji laissé vide par le formulaire.
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    requests_enabled: None,
                    max_requests: None,
                    restock_delay: Default::default(),
                    items: vec![save("", "Coffee")],
                },
            )
            .expect("second enregistrement");
            let form = serde_json::to_string(&render_guest_form(ctx).expect("formulaire")).unwrap();
            assert!(form.contains("☕"), "{form}");
            // L'emoji remplace le colis du vocabulaire, il ne s'y ajoute pas.
            assert!(!form.contains("\"icon\":\"package\""), "{form}");
        });
}

/// Le délai de l'hôte remplace la phrase générique, et un délai effacé la ramène (§2.5).
///
/// Deux contextes parce que le KV du bac à sable est propre à chacun : l'écriture se vérifie là où
/// elle a lieu, la lecture sur une entrée posée d'avance.
#[test]
#[serial]
fn the_host_s_restocking_time_reaches_the_guest() {
    reset_test_store();
    let item = || ConsumableItemInput {
        emoji: String::new(),
        label: "Café".into(),
        label_fr: String::new(),
        label_en: String::new(),
        sort_order: 0,
        low_threshold: 0,
    };

    // Écriture : l'hôte enregistre, et son propre écran le relit.
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    requests_enabled: None,
                    max_requests: None,
                    restock_delay: serde_json::from_value(json!({ "fr": "sous 24 h" }))
                        .expect("délai"),
                    items: vec![item()],
                },
            )
            .expect("enregistrement");
            let host = serde_json::to_string(&render_host_main(ctx.clone())).unwrap();
            assert!(host.contains("sous 24 h"), "{host}");

            // Effacé, le réglage disparaît — l'hôte doit pouvoir reprendre sa promesse.
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    requests_enabled: None,
                    max_requests: None,
                    restock_delay: Default::default(),
                    items: vec![item()],
                },
            )
            .expect("effacement");
            let host = serde_json::to_string(&render_host_main(ctx)).unwrap();
            assert!(!host.contains("sous 24 h"), "{host}");
        });

    // Lecture : le bandeau du voyageur dit le délai au lieu de la phrase générique.
    let stored = serde_json::to_vec(&json!({ "fr": "sous 24 h" })).expect("octets");
    MockContext::guest()
        .with_property(Property::default())
        .with_kv("restock_delay", stored)
        .with_translation("form.notice.delay", "Votre hôte réapprovisionne sous 24 h.")
        .run(|ctx| {
            let form = serde_json::to_string(&render_guest_form(ctx).expect("formulaire")).unwrap();
            assert!(
                form.contains("Votre hôte réapprovisionne sous 24 h."),
                "{form}"
            );
            assert!(!form.contains("\"i18n:form.notice\""), "{form}");
        });

    // Sans réglage, la phrase générique : le voyageur sait quand même que son envoi part.
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let form = serde_json::to_string(&render_guest_form(ctx).expect("formulaire")).unwrap();
            assert!(form.contains("i18n:form.notice"), "{form}");
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
                    requests_enabled: None,
                    max_requests: None,
                    restock_delay: Default::default(),
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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
            assert!(SurfaceAssertions::new(&main).contains_type("Page"));
            assert!(SurfaceAssertions::new(&main).contains_type("IndexedInput"));
            assert!(SurfaceAssertions::new(&main).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&main).contains_type("Button"));

            let stats = render_host_stats(ctx);
            let json = serde_json::to_string(&stats).expect("stats json");
            assert!(json.contains("stats.emptyHint"), "empty catalog");
        });
}

#[test]
#[serial]
fn host_stay_empty_state_when_no_reports() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.listTitle"));
            assert!(json.contains("host.stay.empty"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("List"));
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
                    item_ids: Vec::new(),
                    item_id: Some(Uuid::new_v4()),
                    level: "missing".into(),
                    note: None,
                },
            );
            assert!(err.is_err());
        });
}

/// A 20 000-char guest note: the report keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_note_is_stored_whole_and_quoted_in_the_host_email() {
    reset_test_store();
    let note = format!("{}!", "serviette ".repeat(2_000).trim_end());
    assert_eq!(note.chars().count(), 20_000);

    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
                        label: String::new(),
                        label_fr: "Serviettes".into(),
                        label_en: "Towels".into(),
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
                    item_ids: Vec::new(),
                    item_id: Some(item_id),
                    level: LEVEL_DEFAULT.into(),
                    note: Some(note.clone()),
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone(), ListForStayArgs::default()).expect("list");
            assert_eq!(rows[0].note.as_deref(), Some(note.as_str()));

            let email = host.sent_emails().into_iter().last().expect("host email");
            for (body, prefix) in [
                (&email.content.body.fr, "Précision : "),
                (&email.content.body.en, "Note: "),
            ] {
                assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
                let quoted = body
                    .split("\n\n")
                    .find(|part| part.ends_with('…'))
                    .expect("quoted note");
                let kept = quoted
                    .trim_end_matches('…')
                    .strip_prefix(prefix)
                    .expect("note prefix");
                assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
                assert!(note.starts_with(kept));
            }

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());
        });
}

/// Un catalogue vide ne bloque pas la publication (spec §9 n° 1) : le livret montre son état vide.
#[test]
#[serial]
fn an_empty_catalog_does_not_block_publication() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            assert!(publish_readiness(ctx.clone())
                .expect("readiness")
                .items
                .is_empty());
            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed");
            assert!(publish_readiness(ctx).expect("readiness").items.is_empty());
        });
}

fn named_item(emoji: &str, label: &str) -> ConsumableItemInput {
    ConsumableItemInput {
        emoji: emoji.into(),
        label: label.into(),
        label_fr: String::new(),
        label_en: String::new(),
        sort_order: 0,
        low_threshold: 0,
    }
}

fn save(
    ctx: &portaki_sdk::prelude::Context,
    items: Vec<ConsumableItemInput>,
    restock_delay: &str,
    max_requests: Option<f64>,
) {
    update_config(
        ctx.clone(),
        UpdateConfigArgs {
            requests_enabled: None,
            max_requests,
            restock_delay: serde_json::from_value(json!({ "fr": restock_delay })).expect("délai"),
            items,
        },
    )
    .expect("enregistrement");
}

/// Le message français de `config.<field>`, et la même erreur sous le champ du formulaire.
fn blocking(ctx: &portaki_sdk::prelude::Context, field: &str) -> Option<String> {
    let readiness = publish_readiness(ctx.clone()).expect("readiness");
    let check = readiness
        .items
        .into_iter()
        .find(|check| check.id == format!("config.{field}"))?;
    assert!(!check.ok);
    assert_eq!(
        check.level,
        portaki_sdk::contracts::publish::PublishLevel::Required
    );
    let fr = check.hint.get("fr").to_string();
    let host = serde_json::to_string(&render_host_main(ctx.clone())).expect("json");
    assert!(host.contains(&format!("\"error\":\"{fr}\"")), "{host}");
    Some(fr)
}

#[test]
#[serial]
fn more_than_thirty_products_blocks_publication() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let items = (1..=31)
                .map(|n| named_item("", &format!("Produit {n}")))
                .collect();
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items,
                    items_json: None,
                },
            )
            .expect("replace");
            let readiness = publish_readiness(ctx).expect("readiness");
            let check = readiness
                .items
                .iter()
                .find(|check| check.id == "config.items")
                .expect("config.items");
            assert_eq!(check.hint.get("fr"), "30 produits au maximum.");
        });
}

/// Un emoji sans nom reste enregistré et dit « Donnez un nom au produit. » ; le voyageur ne voit
/// pas de tuile vide.
#[test]
#[serial]
fn a_product_without_a_name_is_flagged_and_hidden_from_the_guest() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            save(
                &ctx,
                vec![named_item("☕", ""), named_item("🧻", "Papier")],
                "",
                None,
            );
            assert_eq!(
                blocking(&ctx, "items.0.label").as_deref(),
                Some("Donnez un nom au produit.")
            );
        });
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let json = serde_json::to_string(&render_guest_form(ctx).expect("render")).unwrap();
            assert!(json.contains("🧻"), "{json}");
            assert!(!json.contains("☕"), "{json}");
        });
}

#[test]
#[serial]
fn a_product_name_past_forty_characters_is_flagged() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            save(&ctx, vec![named_item("", &"c".repeat(41))], "", None);
            assert_eq!(
                blocking(&ctx, "items.0.label").as_deref(),
                Some("40 caractères au maximum.")
            );
        });
}

#[test]
#[serial]
fn a_restock_delay_past_forty_characters_is_flagged() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            save(&ctx, vec![named_item("", "Café")], &"d".repeat(41), None);
            assert_eq!(
                blocking(&ctx, "restock_delay").as_deref(),
                Some("40 caractères au maximum.")
            );
        });
}

/// Hors de 1 à 20, la valeur est gardée et l'erreur affichée — plus de bornage silencieux.
#[test]
#[serial]
fn max_requests_out_of_range_is_reported_not_clamped() {
    for typed in [0.0, 25.0] {
        reset_test_store();
        MockContext::host()
            .with_property(Property::default())
            .run(|ctx| {
                save(&ctx, vec![named_item("", "Café")], "", Some(typed));
                assert_eq!(
                    blocking(&ctx, "max_requests").as_deref(),
                    Some("Entre 1 et 20.")
                );
                save(&ctx, vec![named_item("", "Café")], "", Some(5.0));
                assert_eq!(blocking(&ctx, "max_requests"), None);
            });
    }
}

/// La grille s'ouvre d'une case quand les huit sont prises, et un neuvième produit déjà stocké
/// s'affiche au lieu de disparaître.
///
/// La grille de huit est le dessin et elle reste ; elle n'est plus un plafond.
#[test]
#[serial]
fn the_catalog_grid_grows_past_its_eight_tiles() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        let json = serde_json::to_string(&render_host_main(ctx)).expect("json");
        assert!(json.contains("items.7.label"), "{json}");
        assert!(!json.contains("items.8.label"), "{json}");
        // « Ajouter » demande la neuvième case.
        assert!(json.contains(r#""items_count":9"#), "{json}");
    });
}

/// Un produit déjà signalé pendant ce séjour le dit sur sa tuile (§2.5).
///
/// Sans cela, le voyageur qui rouvre le formulaire ne voit aucune différence, resignale le même
/// paquet de café, et l'hôte se déplace deux fois. Une fois le produit réapprovisionné, la mention
/// disparaît : le redemander redevient légitime.
#[test]
#[serial]
fn an_already_reported_product_says_so_until_it_is_restocked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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
            let item_id = list_items(ctx.clone()).expect("items")[0].id;

            let before = serde_json::to_string(&render_guest_form(ctx.clone()).expect("form"))
                .expect("json");
            assert!(!before.contains("form.item.reported"), "{before}");

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: vec![item_id],
                    item_id: None,
                    level: "low".into(),
                    note: None,
                },
            )
            .expect("submit");

            let after = serde_json::to_string(&render_guest_form(ctx.clone()).expect("form"))
                .expect("json");
            assert!(after.contains("form.item.reported.low"), "{after}");
            assert!(!after.contains("form.item.reported.missing"), "{after}");
        });
}

/// Le plafond du séjour : une demande passe, la suivante est refusée et la carte le dit.
#[test]
#[serial]
fn the_stay_cap_stops_requests() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_kv("settings", br#"{"max_requests":1}"#.to_vec())
        .run(|ctx| {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items: vec![ConsumableItemInput {
                        emoji: String::new(),
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
            let item_id = list_items(ctx.clone()).expect("list items")[0].id;
            let ask = |ctx: &portaki_sdk::prelude::Context| {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        item_ids: Vec::new(),
                        item_id: Some(item_id),
                        level: LEVEL_DEFAULT.into(),
                        note: None,
                    },
                )
            };
            ask(&ctx).expect("first request");
            assert!(ask(&ctx).is_err(), "the second goes past the cap");
            let json = serde_json::to_string(&render_home_card(ctx).expect("render")).unwrap();
            assert!(json.contains("home.card.requestsLimit"), "{json}");
            assert!(!json.contains("home.card.openForm"), "{json}");
        });
}
