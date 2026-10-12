//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use serial_test::serial;

use uuid::Uuid;

use chrono::{Duration, Utc};
use portaki_sdk::context::HostProfile;
use portaki_sdk::prelude::StayContext;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use pre_arrival_form::{
    get_status, publish_readiness, render_guest_form, render_home_card, render_host_main,
    render_host_resend, render_host_stats, render_host_stay, resend_form, reset_test_store,
    send_form_available, send_reminder_j0, send_reminder_j1, send_reminder_j3, stats_summary,
    submit, Deadline, ModuleConfig, ResendFormArgs, ShowWhen, SubmitArgs,
};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_submit() -> SubmitArgs {
    SubmitArgs {
        arrival_time_estimated: Some("17:30".into()),
        guest_occasion: Some("birthday".into()),
        guest_allergies: None,
        guest_count: Some("2".into()),
        special_needs: None,
        id_document: None,
        message_to_host: Some("Merci !".into()),
        ..Default::default()
    }
}

#[test]
#[serial]
fn home_card_renders_form_when_incomplete() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_home_card(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.pending"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("guest.form"));
            assert!(!json.contains("TimePicker"));

            // Sans date d'arrivée, aucune borne de créneau : le sélecteur libre reste.
            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("TimePicker"));
            assert!(SurfaceAssertions::new(&form).contains_type("TextArea"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            // Overlay chrome owns framing — no nested Card around the form.
            assert!(!SurfaceAssertions::new(&form).contains_type("Card"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("home.card.intro"));
            assert!(form_json.contains("submit"));
        });
}

#[test]
#[serial]
fn submit_then_status_and_thanks_card() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            // Still before check-in — completed form stays editable.
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(2)),
                checkout_at: Some(Utc::now() + Duration::days(5)),
                booking_channel: None,
                ..StayContext::default()
            });

            let before = get_status(ctx.clone()).expect("status");
            assert!(!before.completed);

            submit(ctx.clone(), sample_submit()).expect("submit");

            let after = get_status(ctx.clone()).expect("status after");
            assert!(after.completed);
            assert_eq!(after.arrival_time_estimated.as_deref(), Some("17:30"));
            assert_eq!(after.guest_occasion.as_deref(), Some("birthday"));

            let surface = render_home_card(ctx.clone()).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("home.formalities.allReady"));
            assert!(json.contains("home.task.completed"));
            assert!(json.contains("home.task.preArrival.label"));
            assert!(json.contains("check-circle"));
            assert!(!json.contains("TimePicker"));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(SurfaceAssertions::new(&form).contains_type("Button"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("form.submitUpdate"));
            // L'heure se choisit en créneaux depuis que le logement en donne les bornes : la
            // réponse libre « 17:30 » revient cochée sur le créneau qui la contient.
            assert!(SurfaceAssertions::new(&form).contains_type("ChoiceList"));
            assert!(!SurfaceAssertions::new(&form).contains_type("TimePicker"));
            assert!(form_json.contains("\"segmented\""), "{form_json}");
            // Et le formulaire déjà envoyé s'ouvre sur la confirmation (§2.20).
            assert!(SurfaceAssertions::new(&form).contains_type("Celebration"));
            // L'occasion est désormais une valeur de liste, présélectionnée dans le choix : le
            // libellé affiché est traduit par le livret, le formulaire porte la valeur.
            assert!(form_json.contains("\"value\":\"birthday\""));
            assert!(form_json.contains("i18n:form.occasion.birthday"));
        });
}

#[test]
#[serial]
fn completed_form_locks_after_checkin() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_translation("form.arrival.from", "dès {time}")
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::hours(2)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            submit(ctx.clone(), sample_submit()).expect("submit before check-in");

            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() - Duration::hours(1)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });

            let form = render_guest_form(ctx.clone()).expect("guest surface");
            assert!(!SurfaceAssertions::new(&form).contains_type("Form"));
            assert!(!SurfaceAssertions::new(&form).contains_type("Button"));
            let form_json = serde_json::to_string(&form).expect("form json");
            assert!(form_json.contains("home.card.thanks"));
            assert!(form_json.contains("home.card.lockedHint"));
            // « dès 17:30 » : l'heure annoncée est un plancher, pas un rendez-vous.
            assert!(form_json.contains("dès 17:30"), "{form_json}");

            let err = submit(
                ctx,
                SubmitArgs {
                    arrival_time_estimated: Some("18:00".into()),
                    guest_occasion: None,
                    guest_allergies: None,
                    guest_count: None,
                    special_needs: None,
                    id_document: None,
                    message_to_host: None,
                    ..Default::default()
                },
            )
            .expect_err("submit locked after check-in");
            assert!(format!("{err:?}").contains("form_locked_after_checkin"));
        });
}

#[test]
#[serial]
fn home_card_gated_omits_form_teaser_keeps_police_fragment() {
    reset_test_store();
    let config = json!({
        "show_when": "before",
        "ask_arrival_time": true,
        "ask_occasion": true,
        "ask_allergies": true,
        "ask_guest_count": true,
        "ask_special_needs": false,
        "ask_id_document": false
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(10)),
                checkout_at: Some(Utc::now() + Duration::days(14)),
                booking_channel: None,
                ..StayContext::default()
            });

            let surface = render_home_card(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("HostFragment"));
            // Gated: no form ListItem / soon teaser — police fragment only.
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(!json.contains("home.formalities.pendingGate"));
            assert!(!json.contains("home.card.notYet"));
            assert!(!json.contains("home.task.preArrival"));
            assert!(json.contains("home.formalities.pending"));
            assert!(!json.contains("home.card.intro"));
            assert!(!json.contains("TimePicker"));
            assert!(!json.contains("\"type\":\"Form\"") && !json.contains("\"type\": \"Form\""));
        });
}

#[test]
#[serial]
fn send_form_available_noops_when_gated() {
    reset_test_store();
    let config = json!({
        "show_when": "checkin"
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(5)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            send_form_available(ctx, portaki_sdk::prelude::EmptyArgs {})
                .expect("sendFormAvailable gated no-op");
        });
}

#[test]
#[serial]
fn send_form_available_ok_when_confirm() {
    reset_test_store();
    let config = json!({
        "show_when": "confirm"
    });

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run(|mut ctx| {
            let stay_id = ctx
                .guest
                .as_ref()
                .map(|guest| guest.session_id)
                .expect("guest stay");
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + Duration::days(20)),
                checkout_at: None,
                booking_channel: None,
                ..StayContext::default()
            });
            send_form_available(ctx, portaki_sdk::prelude::EmptyArgs {})
                .expect("sendFormAvailable when available");
        });
}

/// Les e-mails de relance partis, pour une arrivée dans `checkin_in` et la config `config` ;
/// `answered` : le voyageur a déjà répondu.
fn reminders_sent(config: serde_json::Value, checkin_in: Duration, answered: bool) -> Vec<String> {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&config)
        .run_with(|mut ctx, host| {
            let stay_id = ctx.guest.as_ref().map(|guest| guest.session_id).unwrap();
            ctx.stay = Some(StayContext {
                stay_id,
                checkin_at: Some(Utc::now() + checkin_in),
                ..StayContext::default()
            });
            if answered {
                submit(ctx.clone(), sample_submit()).unwrap();
            }
            let none = portaki_sdk::prelude::EmptyArgs {};
            send_reminder_j3(ctx.clone(), none).unwrap();
            send_reminder_j1(ctx.clone(), none).unwrap();
            send_reminder_j0(ctx, none).unwrap();
            host.sent_emails()
                .into_iter()
                .map(|mail| mail.email_id)
                .collect()
        })
}

#[test]
#[serial]
fn only_the_chosen_deadline_reminds_an_open_unanswered_form() {
    let open = json!({ "show_when": "confirm" });
    let in_ten_days = Duration::days(10);
    // Défaut : la veille à 18 h, et seule sa relance part.
    assert_eq!(
        reminders_sent(open.clone(), in_ten_days, false),
        ["reminder-j-1-18h"]
    );
    assert_eq!(
        reminders_sent(
            json!({ "show_when": "confirm", "deadline": "j-3" }),
            in_ten_days,
            false
        ),
        ["reminder-j-3"]
    );
    // Déjà répondu, relance éteinte : rien.
    assert!(reminders_sent(open.clone(), in_ten_days, true).is_empty());
    assert!(reminders_sent(
        json!({ "show_when": "confirm", "reminder": false }),
        in_ten_days,
        false
    )
    .is_empty());
    // Formulaire pas encore ouvert (48 h avant) : l'e-mail d'ouverture suffira.
    assert!(reminders_sent(json!({ "show_when": "before" }), in_ten_days, false).is_empty());
    // 48 h avant + jour même 12 h, passage de J-1 : ouvert depuis moins de 24 h, l'e-mail
    // d'ouverture part à ce passage-là — pas de relance ; ouvert depuis 28 h, elle part.
    let before_noon = json!({ "show_when": "before", "deadline": "j0-12h" });
    assert!(reminders_sent(before_noon.clone(), Duration::hours(30), false).is_empty());
    assert_eq!(
        reminders_sent(before_noon, Duration::hours(20), false),
        ["reminder-j0-12h"]
    );
    // Limite passée (arrivée dans une heure, limite la veille à 18 h) : pas de relance.
    assert!(reminders_sent(open, Duration::hours(1), false).is_empty());
}

/// Les champs que la page ne montre plus : leur valeur enregistrée reste, `updateConfig` fusionne.
const RETIRED: &[&str] = &[
    "ask_arrival_time",
    "ask_allergies",
    "ask_special_needs",
    "ask_id_document",
];

#[test]
#[serial]
fn host_main_renders_config_editor() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_config(&ModuleConfig {
            show_when: ShowWhen::Checkin,
            ask_id_document: true,
            ..ModuleConfig::default()
        })
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            // Exactement les clés déclarées, moins celles que la page a quittées — et les
            // questions, un tableau vide ici (`rowName`, sans champ nommé ; voir le test suivant).
            let mut not_in_form = RETIRED.to_vec();
            not_in_form.push("custom_questions");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &not_in_form,
            );
            let page = SurfaceAssertions::new(&surface);
            assert!(page.contains_type("Section"));
            assert!(page.contains_type("DataTable"));
            assert!(page.contains_type("TimePicker"));
            let json = serde_json::to_string(&surface).expect("surface json");
            for key in [
                "host.section.arrival",
                "host.section.questions",
                "host.section.reminder",
                "late_from",
                "ask_needs",
                "show_when",
            ] {
                assert!(json.contains(key), "{key}");
            }
            // La pièce d'identité a quitté la page, même cochée.
            assert!(!json.contains("ask_id_document"));
            // Arrivée tardive à 22:00 par défaut, pas de 30 min par défaut.
            assert!(
                json.contains(r#""name":"late_from","value":"22:00""#),
                "{json}"
            );
            assert!(
                json.contains(r#""name":"slot_step","value":"30""#),
                "{json}"
            );
        });
}

/// « Besoins particuliers » : un interrupteur pour deux réglages d'avant. Lu des deux tant que
/// l'hôte ne l'a pas enregistré, puis seul — l'éteindre ne se rallume pas par `ask_allergies`.
#[test]
fn the_needs_toggle_reads_both_old_keys_until_it_is_saved() {
    let read = |config: serde_json::Value| -> bool {
        serde_json::from_value::<ModuleConfig>(config)
            .expect("config")
            .asks_needs()
    };
    assert!(ModuleConfig::default().asks_needs());
    assert!(read(
        json!({ "ask_allergies": false, "ask_special_needs": true })
    ));
    assert!(!read(
        json!({ "ask_allergies": false, "ask_special_needs": false })
    ));
    assert!(!read(json!({ "ask_allergies": true, "ask_needs": false })));
    assert!(read(json!({ "ask_allergies": false, "ask_needs": true })));
}

#[test]
#[serial]
fn guest_form_respects_question_toggles() {
    reset_test_store();
    let render = |config: serde_json::Value| -> String {
        MockContext::guest()
            .with_property(Property::default())
            .with_config(&config)
            .run(|ctx| {
                serde_json::to_string(&render_guest_form(ctx).expect("guest surface"))
                    .expect("surface json")
            })
    };
    let json = render(json!({
        "show_when": "confirm",
        "ask_arrival_time": false,
        "ask_occasion": false,
        "ask_allergies": true,
        "ask_guest_count": false,
        "ask_special_needs": false,
        "ask_id_document": true
    }));
    // L'heure est toujours demandée (§9 cas 4), et la pièce d'identité ne l'est plus.
    assert!(json.contains("form.arrival.label"));
    assert!(!json.contains("form.idDocument.label"));
    assert!(!json.contains("form.occasion.label"));
    assert!(!json.contains("form.guestCount.label"));
    // Besoins particuliers : les allergies comprises.
    assert!(json.contains("form.specialNeeds.label"));
    assert!(json.contains("form.allergies.label"));

    let json = render(json!({ "show_when": "confirm", "ask_needs": false }));
    assert!(!json.contains("form.specialNeeds.label"));
    assert!(!json.contains("form.allergies.label"));
}

#[test]
#[serial]
fn host_stay_surface_pending_without_response() {
    reset_test_store();
    let stay_id = Uuid::new_v4();

    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({
                "stayId": stay_id.to_string(),
                "guestName": "Liam O'Brien",
                "stayDates": "21 – 26 août",
            });
            let surface = render_host_stay(ctx).expect("host stay");
            assert!(SurfaceAssertions::new(&surface).contains_type("Page"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("surface.host.stay.title"));
            assert!(json.contains("host.stay.status.pending"));
            assert!(json.contains("host.stay.pending"));
            assert!(!json.contains("form.arrival.label"));
        });
}

#[test]
#[serial]
fn host_stay_surface_shows_completed_response() {
    reset_test_store();
    let stay_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("uuid");

    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            if let Some(guest) = ctx.guest.as_mut() {
                guest.session_id = stay_id;
            }
            submit(
                ctx,
                SubmitArgs {
                    arrival_time_estimated: Some("17:30".into()),
                    guest_occasion: Some("Lune de miel".into()),
                    guest_allergies: Some("Fruits à coque".into()),
                    guest_count: None,
                    special_needs: None,
                    id_document: None,
                    message_to_host: Some("Champagne au frais".into()),
                    ..Default::default()
                },
            )
            .expect("submit");
        });

    MockContext::host()
        .with_property(Property::default())
        .with_translation("form.arrival.from", "dès {time}")
        .run(|mut ctx| {
            ctx.input = serde_json::json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx).expect("host stay");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pill"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.status.done"));
            assert!(
                json.contains("\"icon\":\"clipboard\"") || json.contains("\"icon\": \"clipboard\"")
            );
            assert!(json.contains("host.stay.arrival.label"));
            assert!(json.contains("host.stay.occasion.label"));
            assert!(json.contains("host.stay.allergies.label"));
            assert!(json.contains("clock-circle"));
            // « dès 17:30 » : le voyageur annonce un plancher, pas un rendez-vous. Lu comme une
            // heure exacte, l'hôte attend à l'heure pile quelqu'un qui a annoncé un créneau.
            assert!(json.contains("dès 17:30"), "{json}");
            assert!(json.contains("Lune de miel"));
            assert!(json.contains("Fruits à coque"));
            assert!(json.contains("Champagne au frais"));
        });
}

#[test]
#[serial]
fn host_stay_surface_missing_stay_id() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_stay(ctx).expect("host stay");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("host.stay.missingStay"));
        });
}

/// L'heure d'arrivée est toujours posée : plus de recommandation « une question au moins ». Une
/// heure d'arrivée tardive illisible bloque, sous son champ.
#[test]
#[serial]
fn publish_readiness_blocks_on_a_bad_late_from() {
    let config = json!({
        "ask_occasion": false,
        "ask_needs": false,
        "ask_guest_count": false,
        "late_from": "25:00"
    });
    MockContext::host()
        .with_property(Property::default())
        .with_config(&config)
        .run(|ctx| {
            let items = publish_readiness(ctx).expect("publishReadiness").items;
            assert!(items.iter().all(|item| item.id != "questions"));
            let item = items
                .iter()
                .find(|item| item.id == "config.late_from")
                .expect("late_from");
            assert!(!item.ok);
        });
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            assert!(publish_readiness(ctx)
                .expect("publishReadiness")
                .items
                .is_empty());
        });
}

/// Un séjour dont l'entrée est à 16 h (Paris), pour que les créneaux soient les mêmes à chaque
/// exécution — ils sortent de l'heure d'arrivée, donc d'une date fixe.
fn stay_opening_at_four(stay_id: Uuid, party_size: Option<u32>) -> StayContext {
    StayContext {
        stay_id,
        checkin_at: Some("2026-10-04T14:00:00Z".parse().expect("arrivée")),
        checkout_at: Some("2026-10-11T09:00:00Z".parse().expect("départ")),
        party_size,
        ..StayContext::default()
    }
}

/// Les créneaux sont bornés par l'heure d'entrée du logement, pas écrits dans le module (§2.20).
///
/// Un sélecteur d'heure libre laissait annoncer 11 h 15 à un logement qui ouvre à 16 h.
#[test]
#[serial]
fn the_arrival_slots_are_bounded_by_the_check_in_hour() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_now(
            "2026-10-03T09:00:00Z"
                .parse()
                .expect("la veille, formulaire ouvert"),
        )
        .with_translation("form.arrival.slot.range", "{from}–{to} h")
        .with_translation("form.arrival.slot.late", "Après {from} h")
        .with_translation("form.arrival.from", "dès {time}")
        .run(|mut ctx| {
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(stay_opening_at_four(stay_id, None));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(!SurfaceAssertions::new(&form).contains_type("TimePicker"));
            let json = serde_json::to_string(&form).expect("json");
            // 14 h UTC, 16 h à Paris : les bornes sont celles du logement.
            assert!(json.contains("16–17 h"), "{json}");
            assert!(json.contains("17–19 h"), "{json}");
            assert!(json.contains("Après 19 h"), "{json}");
            // La valeur envoyée reste une heure : la plateforme la recopie sur le séjour.
            assert!(json.contains(r#""value":"17:00""#), "{json}");
            // Rien n'est coché d'avance sur une question obligatoire : la liste se ferme sur
            // son nom, sans `value`. Un créneau préchoisi part tel quel si personne ne le
            // regarde.
            assert!(
                json.contains(r#""layout":"segmented","name":"arrivalTimeEstimated"}"#),
                "{json}"
            );
            assert!(json.contains("dès 16:00"), "{json}");
        });
}

/// Le nombre de voyageurs ne se demande pas quand la réservation le dit (§2.20).
#[test]
#[serial]
fn the_party_size_is_read_from_the_booking_not_asked() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_now(
            "2026-10-03T09:00:00Z"
                .parse()
                .expect("la veille, formulaire ouvert"),
        )
        .with_translation(
            "form.guestCount.fromBooking",
            "{count}, repris de votre réservation",
        )
        .run(|mut ctx| {
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(stay_opening_at_four(stay_id, Some(3)));

            let form = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&form).contains_type("KeyValue"));
            let json = serde_json::to_string(&form).expect("json");
            assert!(json.contains("3, repris de votre réservation"), "{json}");
            // Et plus de champ de saisie pour ce nombre.
            assert!(!json.contains(r#""name":"guestCount""#), "{json}");
        });
}

/// Sans nombre de voyageurs côté plateforme, la question revient : c'est la seule façon de le
/// savoir.
#[test]
#[serial]
fn without_a_party_size_the_question_comes_back() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_now(
            "2026-10-03T09:00:00Z"
                .parse()
                .expect("la veille, formulaire ouvert"),
        )
        .run(|mut ctx| {
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(stay_opening_at_four(stay_id, None));

            let form = render_guest_form(ctx).expect("guest surface");
            let json = serde_json::to_string(&form).expect("json");
            assert!(json.contains(r#""name":"guestCount""#), "{json}");
        });
}

/// « Envoyer à Claire », et la confirmation qui la nomme (§2.20). Sans prénom servi par la
/// plateforme, les deux retombent sur leur forme impersonnelle.
#[test]
#[serial]
fn the_host_is_named_on_the_button_and_in_the_confirmation() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_now(
            "2026-10-03T09:00:00Z"
                .parse()
                .expect("la veille, formulaire ouvert"),
        )
        .with_translation("form.submit.named", "Envoyer à {host}")
        .with_translation("form.submitUpdate.named", "Mettre à jour pour {host}")
        .with_translation(
            "form.received.title.named",
            "{host} a bien reçu vos informations",
        )
        .with_translation("form.received.message.slot", "Vous êtes attendus {slot}.")
        .with_translation("form.arrival.slot.range", "{from}–{to} h")
        .with_translation("form.arrival.slot.late", "Après {from} h")
        .run(|mut ctx| {
            let stay_id = ctx.guest.as_ref().expect("guest").session_id;
            ctx.stay = Some(stay_opening_at_four(stay_id, Some(2)));
            ctx.host = Some(HostProfile {
                name: "Claire".to_string(),
                ..HostProfile::default()
            });

            let fresh = render_guest_form(ctx.clone()).expect("guest surface");
            let json = serde_json::to_string(&fresh).expect("json");
            assert!(json.contains("Envoyer à Claire"), "{json}");
            assert!(!SurfaceAssertions::new(&fresh).contains_type("Celebration"));

            submit(
                ctx.clone(),
                SubmitArgs {
                    arrival_time_estimated: Some("17:00".into()),
                    ..Default::default()
                },
            )
            .expect("submit");

            let done = render_guest_form(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&done).contains_type("Celebration"));
            let json = serde_json::to_string(&done).expect("json");
            assert!(
                json.contains("Claire a bien reçu vos informations"),
                "{json}"
            );
            // Le créneau annoncé est la seule chose que l'hôte ait reçue de précis.
            assert!(json.contains("Vous êtes attendus 17–19 h."), "{json}");
            assert!(json.contains("🗝️"), "{json}");
            assert!(json.contains("Mettre à jour pour Claire"), "{json}");
        });
}

/// Trois questions de l'hôte : oui / non obligatoire, choix, texte libre — le config tel que le
/// formulaire du dashboard l'envoie (sous-liste `options`, interrupteur en texte).
fn custom_questions_config() -> serde_json::Value {
    json!({
        "custom_questions": [
            { "id": "q1", "label": { "fr": "Draps pour le canapé-lit ?", "en": "Sofa bed sheets?" },
              "type": "yes_no", "required": "true" },
            { "id": "q2", "label": "Petit-déjeuner ?", "type": "choice",
              "options": [{ "label": "Sucré" }, { "label": "Salé" }, { "label": "" }] },
            { "id": "q3", "label": "Votre train ?", "type": "text", "required": false }
        ]
    })
}

#[test]
#[serial]
fn custom_questions_are_checked_with_the_spec_messages() {
    let problems = |config: serde_json::Value| -> Vec<(String, String)> {
        let config: ModuleConfig = serde_json::from_value(config).expect("config");
        config
            .problems()
            .into_iter()
            .map(|(field, error)| (field, error.fr))
            .collect()
    };
    assert!(problems(custom_questions_config()).is_empty());
    assert_eq!(
        problems(json!({ "custom_questions": [
            { "label": "  ", "type": "text" },
            { "label": "Choix ?", "type": "choice", "options": [{ "label": "Un" }] }
        ]})),
        [
            (
                "custom_questions.0.label".to_string(),
                "Écrivez la question.".to_string()
            ),
            (
                "custom_questions.1.options".to_string(),
                "Ajoutez au moins 2 options.".to_string()
            ),
        ]
    );
    let six: Vec<_> = (0..6)
        .map(|i| json!({ "label": format!("Q{i}") }))
        .collect();
    assert_eq!(
        problems(json!({ "custom_questions": six })),
        [(
            "custom_questions".to_string(),
            "5 questions au maximum.".to_string()
        )]
    );

    // Bloquant à la publication, sous le champ dans le formulaire.
    MockContext::host()
        .with_property(Property::default())
        .with_config(&json!({ "custom_questions": [{ "label": "", "type": "yes_no" }] }))
        .run(|ctx| {
            let readiness = publish_readiness(ctx.clone()).expect("publishReadiness");
            let item = readiness
                .items
                .iter()
                .find(|item| item.id == "config.custom_questions.0.label")
                .expect("label check");
            assert!(!item.ok);
            assert_eq!(item.hint.fr, "Écrivez la question.");
            let surface = render_host_main(ctx).expect("host main");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Écrivez la question."), "{json}");
        });
}

#[test]
#[serial]
fn host_main_edits_custom_questions() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_config(&custom_questions_config())
        .with_translation("host.custom.required", "Obligatoire")
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                RETIRED,
            );
            let json = serde_json::to_string(&surface).expect("json");
            // Un tableau ordonnable, chaque question dans une fenêtre.
            assert!(json.contains(r#""panel":"dialog""#), "{json}");
            assert!(json.contains(r#""sortable":true"#), "{json}");
            assert!(json.contains("3 / 5"), "{json}");
            // La ligne : « Obligatoire », ou les options, sous la question.
            assert!(json.contains(r#""sub":"Obligatoire""#), "{json}");
            assert!(json.contains(r#""sub":"Sucré · Salé""#), "{json}");
            assert!(json.contains("custom_questions.0.label"));
            assert!(json.contains("custom_questions.0.required"));
            assert!(json.contains("custom_questions.1.options.5.label"));
            assert!(json.contains("Salé"));
            // Les options ne s'affichent que pour « Choix ».
            assert!(json.contains(r#""visibleWhen":"custom_questions.0.type=choice""#));
            assert!(json.contains(r#""visibleWhen":"custom_questions.1.type=choice""#));
        });
}

/// « Ajouter une question » ouvre une ligne neuve, et plus de ligne à 5.
#[test]
#[serial]
fn adding_a_question_opens_a_row_up_to_five() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_config(&custom_questions_config())
        .run(|mut ctx| {
            ctx.input = json!({ "newQuestions": "neuve,autre,encore" });
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains(r#""id":"neuve""#), "{json}");
            assert!(json.contains("custom_questions.4.label"), "{json}");
            assert!(!json.contains("custom_questions.5.label"), "{json}");
            assert!(json.contains("5 / 5"), "{json}");
            assert!(!json.contains("i18n:host.custom.add"), "{json}");
        });
}

#[test]
#[serial]
fn guest_form_asks_the_custom_questions_after_the_standard_ones() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&custom_questions_config())
        .run(|ctx| {
            let form = render_guest_form(ctx).expect("guest form");
            let json = serde_json::to_string(&form).expect("json");
            for name in ["\"custom0\"", "\"custom1\"", "\"custom2\""] {
                assert!(json.contains(name), "{name} in {json}");
            }
            assert!(json.contains("Draps pour le canapé-lit ?"));
            assert!(json.contains("form.custom.yes"));
            assert!(json.contains("Sucré"));
            // L'option vide ne devient pas une pastille.
            assert!(!json.contains("\"value\":\"2\""), "{json}");
            // Après les questions standard, avant le message à l'hôte.
            let standard = json.find("guestOccasion").expect("occasion");
            let custom = json.find("\"custom0\"").expect("custom0");
            let message = json.find("messageToHost").expect("message");
            assert!(standard < custom && custom < message);
            // La première est obligatoire.
            let first = &json[custom.saturating_sub(300)..custom + 300];
            assert!(first.contains("\"required\":true"), "{first}");
        });
}

#[test]
#[serial]
fn custom_answers_are_kept_and_shown_on_the_stay() {
    reset_test_store();
    let stay_id = Uuid::parse_str("33333333-3333-3333-3333-333333333333").expect("uuid");

    MockContext::guest()
        .with_property(Property::default())
        .with_config(&custom_questions_config())
        .run(|mut ctx| {
            if let Some(guest) = ctx.guest.as_mut() {
                guest.session_id = stay_id;
            }
            // Obligatoire sans réponse : refusé.
            let refused = submit(ctx.clone(), sample_submit()).expect_err("required");
            assert!(refused.to_string().contains("custom_answer_required"));
            submit(
                ctx.clone(),
                SubmitArgs {
                    custom0: Some("yes".into()),
                    custom1: Some("1".into()),
                    custom2: Some("TGV de 18 h 04".into()),
                    ..sample_submit()
                },
            )
            .expect("submit");
            // Rouvert, le formulaire revient rempli.
            let form = render_guest_form(ctx).expect("guest form");
            let json = serde_json::to_string(&form).expect("json");
            assert!(json.contains("TGV de 18 h 04"), "{json}");
        });

    MockContext::host()
        .with_property(Property::default())
        .with_config(&custom_questions_config())
        .run(|mut ctx| {
            ctx.input = json!({ "stayId": stay_id.to_string() });
            let surface = render_host_stay(ctx).expect("host stay");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Draps pour le canapé-lit ?"), "{json}");
            assert!(json.contains("form.custom.yes"));
            assert!(json.contains("Petit-déjeuner ?"));
            assert!(json.contains("Salé"));
            assert!(json.contains("TGV de 18 h 04"));
        });

    // La question retirée depuis : la réponse reste lisible, avec la question posée.
    MockContext::host()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = json!({ "stayId": stay_id.to_string() });
            let json = serde_json::to_string(&render_host_stay(ctx).expect("stay")).expect("json");
            assert!(json.contains("Petit-déjeuner ?"));
        });
}

/// « Arrivée tardive · Marie · 23:30 » dans À venir : une arrivée annoncée à « Arrivée tardive à
/// partir de » ou après (22:00 par défaut), rien avant, rien quand l'hôte a vidé le champ.
#[test]
#[serial]
fn a_late_arrival_is_a_task() {
    use portaki_sdk::contracts::timeline::{TimelineStay, TimelineTasksArgs};
    use pre_arrival_form::late_arrival_tasks;
    reset_test_store();
    let late = Uuid::new_v4();
    let early = Uuid::new_v4();
    for (stay_id, time) in [(late, "23:30"), (early, "18:00")] {
        MockContext::guest()
            .with_property(Property::default())
            .with_now("2026-10-03T09:00:00Z".parse().expect("la veille"))
            .run(|mut ctx| {
                ctx.guest.as_mut().expect("guest").session_id = stay_id;
                ctx.stay = Some(stay_opening_at_four(stay_id, None));
                let args = SubmitArgs {
                    arrival_time_estimated: Some(time.into()),
                    ..sample_submit()
                };
                submit(ctx, args).expect("submit");
            });
    }
    let stay = |id: Uuid, name: &str| TimelineStay {
        id,
        check_in: "2026-10-04T14:00:00Z".parse().expect("arrivée"),
        check_out: "2026-10-11T09:00:00Z".parse().expect("départ"),
        guest_name: name.into(),
        status: "UPCOMING".into(),
    };
    let args = TimelineTasksArgs {
        property_id: Uuid::nil(),
        from: "2026-10-01T00:00:00Z".parse().expect("début"),
        to: "2026-10-08T00:00:00Z".parse().expect("fin"),
        stays: vec![stay(late, "Marie"), stay(early, "Ada")],
    };
    MockContext::host()
        .with_property(Property::default())
        .run(|_| {
            let tasks =
                late_arrival_tasks(&ModuleConfig::default(), &args, "Europe/Paris", "L'Islette");
            assert_eq!(tasks.len(), 1, "{tasks:?}");
            assert_eq!(tasks[0].stay_id, Some(late));
            assert_eq!(tasks[0].title.get("fr"), "Arrivée tardive · Marie · 23:30");
            assert_eq!(tasks[0].context.get("fr"), "L'Islette");

            let cleared = ModuleConfig {
                late_from: Some(String::new()),
                ..ModuleConfig::default()
            };
            assert!(late_arrival_tasks(&cleared, &args, "Europe/Paris", "L'Islette").is_empty());
        });
}

/// « Pré-arrivée non remplie · Marie · arrivée demain » dans À venir : une arrivée sans réponse
/// dont la limite (la veille à 18 h par défaut) tombe dans les 24 h ; rien une fois répondu, rien
/// trop tôt, rien après l'arrivée.
#[test]
#[serial]
fn an_unfilled_form_near_its_deadline_is_a_task() {
    use portaki_sdk::contracts::timeline::{TimelineStay, TimelineTasksArgs};
    use pre_arrival_form::unfilled_form_tasks;
    reset_test_store();
    let (marie, ada, anonymous) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    answer(ada);
    let stay = |id: Uuid, name: &str, check_in: &str| TimelineStay {
        id,
        check_in: check_in.parse().expect("arrivée"),
        check_out: "2026-10-20T09:00:00Z".parse().expect("départ"),
        guest_name: name.into(),
        status: "UPCOMING".into(),
    };
    let args = TimelineTasksArgs {
        property_id: Uuid::nil(),
        from: "2026-10-10T00:00:00Z".parse().expect("début"),
        to: "2026-10-17T00:00:00Z".parse().expect("fin"),
        stays: vec![
            stay(marie, "Marie", "2026-10-13T14:00:00Z"),
            stay(ada, "Ada", "2026-10-13T14:00:00Z"),
            stay(anonymous, " ", "2026-10-14T14:00:00Z"),
        ],
    };
    let at = |now: &str| now.parse().expect("maintenant");
    let config = ModuleConfig {
        show_when: ShowWhen::Confirm,
        ..ModuleConfig::default()
    };
    MockContext::host()
        .with_property(Property::default())
        .run(|_| {
            // Le 12 à 10 h, Paris : la limite de Marie (le 12 à 18 h) est dans les 24 h.
            let tasks =
                unfilled_form_tasks(&config, &args, at("2026-10-12T08:00:00Z"), "Europe/Paris");
            assert_eq!(tasks.len(), 1, "{tasks:?}");
            assert_eq!(tasks[0].stay_id, Some(marie));
            assert_eq!(tasks[0].title.get("fr"), "Pré-arrivée non remplie");
            assert_eq!(tasks[0].context.get("fr"), "Marie · arrivée demain");

            // Le 13 au matin : Marie arrive aujourd'hui, le voyageur sans nom demain.
            let tasks =
                unfilled_form_tasks(&config, &args, at("2026-10-13T07:00:00Z"), "Europe/Paris");
            let contexts: Vec<_> = tasks
                .iter()
                .map(|t| t.context.get("fr").to_string())
                .collect();
            assert_eq!(contexts, ["Marie · arrivée aujourd’hui", "Arrivée demain"]);

            // Limite à J-3 : prévenu trois jours avant, avec la date.
            let j3 = ModuleConfig {
                deadline: Deadline::J3,
                ..config.clone()
            };
            let tasks = unfilled_form_tasks(&j3, &args, at("2026-10-10T08:00:00Z"), "Europe/Paris");
            assert_eq!(tasks.len(), 1, "{tasks:?}");
            assert_eq!(tasks[0].context.get("fr"), "Marie · arrivée le 13/10");

            // Trop tôt, puis après l'arrivée.
            assert!(unfilled_form_tasks(
                &config,
                &args,
                at("2026-10-10T08:00:00Z"),
                "Europe/Paris"
            )
            .is_empty());
            assert!(unfilled_form_tasks(
                &config,
                &args,
                at("2026-10-14T15:00:00Z"),
                "Europe/Paris"
            )
            .is_empty());
        });
}

/// Le voyageur du séjour `stay_id` répond au formulaire.
fn answer(stay_id: Uuid) {
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            if let Some(guest) = ctx.guest.as_mut() {
                guest.session_id = stay_id;
            }
            submit(ctx, sample_submit()).expect("submit");
        });
}

/// « Renvoyer le formulaire » : le canal, l'aperçu et le bouton tant que le formulaire attend une
/// réponse ; une phrase seulement une fois rempli, ou fermé par l'arrivée.
#[test]
#[serial]
fn the_resend_action_offers_the_form_until_it_is_answered_or_closed() {
    reset_test_store();
    let stay_id = Uuid::new_v4();
    let render = |check_in: chrono::DateTime<Utc>| {
        let mut out = String::new();
        MockContext::host()
            .with_property(Property::default())
            .with_config(&json!({ "show_when": "confirm" }))
            .run(|mut ctx| {
                ctx.input = json!({
                    "stayId": stay_id.to_string(),
                    "stay": { "id": stay_id.to_string(), "checkIn": check_in.to_rfc3339() },
                });
                out =
                    serde_json::to_string(&render_host_resend(ctx).expect("resend")).expect("json");
            });
        out
    };

    let open = render(Utc::now() + Duration::days(5));
    assert!(open.contains("host.resend.channel.email"), "{open}");
    assert!(open.contains("host.resend.preview.label"), "{open}");
    assert!(open.contains(r#""name":"resendForm""#), "{open}");
    assert!(open.contains(&stay_id.to_string()), "{open}");

    let closed = render(Utc::now() - Duration::hours(1));
    assert!(closed.contains("host.resend.closed"), "{closed}");
    assert!(!closed.contains("resendForm"), "{closed}");

    answer(stay_id);
    let done = render(Utc::now() + Duration::days(5));
    assert!(done.contains("host.resend.done"), "{done}");
    assert!(!done.contains("resendForm"), "{done}");
}

/// Chaque renvoi part sous un identifiant neuf (la plateforme dédoublonne par identifiant) ; plus
/// rien une fois le formulaire rempli.
#[test]
#[serial]
fn resending_mails_the_guest_each_time_until_answered() {
    reset_test_store();
    let stay_id = Uuid::new_v4();
    let resend = || {
        let mut out = (Ok(()), Vec::new());
        MockContext::host()
            .with_property(Property::default())
            .run_with(|ctx, host| {
                let result = resend_form(ctx, ResendFormArgs { stay_id });
                let ids = host
                    .sent_emails()
                    .into_iter()
                    .map(|mail| mail.email_id)
                    .collect();
                out = (result, ids);
            });
        out
    };

    let (first, first_ids) = resend();
    let (second, second_ids) = resend();
    first.expect("first resend");
    second.expect("second resend");
    assert_eq!(first_ids.len(), 1);
    assert_eq!(second_ids.len(), 1);
    assert!(first_ids[0].starts_with("form-resend-"), "{first_ids:?}");
    assert_ne!(first_ids, second_ids);

    answer(stay_id);
    let (refused, none) = resend();
    assert!(refused.is_err());
    assert!(none.is_empty());
}

/// « Formulaires remplis · N » : les réponses de la période ; l'état vide sans réponse.
#[test]
#[serial]
fn the_stats_tile_counts_the_forms_filled_in_the_period() {
    reset_test_store();
    let summary = || {
        let mut value = String::new();
        MockContext::host()
            .with_property(Property::default())
            .run(|ctx| {
                let args = serde_json::from_value(json!({
                    "propertyId": Uuid::nil(),
                    "period": 30,
                    "key": "forms",
                }))
                .expect("args");
                value = stats_summary(ctx, args).expect("summary").value;
            });
        value
    };
    let card = || {
        let mut out = String::new();
        MockContext::host()
            .with_property(Property::default())
            .run(|ctx| {
                out = serde_json::to_string(&render_host_stats(ctx).expect("card")).expect("json");
            });
        out
    };

    assert_eq!(summary(), "0");
    assert!(card().contains("stats.empty"));

    answer(Uuid::new_v4());
    answer(Uuid::new_v4());
    assert_eq!(summary(), "2");
    assert!(card().contains(r#""value":"2""#), "{}", card());
}
