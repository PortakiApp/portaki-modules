//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use serial_test::serial;

use portaki_test_utils::{MockContext, SurfaceAssertions};
use wifi_guest::{
    render_explore_detail, render_home_card, render_host_main, render_host_stay, ModuleConfig,
    Network, RevealPolicy, WifiSecurity,
};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn network(id: &str, ssid: &str, password: &str) -> Network {
    Network {
        id: id.into(),
        ssid: ssid.into(),
        password: password.into(),
        ..Network::default()
    }
}

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        networks: vec![network("main", "Islette_Guest", "soleil2026")],
        note: Some(I18nText::new("5 GHz conseillé", "Prefer 5 GHz")),
        reveal_policy: RevealPolicy::DayBefore16h,
        ..ModuleConfig::default()
    }
}

fn always_reveal_config() -> ModuleConfig {
    ModuleConfig {
        reveal_policy: RevealPolicy::Always,
        note: None,
        ..sample_config()
    }
}

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_with_config_and_masks_password() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Islette_Guest"));
            assert!(json.contains("i18n:nav.wifi-guest"));
            assert!(!json.contains("soleil2026"));
            assert!(json.contains("••••••"));
        });
}

#[test]
#[serial]
fn detail_shows_security_banner_and_copy_when_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("soleil2026"));
            // La copie est portée par la ligne du mot de passe, pas par un bouton dessous : le
            // livret dessine l'action dans la ligne (§2.2).
            assert!(json.contains("\"copy\":true"));
            assert!(json.contains("i18n:guest.copyPassword"));
        });
}

/// Le code porte la clé en clair dans ses pixels : l'afficher avant la date de révélation rendrait
/// exactement ce que la ligne masquée juste au-dessus refuse — et un téléphone aurait rejoint le
/// réseau, ce qu'aucun masque ne pourrait défaire ensuite.
#[test]
#[serial]
fn no_qr_code_while_the_password_is_still_held_back() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest surface");
            assert!(!SurfaceAssertions::new(&surface).contains_type("QRCode"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(!json.contains("WIFI:T:"));
        });
}

#[test]
#[serial]
fn the_qr_code_carries_the_network_once_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("QRCode"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("WIFI:T:WPA;S:Islette_Guest;P:soleil2026;;"));
        });
}

/// Un portail captif n'a pas de clé. Le code doit dire `nopass`, sinon le téléphone réclame un mot
/// de passe que personne ne possède — et il le fait après avoir scanné, donc l'erreur a l'air de
/// venir du livret.
#[test]
#[serial]
fn an_open_network_is_announced_as_open() {
    let mut portal = network("main", "Islette_Guest", "");
    portal.security = WifiSecurity::Nopass;
    let open = ModuleConfig {
        networks: vec![portal],
        reveal_policy: RevealPolicy::Always,
        ..sample_config()
    };
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&open)
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("guest surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("WIFI:T:nopass;S:Islette_Guest;;"));
            assert!(!json.contains(";P:"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            // `password` : la clé à plat d'avant la liste, lue seulement — le formulaire écrit `networks`.
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &["password"],
            );
            // The password is never sent back to the form: blank keeps it.
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Islette_Guest"));
            assert!(!json.contains("soleil2026"));
        });
}

/// Spec §2 : deux cartes, Réseaux puis Affichage ; une ligne par réseau, ses champs nommés par ligne.
#[test]
#[serial]
fn host_main_has_the_two_cards_of_the_spec() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("i18n:host.section.networks"));
            assert!(json.contains("i18n:host.section.display"));
            assert!(json.contains(r#""name":"networks.0.ssid""#), "{json}");
            assert!(
                json.contains(r#""name":"networks.0.id","value":"main""#),
                "{json}"
            );
            assert!(json.contains("i18n:host.networks.add"));
        });
}

/// A host writing in English: the French message stays; the guest reads their language.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    // Les textes traduits : le message, et le libellé de chaque réseau.
    let localized = config_save::localized_paths(EMISSIONS);
    assert!(localized.iter().any(|p| p == "note"), "{localized:?}");
    let stored = serde_json::json!({
        "networks": [{ "id": "main", "ssid": "Villa", "security": "wpa" }],
        "note": { "fr": "5 GHz conseillé", "en": "Prefer 5 GHz" },
        "reveal_policy": "always"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            assert_eq!(config_save::form_args(&surface)["note"], "Prefer 5 GHz");
            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["note"], stored["note"]);
        });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-GB".into();
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("Prefer 5 GHz"));
            assert!(!json.contains("5 GHz conseillé"));
        });
}

/// La note de l'hôte est le sous-titre de la carte : la redire dans le corps l'affichait deux fois
/// sur le même écran. Dans la feuille, où il n'y a pas de sous-titre, elle reste.
#[test]
#[serial]
fn the_host_note_is_said_once_on_the_card_and_once_in_the_sheet() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let card = serde_json::to_string(&render_home_card(ctx.clone()).expect("carte"))
                .expect("json");
            assert_eq!(card.matches("5 GHz conseillé").count(), 1, "{card}");

            let sheet =
                serde_json::to_string(&render_explore_detail(ctx).expect("feuille")).expect("json");
            assert_eq!(sheet.matches("5 GHz conseillé").count(), 1, "{sheet}");
        });
}

/// Une configuration enregistrée avant le champ `hidden`, et avant la liste, se relit.
///
/// Les hôtes qui ont déjà rempli leur Wi-Fi ont un blob à plat : s'il fallait la liste, leur
/// module cesserait de se charger du jour où on la déclare.
#[test]
fn a_config_saved_before_the_list_still_loads() {
    let stored = serde_json::json!({
        "ssid": "Islette_Guest",
        "password": "soleil2026",
        "security": "wpa",
        "reveal_policy": "always"
    });
    let config: ModuleConfig = serde_json::from_value(stored).expect("relecture");
    let networks = config.networks();
    assert_eq!(networks[0].ssid, "Islette_Guest");
    assert!(!networks[0].hidden, "un réseau d'avant le champ s'annonce");
}

/// Une config d'avant la liste s'affiche au voyageur comme avant, QR compris.
#[test]
#[serial]
fn a_flat_config_still_reaches_the_guest() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&serde_json::json!({ "ssid": "Villa", "password": "soleil2026", "reveal_policy": "always" }))
        .run(|ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("WIFI:T:WPA;S:Villa;P:soleil2026;;"), "{json}");
        });
}

/// Spec cas 5 : deux réseaux, deux blocs dans la feuille, le premier seul sur la carte.
#[test]
#[serial]
fn two_networks_two_blocks_and_the_first_on_the_card() {
    let mut fast = network("fast", "Islette_5G", "rapide2026");
    fast.label = Some(I18nText::new("5 GHz, plus rapide", "5 GHz, faster"));
    let mut main = network("main", "Islette_Guest", "soleil2026");
    main.label = Some(I18nText::new("Principal", "Main"));
    let config = ModuleConfig {
        networks: vec![main, fast],
        ..always_reveal_config()
    };
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let card =
                serde_json::to_string(&render_home_card(ctx.clone()).expect("carte")).unwrap();
            assert!(
                card.contains("Islette_Guest") && !card.contains("Islette_5G"),
                "{card}"
            );
            let sheet =
                serde_json::to_string(&render_explore_detail(ctx).expect("feuille")).unwrap();
            assert!(sheet.contains("Islette_5G") && sheet.contains("5 GHz, plus rapide"));
            assert_eq!(sheet.matches("WIFI:T:").count(), 2);
        });
}

/// `show_qr` coupé : pas de code ; un réseau masqué : `H:true` et l'aide de saisie manuelle.
#[test]
#[serial]
fn the_qr_can_be_turned_off_and_a_hidden_network_says_so() {
    let mut hidden = network("main", "Islette_Guest", "soleil2026");
    hidden.hidden = true;
    let config = ModuleConfig {
        networks: vec![hidden],
        ..always_reveal_config()
    };
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_explore_detail(ctx).expect("feuille")).unwrap();
            assert!(json.contains("H:true"), "{json}");
            assert!(json.contains("i18n:guest.hidden.help"));
        });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&ModuleConfig {
            show_qr: false,
            ..always_reveal_config()
        })
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("feuille");
            assert!(!SurfaceAssertions::new(&surface).contains_type("QRCode"));
        });
}

/// Un mot de passe trop court s'affiche sous son champ dans le tiroir.
#[test]
#[serial]
fn a_short_wpa_password_shows_its_error_under_the_field() {
    let config = ModuleConfig {
        networks: vec![network("main", "Villa", "court")],
        ..sample_config()
    };
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("tiroir")).unwrap();
            assert!(
                json.contains(r#""error":"Le mot de passe WPA fait de 8 à 63 caractères.""#),
                "{json}"
            );
        });
}

/// L'encart séjour dit quand le voyageur voit le mot de passe, jamais le mot de passe.
#[test]
#[serial]
fn the_stay_encart_says_when_never_what() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|mut ctx| {
            ctx.input = serde_json::json!({
                "stayId": "11111111-1111-1111-1111-111111111111",
                "stay": { "checkIn": "2099-08-24T14:00:00Z", "checkOut": "2099-08-29T08:00:00Z" }
            });
            let json = serde_json::to_string(&render_host_stay(ctx).expect("encart")).unwrap();
            assert!(!json.contains("soleil2026"));
            assert!(json.contains("23/08") && json.contains("16:00"), "{json}");
        });
}
