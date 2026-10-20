//! Integration-style unit tests with `portaki-test-utils`.

use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

mod sample;

use consumables::{
    list_for_stay, list_open_count, render_guest_form, render_home_card, render_host_main,
    render_host_stats, render_host_stay, reset_test_store, seed_defaults, stats_summary, submit,
    task_complete, timeline_tasks, update_status, ListForStayArgs, SubmitArgs, UpdateStatusArgs,
    GUEST_TEXT_EMAIL_MAX_CHARS, LEVEL_DEFAULT, STATUS_DEFAULT,
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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Café", "Coffee", "")]) }),
            );

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
            sample::save(&ctx, json!({ "products": sample::products(&[("Papier toilette", "Toilet paper", ""), ("Café", "Coffee", "")]) }));

            let items = consumables::Products::load(&ctx).unwrap().rows().to_vec();
            assert_eq!(items.len(), 2);

            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: vec![items[0].uuid(), items[1].uuid()],
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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Papier toilette", "Toilet paper", "")]) }),
            );
            let item_id = sample::id(0);

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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Papier toilette", "Toilet paper", "")]) }),
            );

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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Papier toilette", "Toilet paper", "")]) }),
            );

            let items = consumables::Products::load(&ctx).unwrap().rows().to_vec();
            assert_eq!(items.len(), 1);
            let item_id = items[0].uuid();

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
        views: None,
    }
}

#[test]
#[serial]
fn host_mark_restocked_clears_open_list() {
    const ITEMS: [(&str, &str, &str); 1] = [("Savon", "Soap", "")];
    reset_test_store();
    let mut report_id = Uuid::nil();
    let mut stay_id = Uuid::nil();
    let mut item_id = Uuid::nil();

    sample::with(MockContext::guest(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            item_id = sample::id(0);
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

    sample::with(MockContext::host(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            let open = list_open_count(ctx.clone()).expect("open count");
            assert_eq!(open.open_count, 1);
            // « Demandes de réassort · 1 », « Savon en tête ».
            let tile = stats_summary(ctx.clone(), stock_args()).expect("tile");
            assert_eq!(tile.value, "1");
            assert_eq!(tile.label.fr, "demandes de réassort");
            assert_eq!(
                tile.attention.as_ref().map(|a| a.text.fr.as_str()),
                Some("Savon en tête")
            );
            assert_eq!(
                tile.attention.as_ref().map(|a| a.text.en.as_str()),
                Some("Soap on top")
            );

            update_status(
                ctx.clone(),
                UpdateStatusArgs {
                    report_id,
                    status: "restocked".into(),
                    host_reply: None,
                },
            )
            .expect("restock");

            let open = list_open_count(ctx.clone()).expect("open after");
            assert_eq!(open.open_count, 0);
            // Une demande livrée reste une demande de la période.
            assert_eq!(
                stats_summary(ctx.clone(), stock_args())
                    .expect("tile")
                    .value,
                "1"
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

            let surface = render_host_main(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("DataTable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            // Livrée, la demande reste dans la table et quitte « À traiter ».
            assert!(json.contains("i18n:status.restocked"), "{json}");
            assert!(!json.contains("host.received.toHandle"), "{json}");
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
            let items = consumables::Products::load(&ctx).unwrap().rows().to_vec();
            assert_eq!(items.len(), 6);

            seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed again");
            assert_eq!(
                consumables::Products::load(&ctx)
                    .unwrap()
                    .rows()
                    .to_vec()
                    .len(),
                6
            );
        });
}

/// Le délai de l'hôte remplace la phrase générique ; sans délai, « dans la journée » (§2.2, §2.5).
#[test]
#[serial]
fn the_host_s_restocking_time_reaches_the_guest() {
    reset_test_store();
    let delay = json!({ "restock_delay": { "fr": "sous 24 h" } });
    let form = sample::with_meta(MockContext::guest(), &[("Café", "", "")], delay)
        .with_translation("form.notice.delay", "Votre hôte réapprovisionne sous 24 h.")
        .run(|ctx| serde_json::to_string(&render_guest_form(ctx).expect("formulaire")).unwrap());
    assert!(
        form.contains("Votre hôte réapprovisionne sous 24 h."),
        "{form}"
    );
    assert!(!form.contains("\"i18n:form.notice\""), "{form}");

    let form = sample::with(MockContext::guest(), &[("Café", "", "")])
        .with_translation(
            "form.notice.delay",
            "Votre hôte réapprovisionne dans la journée.",
        )
        .run(|ctx| serde_json::to_string(&render_guest_form(ctx).expect("formulaire")).unwrap());
    assert!(
        form.contains("Votre hôte réapprovisionne dans la journée."),
        "{form}"
    );
}

#[test]
#[serial]
fn host_main_and_stats_render() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let main = render_host_main(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&main).contains_type("Page"));
            assert!(SurfaceAssertions::new(&main).contains_type("DataTable"));
            assert!(SurfaceAssertions::new(&main).contains_type("Section"));
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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Serviettes", "Towels", "")]) }),
            );
            let item_id = sample::id(0);

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

#[test]
#[serial]
fn an_empty_catalog_does_not_block_publication() {
    reset_test_store();
    MockContext::host().run(|ctx| {
        assert_eq!(sample::blocking(&ctx, "rows"), None);
        seed_defaults(ctx.clone(), EmptyArgs {}).expect("seed");
        assert_eq!(
            sample::blocking(&ctx, ""),
            None,
            "the starter list is publishable"
        );
    });
}

#[test]
#[serial]
fn more_than_thirty_products_blocks_publication() {
    let names: Vec<String> = (1..=31).map(|n| format!("Produit {n}")).collect();
    let items: Vec<(&str, &str, &str)> = names.iter().map(|n| (n.as_str(), "", "")).collect();
    sample::with(MockContext::host(), &items).run(|ctx| {
        assert_eq!(
            sample::blocking(&ctx, "products.rows").as_deref(),
            Some("30 produits au maximum.")
        );
    });
}

#[test]
#[serial]
fn a_product_without_a_name_is_flagged_and_hidden_from_the_guest() {
    let items = [("", "", "☕"), ("Papier", "", "🧻")];
    sample::with(MockContext::host(), &items).run(|ctx| {
        let field = format!("{}.label", sample::id(0));
        assert_eq!(
            sample::blocking(&ctx, &field).as_deref(),
            Some("Donnez un nom au produit.")
        );
    });
    let json = sample::with(MockContext::guest(), &items)
        .run(|ctx| serde_json::to_string(&render_guest_form(ctx).expect("render")).unwrap());
    assert!(json.contains("🧻") && !json.contains("☕"), "{json}");
}

#[test]
#[serial]
fn a_product_name_past_forty_characters_is_flagged() {
    let name = "c".repeat(41);
    sample::with(MockContext::host(), &[(name.as_str(), "", "")]).run(|ctx| {
        let field = format!("{}.label", sample::id(0));
        assert_eq!(
            sample::blocking(&ctx, &field).as_deref(),
            Some("40 caractères au maximum.")
        );
    });
}

#[test]
#[serial]
fn a_restock_delay_past_forty_characters_is_flagged() {
    let delay = json!({ "restock_delay": { "fr": "d".repeat(41) } });
    sample::with_meta(MockContext::host(), &[("Café", "", "")], delay).run(|ctx| {
        assert_eq!(
            sample::blocking(&ctx, "restock_delay").as_deref(),
            Some("40 caractères au maximum.")
        );
    });
}

#[test]
#[serial]
fn max_requests_out_of_range_is_reported_not_clamped() {
    for typed in [0, 25] {
        sample::with(MockContext::host(), &[("Café", "", "")]).run(|ctx| {
            sample::save(&ctx, json!({ "max_requests": typed }));
            assert_eq!(
                sample::blocking(&ctx, "max_requests").as_deref(),
                Some("Entre 1 et 20.")
            );
            sample::save(&ctx, json!({ "max_requests": "5" }));
            assert_eq!(sample::blocking(&ctx, "max_requests"), None);
        });
    }
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
            sample::save(
                &ctx,
                json!({ "products": sample::products(&[("Café", "Coffee", "")]) }),
            );
            let item_id = sample::id(0);

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
        .with_resource(
            "products",
            sample::products(&[("Café", "Coffee", "")]),
            json!({ "max_requests": 1 }),
        )
        .run(|ctx| {
            let item_id = sample::id(0);
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

/// À traiter → Prévu (avec une réponse) → Livré : la réponse atteint le voyageur, « Prévu » garde
/// la demande ouverte sans dater de réassort, et la tâche d'À venir ne vit que tant qu'à traiter.
#[test]
#[serial]
fn planned_with_a_reply_reaches_the_guest_and_ends_the_task() {
    const ITEMS: [(&str, &str, &str); 1] = [("Café", "Coffee", "")];
    reset_test_store();
    let mut stay_id = Uuid::nil();
    sample::with(MockContext::guest(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            let item_id = sample::id(0);
            submit(
                ctx.clone(),
                SubmitArgs {
                    item_ids: vec![item_id],
                    item_id: None,
                    level: "missing".into(),
                    note: None,
                },
            )
            .expect("submit");
            stay_id = list_for_stay(ctx, ListForStayArgs::default()).expect("list")[0].stay_id;
        });

    let tasks_args = || {
        serde_json::from_value(json!({
            "propertyId": Uuid::nil(),
            "from": "2000-01-01T00:00:00Z",
            "to": "2100-01-01T00:00:00Z",
            "stays": [{
                "id": stay_id,
                "checkIn": "2000-01-01T00:00:00Z",
                "checkOut": "2100-01-01T00:00:00Z",
                "guestName": "Marie",
                "status": "CHECKED_IN",
            }],
        }))
        .expect("timeline args")
    };

    let mut report_id = Uuid::nil();
    sample::with(MockContext::host(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            let tasks = timeline_tasks(ctx.clone(), tasks_args())
                .expect("tasks")
                .tasks;
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].title.fr, "Café demandé");
            assert_eq!(tasks[0].context.fr, "Marie");
            assert_eq!(tasks[0].stay_id, Some(stay_id));
            report_id = tasks[0].id.parse().expect("task id is the report id");

            // Le formulaire de l'hôte : ses boutons portent le statut, la coquille y fusionne le
            // champ de réponse sous son nom de fil.
            let main = serde_json::to_string(&render_host_main(ctx.clone()).expect("render"))
                .expect("json");
            assert!(main.contains("host.main.markPlanned"), "{main}");
            assert!(main.contains(r#""name":"hostReply""#), "{main}");
            let args: UpdateStatusArgs = serde_json::from_value(json!({
                "reportId": report_id,
                "status": "planned",
                "hostReply": "  Je passe ce soir vers 18 h.  ",
            }))
            .expect("form payload");
            update_status(ctx.clone(), args).expect("planned");

            assert_eq!(list_open_count(ctx.clone()).expect("open").open_count, 1);
            assert!(timeline_tasks(ctx.clone(), tasks_args())
                .expect("tasks")
                .tasks
                .is_empty());
            let row = &list_for_stay(
                ctx.clone(),
                ListForStayArgs {
                    stay_id: Some(stay_id),
                },
            )
            .expect("rows")[0];
            assert_eq!(row.status, "planned");
            assert_eq!(
                row.host_reply.as_deref(),
                Some("Je passe ce soir vers 18 h.")
            );
            // « Prévu » n'est pas un réassort.
            let stats = serde_json::to_string(&render_host_stats(ctx.clone())).expect("json");
            assert!(!stats.contains("stats.row.restockedAt"), "{stats}");
        });

    sample::with(MockContext::guest(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            let card =
                serde_json::to_string(&render_home_card(ctx.clone()).expect("card")).expect("json");
            assert!(card.contains("status.planned"), "{card}");
            assert!(card.contains("home.card.reply"), "{card}");
            // Toujours pas livré : pas de doublon.
            let form = serde_json::to_string(&render_guest_form(ctx).expect("form")).expect("json");
            assert!(form.contains("form.item.reported.missing"), "{form}");
        });

    sample::with(MockContext::host(), &ITEMS)
        .with_property(Property::default())
        .run(|ctx| {
            task_complete(
                ctx.clone(),
                serde_json::from_value(json!({
                    "propertyId": Uuid::nil(),
                    "taskId": report_id.to_string(),
                }))
                .expect("complete args"),
            )
            .expect("delivered");
            assert_eq!(list_open_count(ctx).expect("open").open_count, 0);
        });
}

/// L'orchestrateur n'appelle `timelineTasks` que pour un module qui déclare la nav
/// `WorkspaceTimelineTask` : sans elle, « Café demandé · Marie » n'atteint jamais À venir.
#[test]
fn timeline_tasks_are_declared_to_the_dashboard() {
    let path = concat!(
        env!("OUT_DIR"),
        "/portaki-emissions/nav-tasks_HostPlacement__WorkspaceTimelineTask.json"
    );
    let raw = std::fs::read_to_string(path).expect("the tasks nav is emitted");
    let nav: serde_json::Value = serde_json::from_str(&raw).expect("nav json");
    assert_eq!(nav["placement"], "HostPlacement::WorkspaceTimelineTask");
    assert_eq!(nav["path"], "tasks");
    assert_eq!(nav["label_key"], "catalog.host.tasks");
}
