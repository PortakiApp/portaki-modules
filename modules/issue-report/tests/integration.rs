//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use serial_test::serial;

use issue_report::{
    add, list_for_stay, list_recent, render_guest_form, render_home_card, render_host_add,
    render_host_main, render_host_stats, render_host_stay, reset_test_store, resolve,
    stats_summary, submit, task_complete, task_toggle, timeline_tasks, AddArgs, Category,
    ResolveArgs, SubmitArgs, GUEST_TEXT_EMAIL_MAX_CHARS,
};
use portaki_sdk::contracts::stats::StatsSummaryArgs;
use portaki_sdk::contracts::timeline::{TaskCompleteArgs, TaskToggleArgs, TimelineTasksArgs};
use portaki_sdk::limits;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use uuid::Uuid;

#[test]
#[serial]
fn home_card_opens_form_overlay_when_no_reports() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.intro"));
            assert!(json.contains("danger-triangle"));
            assert!(json.contains("guest.form"));
            assert!(json.contains("home.card.openForm"));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("form.category.label"));
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
                    category: Category::Appliance,
                    summary: "Oven broken".into(),
                    details: Some("Won't heat".into()),
                    photo: None,
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
                    category: Category::Noise,
                    summary: "Loud neighbors".into(),
                    details: None,
                    photo: None,
                },
            )
            .expect("submit second");

            let rows = list_for_stay(ctx.clone()).expect("list after second");
            assert_eq!(rows.len(), 2);

            let surface = render_home_card(ctx).expect("guest surface");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.card.thanks"));
            assert!(json.contains("home.card.yourReports"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

/// §9 #5 : le voyageur voit « Résolu » dans son historique dès que l'hôte a clos le signalement.
#[test]
#[serial]
fn the_guest_history_shows_resolved_reports() {
    reset_test_store();
    let guest = MockContext::guest().with_property(Property::default());
    let mut oven = None;
    guest.clone().run(|ctx| {
        for (category, summary) in [(Category::Appliance, "Oven"), (Category::Noise, "Noise")] {
            let report = submit(
                ctx.clone(),
                SubmitArgs {
                    category,
                    summary: summary.into(),
                    details: None,
                    photo: None,
                },
            );
            report.expect("submit");
        }
        oven = list_for_stay(ctx)
            .expect("list")
            .into_iter()
            .find(|row| row.summary == "Oven")
            .map(|row| row.id);
    });
    let card = || {
        let mut json = String::new();
        guest.clone().run(|ctx| {
            json = serde_json::to_string(&render_home_card(ctx).expect("guest surface"))
                .expect("surface json");
        });
        json
    };
    assert!(!card().contains("home.card.resolved"));

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let report_id = oven.expect("oven");
            resolve(ctx, ResolveArgs { report_id }).expect("resolve");
        });
    assert_eq!(card().matches("i18n:home.card.resolved").count(), 1);
}

#[test]
#[serial]
fn host_stats_list_recent_after_guest_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    category: Category::Cleanliness,
                    summary: "Bathroom not clean".into(),
                    details: None,
                    photo: None,
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let recent = list_recent(ctx.clone()).expect("listRecent");
            assert_eq!(recent.len(), 1);

            let surface = render_host_stats(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("FeedItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.main.recentTitle"));
            assert!(json.contains("host.main.status.open"));
            assert!(json.contains("i18n:stats.category.cleanliness"));
        });
}

#[test]
#[serial]
fn host_stats_reflect_resolution_and_period() {
    reset_test_store();
    let t0 = DateTime::parse_from_rfc3339("2026-09-01T08:00:00Z")
        .expect("t0")
        .with_timezone(&Utc);
    MockContext::guest()
        .with_property(Property::default())
        .with_now(t0)
        .run(|ctx| {
            for (category, summary) in [
                (Category::Appliance, "Oven"),
                (Category::Appliance, "Boiler"),
                (Category::Access, "Code"),
            ] {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        category,
                        summary: summary.into(),
                        details: None,
                        photo: None,
                    },
                )
                .expect("submit");
            }
        });

    let render = |now: DateTime<Utc>, period: Option<u64>| {
        let mut out = String::new();
        MockContext::host()
            .with_property(Property::default())
            .with_now(now)
            .run(|mut ctx| {
                if let Some(days) = period {
                    ctx.input = serde_json::json!({ "periodDays": days });
                }
                out = serde_json::to_value(render_host_stats(ctx))
                    .expect("surface json")
                    .to_string();
            });
        out
    };

    MockContext::host()
        .with_property(Property::default())
        .with_now(t0 + Duration::hours(6))
        .run(|ctx| {
            let oven = list_recent(ctx.clone())
                .expect("listRecent")
                .into_iter()
                .find(|r| r.summary == "Oven")
                .expect("oven");
            resolve(ctx.clone(), ResolveArgs { report_id: oven.id }).expect("resolve");
            // Second call keeps the first resolution time.
            resolve(ctx.clone(), ResolveArgs { report_id: oven.id }).expect("resolve again");

            let main = serde_json::to_string(&render_host_stats(ctx.clone())).expect("main json");
            assert!(main.contains("host.main.status.resolved"));
            assert_eq!(main.matches("host.main.resolvedIn").count(), 1);

            let tile = stats_summary(
                ctx,
                StatsSummaryArgs {
                    property_id: Uuid::nil(),
                    period: 30,
                    key: "issue-stats".into(),
                },
            )
            .expect("statsSummary");
            assert_eq!(tile.value, "3");
            assert_eq!(tile.label.fr, "signalements · 30 j");
            assert_eq!(tile.attention.expect("open").text.fr, "2 en cours");
        });

    let text = render(t0 + Duration::hours(9), None);
    assert!(text.contains(
        r#""delta":"sur 30 jours","icon":"danger-triangle","label":"i18n:stats.reports","type":"Stat","value":"3""#
    ));
    assert!(text.contains(
        r#""delta":"délai moyen 6 h","icon":"check-circle","label":"i18n:stats.resolved","type":"Stat","value":"1""#
    ));
    assert!(text
        .contains(r#""icon":"clock-circle","label":"i18n:stats.open","type":"Stat","value":"2""#));
    assert!(text.contains(
        r#""display":"2 signalements","label":"i18n:stats.category.appliance","value":2.0"#
    ));
    assert!(text.find("stats.category.appliance") < text.find("stats.category.access"));
    // 30 days → five weekly slices; the oven, reported and resolved in the last one, took 6 h.
    assert!(
        text.contains(r#""display":"6 h","label":"S5","value":6.0"#),
        "{text}"
    );

    let later = t0 + Duration::days(40);
    assert!(render(later, None).contains(
        r#""icon":"danger-triangle","label":"i18n:stats.reports","type":"Stat","value":"0""#
    ));
    let quarter = render(later, Some(90));
    assert!(quarter.contains(
        r#""delta":"sur 90 jours","icon":"danger-triangle","label":"i18n:stats.reports","type":"Stat","value":"3""#
    ));
}

/// A 20 000-char description: the report keeps it whole, the host email quotes at most
/// `GUEST_TEXT_EMAIL_MAX_CHARS` chars then `…`, and the CTA reads « Voir plus ».
#[test]
#[serial]
fn long_details_are_stored_whole_and_quoted_in_the_host_email() {
    reset_test_store();
    let details = format!("{}!", "robinet ".repeat(2_500).trim_end());
    assert_eq!(details.chars().count(), 20_000);

    MockContext::guest()
        .with_property(Property::default())
        .run_with(|ctx, host| {
            submit(
                ctx.clone(),
                SubmitArgs {
                    category: Category::Appliance,
                    summary: "Fuite sous l'évier".into(),
                    details: Some(details.clone()),
                    photo: None,
                },
            )
            .expect("submit");

            let rows = list_for_stay(ctx.clone()).expect("list");
            assert_eq!(rows[0].details.as_deref(), Some(details.as_str()));

            let email = host.sent_emails().into_iter().last().expect("host email");
            let body = &email.content.body.fr;
            assert!(body.chars().count() <= limits::EMAIL_BODY_MAX_CHARS);
            assert!(body.contains("Fuite sous l'évier"));
            let quoted = body
                .split("\n\n")
                .find(|part| part.ends_with('…'))
                .expect("quoted details");
            let kept = quoted.trim_end_matches('…');
            assert!(kept.chars().count() <= GUEST_TEXT_EMAIL_MAX_CHARS);
            assert!(details.starts_with(kept));

            let cta = email.content.cta.as_ref().expect("cta");
            assert_eq!(cta.label.fr, "Voir plus");
            assert_eq!(cta.label.en, "See more");
            assert_eq!(email.property_id, Some(ctx.property_id));
            assert!(email.action_url.is_none());
        });
}

const PHOTO: &str = "portaki-file:6f1c1d2e-3a4b-4c5d-8e9f-0a1b2c3d4e5f";

#[test]
#[serial]
fn a_guest_photo_reaches_the_host_screen_and_the_stats() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let form = render_guest_form(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("ImageUpload"));

            let refused = submit(
                ctx.clone(),
                SubmitArgs {
                    category: Category::Other,
                    summary: "Tracker".into(),
                    details: None,
                    photo: Some("https://evil.example/pixel.png".into()),
                },
            );
            assert!(refused.is_err(), "only a platform reference is accepted");

            for photo in [Some(PHOTO.to_string()), Some("  ".into()), None] {
                submit(
                    ctx.clone(),
                    SubmitArgs {
                        category: Category::Appliance,
                        summary: "Oven".into(),
                        details: None,
                        photo,
                    },
                )
                .expect("submit");
            }
            let rows = list_for_stay(ctx).expect("list");
            assert_eq!(
                rows.iter()
                    .filter(|r| r.photo.as_deref() == Some(PHOTO))
                    .count(),
                1
            );
            assert_eq!(rows.iter().filter(|r| r.photo.is_none()).count(), 2);
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let main = serde_json::to_string(&render_host_stats(ctx.clone())).expect("main json");
            assert!(main.contains("host.main.withPhoto"));
            let stats = serde_json::to_value(render_host_stats(ctx))
                .expect("stats json")
                .to_string();
            assert!(stats.contains(
                r#""icon":"image","label":"i18n:stats.withPhoto","type":"Stat","value":"1""#
            ));
        });
}

#[test]
#[serial]
fn a_feed_row_opens_the_report_detail() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    category: Category::Appliance,
                    summary: "Oven".into(),
                    details: Some("Door stuck".into()),
                    photo: None,
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            let feed = serde_json::to_string(&render_host_stats(ctx.clone())).expect("json");
            assert!(feed.contains("host.surface.overlay"));
            assert!(feed.contains("issueId"));

            let report = list_recent(ctx.clone()).expect("recent").remove(0);
            ctx.input = serde_json::json!({ "issueId": report.id, "periodDays": 30 });
            let detail = serde_json::to_string(&render_host_stats(ctx)).expect("json");
            assert!(detail.contains("Door stuck"));
            assert!(detail.contains("host.detail.resolve"));
            assert!(detail.contains(&format!("/stays/{}", report.stay_id)));
            assert!(!detail.contains("FeedItem"));
        });
}

/// L'hôte retire une catégorie : la pastille disparaît du formulaire (§10).
#[test]
#[serial]
fn the_form_shows_only_the_categories_the_host_offers() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "category_appliance": true, "category_other": true }))
        .run(|ctx| {
            let json = serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap();
            assert!(json.contains("form.category.appliance"), "{json}");
            assert!(json.contains("form.category.other"), "{json}");
            assert!(!json.contains("form.category.noise"), "{json}");
        });

    // Rien coché : le formulaire reste celui que l'hôte avait, les cinq pastilles.
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let json = serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap();
            for wire in ["appliance", "cleanliness", "noise", "access", "other"] {
                assert!(json.contains(&format!("form.category.{wire}")), "{json}");
            }
        });
}

/// Le refus tient à la configuration et non à l'écran : un formulaire resté ouvert dans un
/// téléphone proposait encore la pastille que l'hôte vient de retirer.
#[test]
#[serial]
fn a_dropped_category_is_refused_on_submit() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "category_appliance": true }))
        .run(|ctx| {
            let refused = submit(
                ctx.clone(),
                SubmitArgs {
                    category: Category::Noise,
                    summary: "Musique forte chez les voisins".into(),
                    details: None,
                    photo: None,
                },
            );
            assert!(
                refused.is_err(),
                "une catégorie retirée n'est pas signalable"
            );

            submit(
                ctx,
                SubmitArgs {
                    category: Category::Appliance,
                    summary: "Le four ne chauffe plus".into(),
                    details: None,
                    photo: None,
                },
            )
            .expect("la catégorie proposée passe");
        });
}

/// Le rappel d'urgence du §2.17, et son lien vers les numéros.
///
/// Un signalement n'est pas un appel : il attend que l'hôte le lise. Quand ça ne peut pas
/// attendre, le formulaire doit le dire — et emmener là où sont les numéros, pas afficher un
/// numéro de plus que l'hôte aurait saisi ici.
#[test]
#[serial]
fn the_form_says_not_to_wait_in_an_emergency() {
    MockContext::guest().run(|ctx| {
        let json = serde_json::to_string(&render_guest_form(ctx).expect("form")).expect("json");
        assert!(json.contains("form.urgent.note"), "{json}");
        assert!(json.contains(r#""type":"navigate","to":"aide""#), "{json}");
    });
}

/// Une seule catégorie offerte : pas de pastilles, pas de question (§2.17).
///
/// Choisir dans une liste d'un seul élément n'est pas un choix — c'est une ligne de plus à lire
/// dans une feuille qu'on ouvre parce que quelque chose ne va pas. La catégorie part quand même
/// au signalement : le formulaire la porte en valeur.
#[test]
#[serial]
fn one_category_asks_no_question() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "category_appliance": true }))
        .run(|ctx| {
            let json = serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap();
            assert!(!json.contains("form.category.label"), "{json}");
            assert!(!json.contains("\"ChoiceList\""), "{json}");
            assert!(json.contains("\"value\":\"appliance\""), "{json}");
        });
}

/// La feuille dit ce qu'on y fait avant de demander quoi que ce soit.
///
/// Le livret hisse l'en-tête d'un formulaire dans la barre du panneau : sans lui, la feuille
/// s'ouvrait sur « Catégorie », sans dire ce qu'on est en train de faire ni ce qui arrive après.
#[test]
#[serial]
fn the_sheet_says_what_it_is_before_asking() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let json = serde_json::to_string(&render_guest_form(ctx).expect("form")).unwrap();
            let title = json.find("form.head.title").expect("titre");
            let first_field = json.find("form.category.label").expect("premier champ");
            assert!(title < first_field, "{json}");
            assert!(json.contains("form.head.lead"), "{json}");
        });
}

/// Tout décoché : les cases restent décochées dans le tiroir, le message sous la dernière.
#[test]
#[serial]
fn the_sheet_says_when_a_set_is_empty() {
    MockContext::host()
        .with_property(Property::default())
        .with_config(&json!({ "category_other": false, "phase_after": false }))
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("host surface"))
                .expect("surface json");
            assert!(json.contains("Choisissez au moins une catégorie."));
            assert!(json.contains("Choisissez au moins une période."));
            // Seules la photo et l'option urgente, ouvertes par défaut, restent cochées.
            assert_eq!(json.matches(r#""checked":true"#).count(), 2);
        });
}

fn host_add(ctx: portaki_sdk::prelude::Context, stay_id: Uuid, summary: &str) {
    add(
        ctx,
        AddArgs {
            stay_id,
            category: Category::Appliance,
            summary: summary.into(),
            details: None,
        },
    )
    .expect("add");
}

#[test]
#[serial]
fn the_stay_encart_lists_its_reports_or_says_there_are_none() {
    reset_test_store();
    let (stay, other) = (Uuid::new_v4(), Uuid::new_v4());
    let render = |stay_id: Option<Uuid>| {
        let mut out = String::new();
        MockContext::host()
            .with_property(Property::default())
            .run(|mut ctx| {
                if let Some(id) = stay_id {
                    ctx.input = json!({ "stayId": id });
                }
                out = serde_json::to_string(&render_host_stay(ctx)).expect("surface json");
            });
        out
    };

    let empty = render(Some(stay));
    assert!(empty.contains("host.stay.empty"));
    assert!(!empty.contains("FeedItem"));
    assert!(render(None).contains("host.stay.missingStay"));

    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            host_add(ctx.clone(), stay, "Plus d'eau chaude");
            host_add(ctx, other, "Autre séjour");
        });
    let one = render(Some(stay));
    assert!(one.contains("Plus d'eau chaude"));
    assert!(!one.contains("Autre séjour"));
    assert!(one.contains("host.stay.openOne"));
    assert!(one.contains("host.main.status.open"));

    let id = MockContext::host()
        .run(list_recent)
        .expect("recent")
        .into_iter()
        .find(|row| row.stay_id == stay)
        .expect("report")
        .id;
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| resolve(ctx, ResolveArgs { report_id: id }).expect("resolve"));
    let resolved = render(Some(stay));
    assert!(resolved.contains("host.stay.openNone"));
    assert!(resolved.contains("host.main.status.resolved"));
}

#[test]
#[serial]
fn the_stay_action_adds_a_report_to_the_stay() {
    reset_test_store();
    let stay = Uuid::new_v4();
    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            assert!(serde_json::to_string(&render_host_add(ctx.clone()))
                .unwrap()
                .contains("host.stay.missingStay"));
            ctx.input = json!({ "stayId": stay });
            let form = render_host_add(ctx.clone());
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            let json = serde_json::to_string(&form).unwrap();
            assert!(json.contains(&stay.to_string()));
            assert!(json.contains("host.add.submit"));

            host_add(ctx.clone(), stay, "  Volet bloqué ");
            let empty = add(
                ctx,
                AddArgs {
                    stay_id: stay,
                    category: Category::Other,
                    summary: " ".into(),
                    details: None,
                },
            );
            assert!(empty.is_err());
        });
    let rows = MockContext::host().run(list_recent).expect("recent");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].stay_id, rows[0].summary.as_str()),
        (stay, "Volet bloqué")
    );

    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let refused = add(
                ctx,
                AddArgs {
                    stay_id: stay,
                    category: Category::Other,
                    summary: "x".into(),
                    details: None,
                },
            );
            assert!(refused.is_err());
        });
}

#[test]
#[serial]
fn a_report_open_for_a_day_is_a_task_until_resolved() {
    reset_test_store();
    let t0 = DateTime::parse_from_rfc3339("2026-09-01T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let stay = Uuid::new_v4();
    MockContext::host()
        .with_property(Property::default())
        .with_now(t0)
        .run(|ctx| host_add(ctx, stay, "Plus d'eau chaude"));

    let tasks_at = |now: DateTime<Utc>| {
        let mut out = Vec::new();
        MockContext::host()
            .with_property(Property::default())
            .with_now(now)
            .run(|ctx| {
                out = timeline_tasks(
                    ctx,
                    TimelineTasksArgs {
                        property_id: Uuid::nil(),
                        from: t0 - Duration::days(7),
                        to: t0 + Duration::days(7),
                        stays: Vec::new(),
                    },
                )
                .expect("tasks")
                .tasks;
            });
        out
    };

    assert!(tasks_at(t0 + Duration::hours(12)).is_empty());
    let tasks = tasks_at(t0 + Duration::hours(25));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title.fr, "Signalement non traité depuis 24 h");
    assert_eq!(tasks[0].context.fr, "Plus d'eau chaude");
    assert_eq!(tasks[0].stay_id, Some(stay));
    assert_eq!(tasks[0].at, t0 + Duration::hours(24));

    let task_id = tasks[0].id.clone();
    MockContext::host()
        .with_property(Property::default())
        .with_now(t0 + Duration::hours(26))
        .run(|ctx| {
            let untick = task_toggle(
                ctx.clone(),
                TaskToggleArgs {
                    property_id: Uuid::nil(),
                    task_id: task_id.clone(),
                    item_id: "resolve".into(),
                    done: false,
                    photo: None,
                },
            );
            assert!(untick.is_err());
            task_complete(
                ctx,
                TaskCompleteArgs {
                    property_id: Uuid::nil(),
                    task_id: task_id.clone(),
                },
            )
            .expect("complete");
        });
    assert!(tasks_at(t0 + Duration::hours(27)).is_empty());
}
