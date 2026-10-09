//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
// Pas d'`Eq` : les positions sont des flottants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModuleConfig {
    /// « Afficher mon numéro ». Absent : oui — c'était le comportement avant ce réglage.
    #[field(label = "host.showHost.label")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_host: Option<bool>,
    /// `always` ou `hours` ([`AVAILABILITY`]) : joignable 24 h/24, ou sur une plage.
    #[field(
        kind = "select",
        options = ["always", "hours"],
        label = "host.availability.label"
    )]
    pub host_availability: String,
    /// La plage où l'hôte répond, `HH:MM` ; vide : 08:00 – 21:00.
    #[field(label = "host.hours.from")]
    pub host_hours_from: String,
    #[field(label = "host.hours.to")]
    pub host_hours_to: String,
    #[field(label = "config.contacts")]
    pub contacts: Vec<ContactRow>,
    /// Le numéro que l'hôte veut montrer, quand ce n'est pas celui de son compte.
    ///
    /// Plus obligatoire : la plateforme porte désormais le téléphone du profil hôte
    /// (`ctx.host`), et le §2.16 ne demande à l'hôte que ses contacts, sa pharmacie et son
    /// hôpital — pas son propre numéro, qu'il a déjà donné une fois. Rempli, il gagne : c'est un
    /// choix délibéré, par exemple une ligne dédiée aux voyageurs.
    #[field(label = "host.phone.label")]
    pub host_visible_phone: String,
    /// La pharmacie de garde — un numéro, un service, ce que l'hôte veut y mettre.
    ///
    /// Les deux lignes que le §2.16 attend de l'hôte et qui manquaient : elles ferment la carte
    /// en une phrase, sous les numéros qu'on compose.
    #[field(label = "host.pharmacy.label")]
    pub pharmacy: String,
    /// L'hôpital le plus proche, et à quelle distance.
    #[field(label = "host.hospital.label")]
    pub hospital: String,
    /// Les numéros de la pharmacie, de l'hôpital, et le médecin (§2.3) : une rangée qu'on appelle
    /// d'un geste, là où la phrase ne faisait que le dire.
    #[field(label = "host.pharmacyPhone.label")]
    pub pharmacy_phone: String,
    #[field(label = "host.hospitalPhone.label")]
    pub hospital_phone: String,
    #[field(label = "host.doctor.label")]
    pub doctor: String,
    #[field(label = "host.doctorPhone.label")]
    pub doctor_phone: String,
    /// Les positions (spec Urgences §2.3) : un repère sur la Carte du livret. Le sélecteur de
    /// carte les envoie en texte (`"43.5"`) ou en nombre.
    #[field(label = "host.pharmacy.position")]
    #[serde(default, deserialize_with = "coord")]
    pub pharmacy_lat: Option<f64>,
    #[field(label = "host.pharmacy.position")]
    #[serde(default, deserialize_with = "coord")]
    pub pharmacy_lng: Option<f64>,
    #[field(label = "host.pharmacy.address")]
    pub pharmacy_address: String,
    #[field(label = "host.hospital.position")]
    #[serde(default, deserialize_with = "coord")]
    pub hospital_lat: Option<f64>,
    #[field(label = "host.hospital.position")]
    #[serde(default, deserialize_with = "coord")]
    pub hospital_lng: Option<f64>,
    #[field(label = "host.hospital.address")]
    pub hospital_address: String,
}

/// `43.5`, `"43.5"`, ou rien. Un texte illisible ne pose pas de repère, et ne fait pas refuser
/// toute la configuration.
fn coord<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

/// Disponibilité de l'hôte (§2.2), 24 h/24 d'abord : c'est le défaut.
pub const AVAILABILITY: [&str; 2] = ["always", "hours"];
/// La plage proposée quand l'hôte n'en donne pas.
pub const DEFAULT_HOURS: (&str, &str) = ("08:00", "21:00");
/// Combien de contacts la configuration accepte (§2.2).
pub const MAX_CONTACTS: usize = 20;
const LABEL_MAX: usize = 60;
const NOTE_MAX: usize = 120;

/// The old KV blob kept the contacts as a JSON string, `contacts_json`, before the form slots.
fn legacy(mut old: Value) -> Value {
    if let Some(old) = old.as_object_mut() {
        let listed = old
            .remove("contacts_json")
            .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
        if let (Some(contacts), false) = (listed, old.contains_key("contacts")) {
            old.insert("contacts".into(), contacts);
        }
    }
    old
}

impl ModuleConfig {
    /// L'hôte montre son numéro ; oui sans choix enregistré.
    pub fn show_host(&self) -> bool {
        self.show_host.unwrap_or(true)
    }

    /// La plage où l'hôte répond, ou `None` quand il est joignable 24 h/24.
    pub fn host_hours(&self) -> Option<(&str, &str)> {
        if self.host_availability.trim() != "hours" {
            return None;
        }
        fn pick<'a>(value: &'a str, fallback: &'a str) -> &'a str {
            let value = value.trim();
            if value.is_empty() {
                fallback
            } else {
                value
            }
        }
        Some((
            pick(&self.host_hours_from, DEFAULT_HOURS.0),
            pick(&self.host_hours_to, DEFAULT_HOURS.1),
        ))
    }

    /// Ce qui ne va pas, champ par champ — sous le champ dans le formulaire, et dans
    /// `publishReadiness`, pour que les deux disent la même chose.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let text = crate::i18n::text;
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        let mut push = |field: String, error: Option<I18nText>| {
            if let Some(error) = error {
                problems.push((field, error));
            }
        };
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        if self.host_hours().is_some() {
            push(
                "host_hours_from".into(),
                check::time(self.host_hours_from.trim()),
            );
            push(
                "host_hours_to".into(),
                check::time(self.host_hours_to.trim()),
            );
        }
        let filled = self.contacts.iter().filter(|c| !c.is_blank()).count();
        push(
            "contacts".into(),
            (filled > MAX_CONTACTS).then(|| text("host.contacts.tooMany")),
        );
        for (index, contact) in self.contacts.iter().enumerate() {
            if contact.is_blank() {
                continue;
            }
            push(
                format!("contacts.{index}.label"),
                if contact.label.is_blank() {
                    Some(text("host.contact.label.required"))
                } else {
                    too_long(&contact.label, LABEL_MAX)
                },
            );
            let phone = contact.phone.trim();
            push(
                format!("contacts.{index}.phone"),
                if phone.is_empty() {
                    Some(text("host.contact.phone.required"))
                } else {
                    phone_error(phone)
                },
            );
            push(
                format!("contacts.{index}.note"),
                too_long(&contact.note, NOTE_MAX),
            );
        }
        for (key, value) in [
            ("host_visible_phone", &self.host_visible_phone),
            ("pharmacy_phone", &self.pharmacy_phone),
            ("hospital_phone", &self.hospital_phone),
            ("doctor_phone", &self.doctor_phone),
        ] {
            push(key.into(), phone_error(value.trim()));
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_contacts().is_empty() && self.host_visible_phone.trim().is_empty()
    }

    /// The filled rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_contacts(&self) -> Vec<ContactRow> {
        self.contacts
            .iter()
            .filter(|c| !c.phone.trim().is_empty() && !c.label.is_blank())
            .cloned()
            .collect()
    }
}

/// A contact. The form sends `label` and `phone` (and `id`); the platform keeps the rest —
/// `note`, `category`, the label's other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ContactRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub label: I18nText,
    pub phone: String,
    pub note: I18nText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl ContactRow {
    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.label.is_blank() && self.phone.trim().is_empty()
    }
}

/// E.164, ou un numéro court de service (« 18 », « 3237 ») : on l'appelle tel quel, et le
/// refuser ferait retirer au voyageur le numéro qu'il compose en urgence.
fn phone_error(phone: &str) -> Option<I18nText> {
    let phone = compact(phone);
    let short = (2..=6).contains(&phone.len()) && phone.bytes().all(|b| b.is_ascii_digit());
    if short {
        None
    } else {
        check::phone(&phone)
    }
}

/// Un numéro tel qu'on le compare : sans espaces, points ni tirets (« +33 6 12 34 56 78 »).
pub fn compact(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '.' | '-' | '(' | ')'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_a_plain_label() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "contacts": [{ "label": "Pompiers", "phone": "18" }, { "label": "", "phone": "" }],
            "host_visible_phone": "+33 6"
        }))
        .unwrap();
        let contacts = config.parse_contacts();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].label.get("en"), "Pompiers");
    }

    #[test]
    fn legacy_contacts_json_becomes_contacts() {
        let mapped = legacy(json!({
            "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"Ambulance"},"phone":"15","note":{"fr":"Gratuit"},"category":"medical"}]"#,
            "host_visible_phone": "+33 6"
        }));
        assert_eq!(
            mapped,
            json!({
                "contacts": [{ "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15",
                               "note": { "fr": "Gratuit" }, "category": "medical" }],
                "host_visible_phone": "+33 6"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.contacts[0].note.get("en"), "Gratuit");
        assert_eq!(config.contacts[0].category.as_deref(), Some("medical"));
        // Unreadable: nothing to import, nothing invented.
        assert_eq!(legacy(json!({ "contacts_json": "[oops" })), json!({}));
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"Ambulance"},"phone":"15"}]"#,
            "host_visible_phone": "+33 6"
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let contacts = ModuleConfig::load(&ctx).unwrap().parse_contacts();
                assert_eq!(contacts[0].label.get("en"), "Ambulance");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert!(ModuleConfig::load(&ctx).unwrap().contacts.is_empty()));
    }

    /// Un numéro court se compose tel quel ; un numéro long doit porter son indicatif.
    #[test]
    fn phones_are_e164_or_short_service_numbers() {
        assert!(phone_error("18").is_none());
        assert!(phone_error("3237").is_none());
        assert!(phone_error("+33 6 12 34 56 78").is_none());
        assert!(phone_error("06 12 34 56 78").is_some());
        assert!(phone_error("").is_none());
    }

    /// Les erreurs nomment leur champ ; une ligne vide n'en a pas ; la plage n'est vérifiée que
    /// sur plages.
    #[test]
    fn problems_name_their_field() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "host_availability": "hours",
            "host_hours_from": "8h",
            "contacts": [
                { "label": "", "phone": "" },
                { "label": "Paulette", "phone": "" },
                { "label": "", "phone": "+33612345678" }
            ],
            "doctor_phone": "04 93 12 34 56"
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "host_hours_from",
                "contacts.1.phone",
                "contacts.2.label",
                "doctor_phone"
            ]
        );
        assert_eq!(config.host_hours(), Some(("8h", "21:00")));
        assert!(ModuleConfig::default().host_hours().is_none());
        assert!(ModuleConfig::default().show_host());
    }
}
