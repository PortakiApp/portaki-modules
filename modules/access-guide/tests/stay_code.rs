//! « À chaque séjour » (spec Accès §1, §2.2, §9 cas 2) : le code d'un séjour, lu dans les secrets
//! du séjour — jamais le code fixe d'avant —, changé par l'hôte depuis la fiche du séjour.

use chrono::{DateTime, Utc};
use portaki_sdk::capability;
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::timeline::{TimelineStay, TimelineTasksArgs};
use portaki_sdk::host::with_host;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

use access_guide::{
    missing_stay_code_tasks, render_home_card, render_host_change_code, render_host_stay,
    set_stay_code, HostConfig, StayCodeArgs,
};
use portaki_test_utils::MockContext;

const STAY: Uuid = Uuid::from_u128(0x1a2b);
const OTHER: Uuid = Uuid::from_u128(0x3c4d);

fn at(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .unwrap()
        .with_timezone(&Utc)
}

/// Une boîte à clés « À chaque séjour », codes visibles dès la réservation, l'ancien code fixe
/// encore en base.
fn per_stay() -> HostConfig {
    HostConfig {
        primary_method: "keybox".into(),
        code_scope: "stay".into(),
        keybox_code: "1111".into(),
        reveal_policy: "always".into(),
        address: "1 rue Test".into(),
        ..HostConfig::default()
    }
}

fn guest_home(seed: &[(Uuid, &str)]) -> String {
    // `guest()` porte déjà le stockage ; `with_capabilities` effacerait le voyageur.
    let mut mock = MockContext::guest()
        .with_config(&per_stay())
        .with_stay(StayContext {
            stay_id: STAY,
            ..StayContext::default()
        });
    for (stay, code) in seed {
        mock = mock.with_stay_secret(*stay, "entry_code", *code);
    }
    mock.run(|ctx| serde_json::to_string(&render_home_card(ctx).expect("surface")).unwrap())
}

#[test]
#[serial]
fn the_guest_reads_the_code_of_their_own_stay_only() {
    let json = guest_home(&[(STAY, "4821"), (OTHER, "7777")]);
    assert!(json.contains("4821"));
    assert!(!json.contains("7777"));
    assert!(!json.contains("1111"), "jamais le code fixe d'avant");
    // Sans code saisi : l'attente, pas le code fixe.
    let none = guest_home(&[(OTHER, "7777")]);
    assert!(!none.contains("1111") && !none.contains("7777"));
}

fn set(guest: bool, args: StayCodeArgs) -> (portaki_sdk::Result<()>, Option<String>) {
    let mock = if guest {
        MockContext::guest().with_stay(StayContext {
            stay_id: STAY,
            ..StayContext::default()
        })
    } else {
        MockContext::host().with_capabilities(&[capability::core::STORAGE])
    };
    mock.with_config(&per_stay()).run_with(|ctx, host| {
        let result = set_stay_code(ctx, args);
        let stored = host
            .stay_secrets()
            .get(&(STAY.to_string(), "entry_code".to_string()))
            .cloned();
        (result, stored)
    })
}

#[test]
#[serial]
fn the_host_sets_or_generates_the_code_of_a_stay() {
    let typed = |code: &str| StayCodeArgs {
        stay_id: STAY,
        code: Some(code.into()),
        generate: false,
    };
    let (ok, stored) = set(false, typed(" 48 21 "));
    assert!(ok.is_ok());
    assert_eq!(stored.as_deref(), Some("4821"));
    // 3 à 12 caractères, chiffres, lettres, # ou * (§2.2).
    let (bad, stored) = set(false, typed("12"));
    assert!(bad.is_err() && stored.is_none());
    let (generated, stored) = set(
        false,
        StayCodeArgs {
            stay_id: STAY,
            code: None,
            generate: true,
        },
    );
    assert!(generated.is_ok());
    let code = stored.expect("code généré");
    assert!(code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()));
    // Le voyageur ne change pas son code.
    let (refused, _) = set(true, typed("4821"));
    assert!(refused.is_err());
}

fn host_surface(code: Option<&str>, render: fn(portaki_sdk::Context) -> String) -> String {
    let mut mock = MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            reveal_policy: "day_before_16h".into(),
            ..per_stay()
        })
        .with_now(at("2099-08-20T10:00:00Z"));
    if let Some(code) = code {
        mock = mock.with_stay_secret(STAY, "entry_code", code);
    }
    let (mut ctx, host) = mock.build();
    ctx.input = json!({ "stayId": STAY.to_string(),
                        "stay": { "checkIn": "2099-08-24T14:00:00Z",
                                  "checkOut": "2099-08-29T08:00:00Z" } });
    with_host(host, ctx.clone(), || render(ctx))
}

#[test]
#[serial]
fn the_stay_card_shows_the_stay_code_or_says_it_is_missing() {
    let card = |ctx| serde_json::to_string(&render_host_stay(ctx).expect("stay")).unwrap();
    let shown = host_surface(Some("4821"), card);
    assert!(shown.contains("4821") && shown.contains("i18n:host.stay.code"));
    assert!(!shown.contains("1111"));
    let missing = host_surface(None, card);
    assert!(missing.contains("i18n:host.stay.missingStayCode"));
}

#[test]
#[serial]
fn the_change_code_modal_saves_through_one_command() {
    let modal = |ctx| serde_json::to_string(&render_host_change_code(ctx).expect("modal")).unwrap();
    let json = host_surface(None, modal);
    assert!(json.contains("setStayCode"));
    assert!(json.contains("i18n:host.changeCode.generate"));
    // Pas encore révélé (la veille à 16 h) : aucun avertissement.
    assert!(!json.contains("i18n:host.changeCode.revealed"));
    // Le code fixe : la modale renvoie aux réglages.
    let fixed = MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            code_scope: "fixed".into(),
            ..per_stay()
        })
        .run(|ctx| serde_json::to_string(&render_host_change_code(ctx).unwrap()).unwrap());
    assert!(fixed.contains("i18n:host.changeCode.unavailable"));
}

#[test]
fn a_missing_stay_code_is_a_task_for_that_stay_only() {
    let stay = |id: Uuid, check_in: &str| TimelineStay {
        id,
        check_in: at(check_in),
        check_out: at(check_in) + chrono::Duration::days(3),
        guest_name: "Marie".into(),
        status: "UPCOMING".into(),
    };
    let args = TimelineTasksArgs {
        property_id: Uuid::nil(),
        from: at("2026-08-17T00:00:00Z"),
        to: at("2026-08-31T00:00:00Z"),
        stays: vec![
            stay(STAY, "2026-08-21T14:00:00Z"),
            stay(OTHER, "2026-08-25T14:00:00Z"),
        ],
    };
    let config = per_stay().to_model("fr");
    let now = at("2026-08-19T12:00:00Z");
    let tasks = missing_stay_code_tasks(&config, &args, now, "Europe/Paris", &[STAY]);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].stay_id, Some(OTHER));
    // Le code fixe ne compte plus : rien ne manque côté réglages.
    assert!(!config.entry_code_missing());
}
