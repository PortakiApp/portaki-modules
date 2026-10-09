//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RevealPolicy {
    Always,
    #[serde(rename = "hours_before_24", alias = "hours_before24")]
    HoursBefore24,
    #[default]
    #[serde(rename = "day_before_16h", alias = "day_before16h")]
    DayBefore16h,
    AtCheckin,
}

impl RevealPolicy {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::HoursBefore24 => "hours_before_24",
            Self::DayBefore16h => "day_before_16h",
            Self::AtCheckin => "at_checkin",
        }
    }

    pub const CHOICE_LIST_WIRE_VALUES: &[&str] =
        &["always", "hours_before_24", "day_before_16h", "at_checkin"];

    pub const ALL: &[RevealPolicy] = &[
        Self::Always,
        Self::HoursBefore24,
        Self::DayBefore16h,
        Self::AtCheckin,
    ];
}

/// How the network is secured — what the Wi-Fi QR code has to announce.
///
/// The guest's phone reads the scheme from the code and configures itself: a `WPA` code offered for
/// an open network makes the phone ask for a password nobody has. WPA covers WPA2 and WPA3, which
/// is why the three options are not four.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WifiSecurity {
    #[default]
    Wpa,
    Wep,
    /// Open network — a captive portal, typically.
    Nopass,
}

impl WifiSecurity {
    /// The value the host form sends back, which is what serde reads — lowercase, unlike the
    /// scheme's own `T:` spelling. Confusing the two silently rejects the saved choice.
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Wpa => "wpa",
            Self::Wep => "wep",
            Self::Nopass => "nopass",
        }
    }

    /// The `T:` value of the `WIFI:` scheme.
    pub const fn as_qr_type(self) -> &'static str {
        match self {
            Self::Wpa => "WPA",
            Self::Wep => "WEP",
            Self::Nopass => "nopass",
        }
    }
}

/// One network the guest can join (spec Wi-Fi §2.1). The first is the one on the home card.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Network {
    /// Keeps a row's password through reorders and saves: the platform matches rows by id.
    pub id: String,
    pub ssid: String,
    /// Required once there is more than one network (« 5 GHz, plus rapide »).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<I18nText>,
    pub security: WifiSecurity,
    #[field(secret)]
    pub password: String,
    /// Réseau masqué : sans le `H:true` du code QR, le téléphone cherche un réseau qui ne
    /// s'annonce pas et n'arrive à rien.
    pub hidden: bool,
}

/// At most three networks (spec §2.1) — a bound, never fixed slots.
pub const MAX_NETWORKS: usize = 3;
/// `ssid` is 32 octets at most: the 802.11 limit, in bytes, not characters.
pub const SSID_MAX_BYTES: usize = 32;
pub const LABEL_MAX: usize = 30;
pub const NOTE_MAX: usize = 200;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(
        structured,
        required,
        reveal(guest_pre_arrival, guest_stay),
        label = "host.networks.label"
    )]
    pub networks: Vec<Network>,
    /// QR code de connexion (défaut vrai).
    #[field(label = "host.showQr.label")]
    pub show_qr: bool,
    /// Le message de l'hôte : la légende de la carte, et en tête de la feuille (portail captif).
    #[field(label = "host.note.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<I18nText>,
    #[field(
        kind = "select",
        options = ["always", "hours_before_24", "day_before_16h", "at_checkin"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: RevealPolicy,

    // ── Avant la liste de réseaux : un seul réseau, à plat. Lu tant que `networks` est vide ;
    // le prochain enregistrement de l'hôte écrit `networks`. `password` reste déclaré secret :
    // sans cela, la plateforme cesserait de le déchiffrer et le module recevrait le chiffré.
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay),
        label = "host.password.label"
    )]
    #[serde(rename = "password", skip_serializing_if = "String::is_empty")]
    pub legacy_password: String,
    #[serde(rename = "ssid", skip_serializing_if = "String::is_empty")]
    pub legacy_ssid: String,
    #[serde(rename = "security", skip_serializing_if = "Option::is_none")]
    pub legacy_security: Option<WifiSecurity>,
    #[serde(rename = "hidden", skip_serializing_if = "std::ops::Not::not")]
    pub legacy_hidden: bool,
    #[serde(rename = "hint", skip_serializing_if = "Option::is_none")]
    pub legacy_hint: Option<I18nText>,
    #[serde(rename = "connection_steps", skip_serializing_if = "Option::is_none")]
    pub legacy_connection_steps: Option<I18nText>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            networks: Vec::new(),
            show_qr: true,
            note: None,
            reveal_policy: RevealPolicy::default(),
            legacy_password: String::new(),
            legacy_ssid: String::new(),
            legacy_security: None,
            legacy_hidden: false,
            legacy_hint: None,
            legacy_connection_steps: None,
        }
    }
}

/// The id given to the network read from a config of before the list.
pub const LEGACY_NETWORK_ID: &str = "legacy";

/// The old KV blob spelled two policies the pre-rename way (`hours_before24`, `day_before16h`),
/// which are not options of the select: the platform would not import them.
fn legacy(mut old: Value) -> Value {
    if let Some(policy) = old.get_mut("reveal_policy") {
        if let Ok(parsed) = serde_json::from_value::<RevealPolicy>(policy.clone()) {
            *policy = parsed.as_wire().into();
        }
    }
    old
}

impl ModuleConfig {
    /// The networks to show and edit: the list, or the one network of a config of before it.
    pub fn networks(&self) -> Vec<Network> {
        if self.networks.iter().any(|n| !n.ssid.trim().is_empty()) {
            return self.networks.clone();
        }
        if self.legacy_ssid.trim().is_empty() {
            return self.networks.clone();
        }
        vec![Network {
            id: LEGACY_NETWORK_ID.into(),
            ssid: self.legacy_ssid.clone(),
            label: None,
            security: self.legacy_security.unwrap_or_default(),
            password: self.legacy_password.clone(),
            hidden: self.legacy_hidden,
        }]
    }

    /// Nothing the guest could join yet.
    pub fn is_empty(&self) -> bool {
        self.networks().iter().all(|n| n.ssid.trim().is_empty())
    }

    /// The host's message, or the old hint, then steps: they were the message before it.
    pub fn note_text(&self, locale: &str) -> Option<String> {
        if let Some(note) = text_in(self.note.as_ref(), locale) {
            return Some(note.to_string());
        }
        let old: Vec<&str> = [
            text_in(self.legacy_hint.as_ref(), locale),
            text_in(self.legacy_connection_steps.as_ref(), locale),
        ]
        .into_iter()
        .flatten()
        .collect();
        (!old.is_empty()).then(|| old.join("\n\n"))
    }

    /// The network read from a config of before the list: its password must be typed again,
    /// the platform has no row to keep it from.
    pub fn is_legacy(&self) -> bool {
        self.networks()
            .first()
            .is_some_and(|n| n.id == LEGACY_NETWORK_ID)
    }
}

/// What is wrong with one field, for the drawer (`Field::error`) and `publishReadiness`.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// `config.<key>` — the field the dashboard scrolls to.
    pub id: String,
    /// The i18n key of the field's label.
    pub label: &'static str,
    pub message: I18nText,
    /// Blocks « Publier » ; otherwise a warning.
    pub blocking: bool,
}

impl ModuleConfig {
    /// Every rule of the spec, in the drawer's order.
    pub fn problems(&self) -> Vec<Problem> {
        let mut problems = Vec::new();
        let networks = self.networks();
        let filled: Vec<&Network> = networks
            .iter()
            .filter(|n| !n.ssid.trim().is_empty())
            .collect();
        if filled.is_empty() {
            problems.push(problem(
                "config.networks",
                "host.networks.label",
                "config.networks.empty",
                true,
            ));
        }
        if networks.len() > MAX_NETWORKS {
            problems.push(problem(
                "config.networks",
                "host.networks.label",
                "config.networks.max",
                true,
            ));
        }
        for (index, network) in networks.iter().enumerate() {
            let at = |key: &str| format!("config.networks[{index}].{key}");
            let ssid = network.ssid.trim();
            if ssid.is_empty() && networks.len() > 1 {
                problems.push(problem(
                    &at("ssid"),
                    "host.ssid.label",
                    "config.ssid.required",
                    true,
                ));
            }
            if ssid.len() > SSID_MAX_BYTES {
                problems.push(problem(
                    &at("ssid"),
                    "host.ssid.label",
                    "config.ssid.max",
                    true,
                ));
            }
            let label = network
                .label
                .as_ref()
                .map(|l| l.get("fr").trim())
                .unwrap_or_default();
            if networks.len() > 1 && label.is_empty() {
                problems.push(problem(
                    &at("label"),
                    "host.label.label",
                    "config.label.required",
                    true,
                ));
            }
            if let Some(message) = network
                .label
                .as_ref()
                .and_then(|l| check::max_chars(l.get("fr"), LABEL_MAX))
            {
                problems.push(Problem {
                    id: at("label"),
                    label: "host.label.label",
                    message,
                    blocking: true,
                });
            }
            if let Some(key) = password_rule(network) {
                problems.push(problem(&at("password"), "host.password.label", key, true));
            } else if network.security != WifiSecurity::Nopass
                && network.password.trim() != network.password
            {
                problems.push(problem(
                    &at("password"),
                    "host.password.label",
                    "config.password.space",
                    false,
                ));
            }
        }
        if let Some(message) = self
            .note
            .as_ref()
            .and_then(|n| check::max_chars(n.get("fr"), NOTE_MAX))
        {
            problems.push(Problem {
                id: "config.note".into(),
                label: "host.note.label",
                message,
                blocking: true,
            });
        }
        problems
    }

    /// The problem of one field, for its `Field::error`.
    pub fn error_of(&self, id: &str, locale: &str) -> Option<String> {
        self.problems()
            .into_iter()
            .find(|p| p.id == id && p.blocking)
            .map(|p| p.message.get(locale).to_string())
    }
}

/// WPA : 8 à 63 caractères ; WEP : 5 ou 13 ; ouvert : rien. `None` when the password fits.
fn password_rule(network: &Network) -> Option<&'static str> {
    let length = network.password.chars().count();
    match network.security {
        WifiSecurity::Wpa if !(8..=63).contains(&length) => Some("config.password.wpa"),
        WifiSecurity::Wep if length != 5 && length != 13 => Some("config.password.wep"),
        _ => None,
    }
}

fn problem(id: &str, label: &'static str, message: &str, blocking: bool) -> Problem {
    Problem {
        id: id.into(),
        label,
        message: crate::i18n::text(message),
        blocking,
    }
}

fn text_in<'a>(text: Option<&'a I18nText>, locale: &str) -> Option<&'a str> {
    text.map(|text| text.get(locale).trim())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn default_reveal_policy_is_day_before_16h() {
        assert_eq!(
            ModuleConfig::default().reveal_policy,
            RevealPolicy::DayBefore16h
        );
    }

    #[test]
    fn reveal_policy_choice_list_values_deserialize() {
        for wire in RevealPolicy::CHOICE_LIST_WIRE_VALUES {
            let parsed: RevealPolicy = serde_json::from_value(json!(wire)).unwrap_or_else(|e| {
                panic!("ChoiceList reveal_policy value {wire:?} must deserialize: {e}")
            });
            assert_eq!(parsed.as_wire(), *wire);
        }
    }

    #[test]
    fn legacy_renames_pre_rename_policies_and_keeps_plain_texts() {
        let mapped = legacy(json!({
            "ssid": "Villa",
            "hint": "5 GHz au salon",
            "connection_steps": "Choisir Villa",
            "reveal_policy": "hours_before24"
        }));
        assert_eq!(
            mapped,
            json!({
                "ssid": "Villa",
                "hint": "5 GHz au salon",
                "connection_steps": "Choisir Villa",
                "reveal_policy": "hours_before_24"
            })
        );
        assert_eq!(
            legacy(json!({ "reveal_policy": "day_before16h" }))["reveal_policy"],
            "day_before_16h"
        );
        // Already an option, or unknown: left as is.
        assert_eq!(
            legacy(json!({ "reveal_policy": "at_checkin" })),
            json!({ "reveal_policy": "at_checkin" })
        );
        assert_eq!(
            legacy(json!({ "reveal_policy": "weekly" })),
            json!({ "reveal_policy": "weekly" })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(
            config.note_text("en").as_deref(),
            Some("5 GHz au salon\n\nChoisir Villa"),
            "l'astuce puis les étapes deviennent le message"
        );
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({ "ssid": "Villa", "reveal_policy": "hours_before24" });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.reveal_policy, RevealPolicy::HoursBefore24);
                assert_eq!(config.networks()[0].ssid, "Villa");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default());
            });
    }

    fn wpa(ssid: &str, password: &str) -> Network {
        Network {
            id: ssid.into(),
            ssid: ssid.into(),
            password: password.into(),
            ..Network::default()
        }
    }

    fn ids(config: &ModuleConfig) -> Vec<(String, bool)> {
        config
            .problems()
            .into_iter()
            .map(|p| (p.id, p.blocking))
            .collect()
    }

    /// Spec §8 : les champs uniques `ssid` / `password` deviennent un réseau, `security` absent → WPA.
    #[test]
    fn a_config_of_before_the_list_reads_as_one_network() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "ssid": "Villa", "password": "soleil2026", "hidden": true
        }))
        .unwrap();
        let networks = config.networks();
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].ssid, "Villa");
        assert_eq!(networks[0].password, "soleil2026");
        assert_eq!(networks[0].security, WifiSecurity::Wpa);
        assert!(networks[0].hidden);
        assert!(config.is_legacy());
        assert!(config.problems().is_empty());

        // Une fois la liste enregistrée, elle seule compte.
        let saved = ModuleConfig {
            networks: vec![wpa("Villa-5G", "soleil2026")],
            ..config
        };
        assert_eq!(saved.networks()[0].ssid, "Villa-5G");
        assert!(!saved.is_legacy());
    }

    #[test]
    fn a_new_install_shows_the_qr_and_has_no_network() {
        let config = ModuleConfig::default();
        assert!(config.show_qr);
        assert!(config.is_empty());
        assert_eq!(ids(&config), [("config.networks".to_string(), true)]);
        assert_eq!(
            config.problems()[0].message.get("fr"),
            "Ajoutez au moins un réseau."
        );
    }

    #[test]
    fn each_rule_of_the_spec_says_its_message() {
        let mut long = wpa(&"é".repeat(17), "court");
        long.label = Some(I18nText::new("x".repeat(31), "x"));
        let config = ModuleConfig {
            networks: vec![long, wpa("Villa", " soleil2026 ")],
            note: Some(I18nText::new("n".repeat(201), "n")),
            ..ModuleConfig::default()
        };
        let messages: Vec<(String, String, bool)> = config
            .problems()
            .into_iter()
            .map(|p| (p.id, p.message.get("fr").to_string(), p.blocking))
            .collect();
        assert_eq!(
            messages,
            [
                (
                    "config.networks[0].ssid".into(),
                    "Le nom du réseau fait 32 caractères au maximum.".into(),
                    true
                ),
                (
                    "config.networks[0].label".into(),
                    "30 caractères au maximum.".into(),
                    true
                ),
                (
                    "config.networks[0].password".into(),
                    "Le mot de passe WPA fait de 8 à 63 caractères.".into(),
                    true
                ),
                (
                    "config.networks[1].label".into(),
                    "Donnez un libellé pour distinguer les réseaux.".into(),
                    true
                ),
                (
                    "config.networks[1].password".into(),
                    "Le mot de passe se termine par une espace : est-ce voulu ?".into(),
                    false
                ),
                (
                    "config.note".into(),
                    "200 caractères au maximum.".into(),
                    true
                ),
            ]
        );
        assert_eq!(
            config.error_of("config.networks[0].ssid", "fr").as_deref(),
            Some("Le nom du réseau fait 32 caractères au maximum.")
        );
        assert_eq!(
            config.error_of("config.networks[1].password", "fr"),
            None,
            "un avertissement n'est pas une erreur"
        );
    }

    #[test]
    fn wep_takes_five_or_thirteen_and_an_open_network_none() {
        let mut wep = wpa("Vieux", "abcde");
        wep.security = WifiSecurity::Wep;
        assert!(ModuleConfig {
            networks: vec![wep.clone()],
            ..ModuleConfig::default()
        }
        .problems()
        .is_empty());
        wep.password = "abcdef".into();
        assert_eq!(
            ModuleConfig {
                networks: vec![wep],
                ..ModuleConfig::default()
            }
            .problems()[0]
                .message
                .get("fr"),
            "Le mot de passe WEP fait 5 ou 13 caractères."
        );
        let mut open = wpa("Portail", "");
        open.security = WifiSecurity::Nopass;
        assert!(ModuleConfig {
            networks: vec![open],
            ..ModuleConfig::default()
        }
        .problems()
        .is_empty());
    }

    #[test]
    fn three_networks_at_most() {
        let networks = (0..4).map(|i| {
            let mut n = wpa(&format!("N{i}"), "soleil2026");
            n.label = Some(I18nText::new(format!("L{i}"), format!("L{i}")));
            n
        });
        let config = ModuleConfig {
            networks: networks.collect(),
            ..ModuleConfig::default()
        };
        assert_eq!(
            config.problems()[0].message.get("fr"),
            "3 réseaux au maximum."
        );
    }
}
