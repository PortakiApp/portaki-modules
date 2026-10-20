//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::bundle_text;
use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::sdui::hours;
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
    /// La plage où l'hôte répond, la valeur d'un `TimeRange` : `HH:MM-HH:MM`. Vide : celle des
    /// deux champs d'avant, sinon 08:00 – 21:00.
    #[field(label = "host.hours.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub host_hours: String,
    /// Les deux champs d'avant `host_hours`, lus tant qu'une plage n'a pas été choisie : plus
    /// dessinés, la plateforme garde leur valeur (`updateConfig` fusionne).
    #[field(label = "host.hours.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub host_hours_from: String,
    #[field(label = "host.hours.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub host_hours_to: String,
    /// Copiés vers un autre logement, marqués « À vérifier » (`local`) ; fusionnés ligne par
    /// ligne sur leur `id`, dans l'ordre reçu : le tableau se réordonne.
    #[field(local, item_id = "id", label = "config.contacts")]
    pub contacts: Vec<ContactRow>,
    /// Le numéro que l'hôte veut montrer, quand ce n'est pas celui de son compte.
    ///
    /// Une surface hôte ne lit pas le profil de l'hôte de façon sûre : le champ reste, et le
    /// livret retombe sur le téléphone du profil (`ctx.host`) quand il est vide. Rempli, il gagne :
    /// c'est un choix délibéré, par exemple une ligne dédiée aux voyageurs.
    #[field(label = "host.phone.label")]
    pub host_visible_phone: String,
    // §2.3 Santé : propres au lieu, jamais copiés vers un autre logement (`local`).
    /// La pharmacie : son nom, non traduit.
    #[field(local, label = "host.pharmacy.label")]
    pub pharmacy: String,
    #[field(local, label = "host.pharmacyPhone.label")]
    pub pharmacy_phone: String,
    /// Les horaires de la pharmacie, la valeur d'un `WeeklyHours` (`mon=09:00-19:30;…`) : le
    /// livret dit « Ouverte jusqu'à 19:30 » ou « Fermée ».
    #[field(local, label = "host.pharmacyHours.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pharmacy_hours: String,
    /// Les positions (spec Urgences §2.3) : un repère sur la Carte du livret. Le sélecteur de
    /// carte les envoie en texte (`"43.5"`) ou en nombre.
    #[field(local, label = "host.pharmacy.position")]
    #[serde(default, deserialize_with = "coord")]
    pub pharmacy_lat: Option<f64>,
    #[field(local, label = "host.pharmacy.position")]
    #[serde(default, deserialize_with = "coord")]
    pub pharmacy_lng: Option<f64>,
    #[field(local, label = "host.pharmacy.address")]
    pub pharmacy_address: String,
    /// L'hôpital avec urgences.
    #[field(local, label = "host.hospital.label")]
    pub hospital: String,
    #[field(local, label = "host.hospitalPhone.label")]
    pub hospital_phone: String,
    #[field(local, label = "host.hospital.position")]
    #[serde(default, deserialize_with = "coord")]
    pub hospital_lat: Option<f64>,
    #[field(local, label = "host.hospital.position")]
    #[serde(default, deserialize_with = "coord")]
    pub hospital_lng: Option<f64>,
    #[field(local, label = "host.hospital.address")]
    pub hospital_address: String,
    /// Le médecin ; sa position est facultative.
    #[field(local, label = "host.doctor.label")]
    pub doctor: String,
    #[field(local, label = "host.doctorPhone.label")]
    pub doctor_phone: String,
    #[field(local, label = "host.doctor.position")]
    #[serde(
        default,
        deserialize_with = "coord",
        skip_serializing_if = "Option::is_none"
    )]
    pub doctor_lat: Option<f64>,
    #[field(local, label = "host.doctor.position")]
    #[serde(
        default,
        deserialize_with = "coord",
        skip_serializing_if = "Option::is_none"
    )]
    pub doctor_lng: Option<f64>,
    #[field(local, label = "host.doctor.address")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub doctor_address: String,
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

/// `08:00-21:00`, la valeur d'un `TimeRange`.
fn range(from: &str, to: &str) -> String {
    format!("{from}-{to}")
}

/// Disponibilité de l'hôte (§2.2), 24 h/24 d'abord : c'est le défaut.
pub const AVAILABILITY: [&str; 2] = ["always", "hours"];
/// La plage proposée quand l'hôte n'en donne pas.
pub const DEFAULT_HOURS: (&str, &str) = ("08:00", "21:00");
/// Combien de contacts la configuration accepte (§2.2).
pub const MAX_CONTACTS: usize = 20;
const LABEL_MAX: usize = 60;
const NOTE_MAX: usize = 120;

/// The old KV blob kept the contacts as a JSON string, `contacts_json`, before the form slots;
/// and the host's hours as two fields, `host_hours_from` / `host_hours_to`, before `TimeRange`.
fn legacy(mut old: Value) -> Value {
    if let Some(old) = old.as_object_mut() {
        let half = |old: &serde_json::Map<String, Value>, key: &str| {
            old.get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .to_string()
        };
        let (from, to) = (half(old, "host_hours_from"), half(old, "host_hours_to"));
        if !old.contains_key("host_hours") && !(from.is_empty() && to.is_empty()) {
            old.remove("host_hours_from");
            old.remove("host_hours_to");
            old.insert("host_hours".into(), range(&from, &to).into());
        }
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
    ///
    /// `host_hours` d'abord ; sans lui, les deux champs d'avant ; sans eux, 08:00 – 21:00. Une
    /// plage illisible prend aussi le défaut : l'erreur est sous le champ, le livret reste juste.
    pub fn host_hours(&self) -> Option<(String, String)> {
        if self.host_availability.trim() != "hours" {
            return None;
        }
        hours::parse_range(&self.host_hours_wire())
            .or_else(|| Some((DEFAULT_HOURS.0.into(), DEFAULT_HOURS.1.into())))
    }

    /// La valeur à donner au `TimeRange` : la plage choisie, ou celle des deux champs d'avant.
    pub fn host_hours_wire(&self) -> String {
        let hours = self.host_hours.trim();
        if !hours.is_empty() {
            return hours.to_string();
        }
        let (from, to) = (self.host_hours_from.trim(), self.host_hours_to.trim());
        if from.is_empty() && to.is_empty() {
            String::new()
        } else {
            range(
                if from.is_empty() {
                    DEFAULT_HOURS.0
                } else {
                    from
                },
                if to.is_empty() { DEFAULT_HOURS.1 } else { to },
            )
        }
    }

    /// Ce qui ne va pas, champ par champ — sous le champ dans le formulaire, et dans
    /// `publishReadiness`, pour que les deux disent la même chose.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        let mut push = |field: String, error: Option<I18nText>| {
            if let Some(error) = error {
                problems.push((field, error));
            }
        };
        if self.host_hours().is_some() {
            let wire = self.host_hours_wire();
            let (start, end) = wire.split_once('-').unwrap_or((&wire, ""));
            push(
                "host_hours".into(),
                check::time_range(start.trim(), end.trim()),
            );
        }
        let filled = self.contacts.iter().filter(|c| !c.is_blank()).count();
        push(
            "contacts".into(),
            (filled > MAX_CONTACTS).then(|| bundle_text!("host.contacts.tooMany")),
        );
        for (index, contact) in self.contacts.iter().enumerate() {
            for (key, error) in contact.problems() {
                push(format!("contacts.{index}.{key}"), Some(error));
            }
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

    /// Ce qui ne va pas sur cette ligne, sous-clé par sous-clé ; rien sur une ligne vide.
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        let phone = self.phone.trim();
        [
            (
                "label",
                if self.label.is_blank() {
                    Some(bundle_text!("host.contact.label.required"))
                } else {
                    too_long(&self.label, LABEL_MAX)
                },
            ),
            (
                "phone",
                if phone.is_empty() {
                    Some(bundle_text!("host.contact.phone.required"))
                } else {
                    phone_error(phone)
                },
            ),
            ("note", too_long(&self.note, NOTE_MAX)),
        ]
        .into_iter()
        .filter_map(|(key, error)| Some((key, error?)))
        .collect()
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

    /// Les deux champs d'avant deviennent la plage du `TimeRange` : dans l'import KV
    /// (`legacy`), et à la lecture d'une config que la plateforme tient déjà.
    #[test]
    fn the_old_hours_become_a_time_range() {
        assert_eq!(
            legacy(json!({ "host_hours_from": "09:00", "host_hours_to": "20:00" })),
            json!({ "host_hours": "09:00-20:00" })
        );
        let held: ModuleConfig = serde_json::from_value(json!({
            "host_availability": "hours",
            "host_hours_from": "09:00"
        }))
        .unwrap();
        assert_eq!(held.host_hours_wire(), "09:00-21:00");
        assert_eq!(
            held.host_hours(),
            Some(("09:00".to_string(), "21:00".to_string()))
        );
        // La plage choisie gagne sur les champs d'avant.
        let chosen = ModuleConfig {
            host_hours: "07:30-22:00".into(),
            ..held
        };
        assert_eq!(chosen.host_hours().unwrap().0, "07:30");
        assert!(chosen.problems().is_empty());
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
            "host_hours": "8h-21:00",
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
                "host_hours",
                "contacts.1.phone",
                "contacts.2.label",
                "doctor_phone"
            ]
        );
        // Illisible : l'erreur est sous le champ, le livret garde la plage par défaut.
        assert_eq!(
            config.host_hours(),
            Some(("08:00".to_string(), "21:00".to_string()))
        );
        assert!(ModuleConfig::default().host_hours().is_none());
        assert!(ModuleConfig::default().show_host());
    }
}
