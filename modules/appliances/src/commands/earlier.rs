//! La reprise des appareils d'avant la collection : la ligne `appliances_content` devient les
//! appareils de la collection, une seule fois — `#[collection(legacy = …)]`.
//!
//! Trois formes ont été stockées : le paquet multilingue `{ by_lang }` dans `content_fr` ; avant
//! lui un paquet `{ devices, safetyNotice }` par langue dans `content_fr` / `content_en` ; et
//! avant encore le premier format (`title`, `subtitle`, `icon`, étapes en chaînes, `tip`, sans
//! id), que l'ancien module convertissait à la lecture — la même conversion est refaite ici.
//!
//! Les langues s'alignent par l'id d'un appareil, les étapes par leur rang dans l'appareil. La
//! consigne de sécurité globale n'est plus montrée : elle n'est pas reprise.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::sdui::common::RichTextDoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Device, Display, Step};

/// L'ancienne ligne, gardée déclarée pour être lue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct AppliancesContent {
    pub id: Uuid,
    pub content_fr: String,
    pub content_en: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Bundle {
    by_lang: BTreeMap<String, Payload>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Payload {
    devices: Vec<Old>,
    #[serde(rename = "paperManualsLocation", alias = "paper_manuals_location")]
    paper_manuals_location: String,
    #[serde(rename = "featuredLimit", alias = "featured_limit")]
    featured_limit: Option<u32>,
    #[serde(rename = "roomOrder")]
    room_order: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct Old {
    id: String,
    name: String,
    emoji: String,
    description: String,
    featured: bool,
    order: i32,
    location: String,
    #[serde(rename = "manualUrl")]
    manual_url: String,
    #[serde(rename = "safetyNote")]
    safety_note: String,
    status: String,
    model: String,
    #[serde(rename = "videoUrl")]
    video_url: String,
    steps: Vec<OldStep>,
    #[serde(rename = "manualFile")]
    manual_file: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct OldStep {
    text: String,
    image: String,
}

/// Les appareils d'avant, s'il y en a.
pub fn earlier_devices() -> portaki_sdk::Result<Option<(Vec<Device>, Display)>> {
    let page = find::<AppliancesContent, AppliancesContent>(Query::new().limit(1))?;
    Ok(page
        .items
        .into_iter()
        .next()
        .and_then(|row| convert(&row.content_fr, &row.content_en)))
}

/// Les appareils et les réglages d'une ancienne ligne ; `None` quand elle ne montrait rien.
pub fn convert(content_fr: &str, content_en: &str) -> Option<(Vec<Device>, Display)> {
    let bundle = bundle_of(content_fr, content_en);
    let mut langs: Vec<(&String, Vec<Old>)> = bundle
        .by_lang
        .iter()
        .map(|(lang, payload)| {
            let mut devices = payload.devices.clone();
            // L'ordre du livret : `order`, celui que l'ancien module triait.
            devices.sort_by_key(|d| d.order);
            (lang, devices)
        })
        .collect();
    // Le français mène, comme pour les règles ; un appareil d'une autre langue suit.
    langs.sort_by_key(|(lang, _)| lang.as_str() != "fr");
    let mut order: Vec<String> = Vec::new();
    for (_, devices) in &langs {
        for device in devices {
            if !order.contains(&device.id) {
                order.push(device.id.clone());
            }
        }
    }
    let devices: Vec<Device> = order.iter().filter_map(|id| device(id, &langs)).collect();
    let display = display(&bundle, &langs);
    let nothing = devices.is_empty()
        && display.paper_manuals_location.is_blank()
        && display.featured_limit == 0
        && display.room_order.is_empty();
    (!nothing).then_some((devices, display))
}

/// Les réglages : la limite de la langue qui mène, le rangement des notices par langue, et
/// l'ordre des pièces de chaque langue l'un après l'autre (chaque langue nomme ses pièces, et
/// le livret ne compare qu'à la pièce de sa langue).
fn display(bundle: &Bundle, langs: &[(&String, Vec<Old>)]) -> Display {
    let mut paper = I18nText::default();
    let mut room_order: Vec<String> = Vec::new();
    let mut featured_limit = None;
    for (lang, _) in langs {
        let payload = &bundle.by_lang[*lang];
        put(&mut paper, lang, &payload.paper_manuals_location);
        featured_limit = featured_limit.or(payload.featured_limit);
        for room in &payload.room_order {
            let room = room.trim();
            if !room.is_empty() && !room_order.iter().any(|r| r == room) {
                room_order.push(room.to_string());
            }
        }
    }
    Display {
        featured_limit: featured_limit.unwrap_or_default(),
        room_order,
        paper_manuals_location: paper,
    }
}

fn bundle_of(content_fr: &str, content_en: &str) -> Bundle {
    let value = serde_json::from_str::<Value>(content_fr.trim()).unwrap_or(Value::Null);
    if value.get("by_lang").is_some() {
        return serde_json::from_value(value).unwrap_or_default();
    }
    let by_lang = [("fr", content_fr), ("en", content_en)]
        .into_iter()
        .map(|(lang, raw)| (lang.to_string(), payload(raw)))
        .filter(|(_, payload)| !payload.devices.is_empty())
        .collect();
    Bundle { by_lang }
}

/// Une langue d'avant le paquet multilingue, au premier format ou non.
fn payload(raw: &str) -> Payload {
    let Ok(value) = serde_json::from_str::<Value>(raw.trim()) else {
        return Payload::default();
    };
    if looks_legacy(&value) {
        return migrate_legacy(&value);
    }
    serde_json::from_value(value).unwrap_or_default()
}

/// L'appareil `id`, ses textes de chaque langue ; ce qui est partagé vient de la première langue.
/// Un appareil sans nom dans aucune langue n'était pas montré : il n'est pas repris.
fn device(id: &str, langs: &[(&String, Vec<Old>)]) -> Option<Device> {
    let found: Vec<(&String, &Old)> = langs
        .iter()
        .filter_map(|(lang, devices)| Some((*lang, devices.iter().find(|d| d.id == id)?)))
        .collect();
    let shared = found.first()?.1;
    let mut device = Device {
        id: id.to_string(),
        emoji: shared.emoji.trim().to_string(),
        model: shared.model.trim().to_string(),
        featured: shared.featured,
        hidden: shared.status.trim().eq_ignore_ascii_case("hidden"),
        steps: steps(&found),
        manual_file: shared.manual_file.trim().to_string(),
        manual_url: shared.manual_url.trim().to_string(),
        video_url: shared.video_url.trim().to_string(),
        ..Device::default()
    };
    for (lang, old) in &found {
        put(&mut device.name, lang, &old.name);
        put(&mut device.location, lang, &old.location);
        put(&mut device.safety_note, lang, &old.safety_note);
        if !portaki_sdk::resource::rich_text::plain_text(&old.description)
            .trim()
            .is_empty()
        {
            put(&mut device.description, lang, &old.description);
        }
    }
    (!device.name.is_blank()).then_some(device)
}

/// Les étapes, alignées par rang ; le schéma vaut pour toutes les langues.
fn steps(found: &[(&String, &Old)]) -> Vec<Step> {
    let count = found.iter().map(|(_, d)| d.steps.len()).max().unwrap_or(0);
    (0..count)
        .map(|rank| {
            let mut step = Step {
                id: format!("s{}", rank + 1),
                ..Step::default()
            };
            for (lang, old) in found {
                let Some(old) = old.steps.get(rank) else {
                    continue;
                };
                put(&mut step.text, lang, &old.text);
                if step.image.is_empty() {
                    step.image = old.image.trim().to_string();
                }
            }
            step
        })
        .collect()
}

/// Range `text` dans la langue `lang`, s'il n'est pas vide.
fn put(target: &mut I18nText, lang: &str, text: &str) {
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }
    match lang {
        "fr" => target.fr = text,
        "en" => target.en = text,
        other => {
            target.others.insert(other.to_string(), text);
        }
    }
}

/// Le premier format : des appareils à `title` sans `name`, ou des étapes en chaînes ; ou une
/// consigne seule en `safety_notice`.
fn looks_legacy(value: &Value) -> bool {
    let snake_notice_only =
        || value.get("safety_notice").is_some() && value.get("safetyNotice").is_none();
    let Some(devices) = value.get("devices").and_then(Value::as_array) else {
        return snake_notice_only();
    };
    if devices.is_empty() {
        return snake_notice_only();
    }
    devices.iter().any(|device| {
        let legacy_steps = device
            .get("steps")
            .and_then(Value::as_array)
            .is_some_and(|steps| steps.iter().any(Value::is_string));
        (device.get("title").is_some() && device.get("name").is_none()) || legacy_steps
    })
}

/// La conversion que l'ancien module faisait à la lecture : les étapes et l'astuce deviennent
/// le mode d'emploi libre (une liste, puis un paragraphe), l'icône un emoji si c'en est un.
fn migrate_legacy(value: &Value) -> Payload {
    let field = |v: &Value, key: &str| v.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let devices = value
        .get("devices")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let devices = devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            let id = field(device, "id");
            let steps: Vec<String> = device
                .get("steps")
                .and_then(Value::as_array)
                .map(|steps| {
                    steps
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            Old {
                id: if id.is_empty() {
                    format!("device-{}", index + 1)
                } else {
                    id
                },
                name: field(device, "title"),
                emoji: emoji_of(&field(device, "icon")),
                description: steps_and_tip(&steps, &field(device, "tip")),
                order: index as i32,
                location: field(device, "subtitle"),
                manual_url: field(device, "manualUrl"),
                ..Old::default()
            }
        })
        .collect();
    Payload {
        devices,
        paper_manuals_location: field(value, "paperManualsLocation"),
        ..Payload::default()
    }
}

/// Un nom d'icône (`wifi`, `tv-2`) ne dit rien au livret : seul un vrai glyphe est gardé.
fn emoji_of(icon: &str) -> String {
    let icon = icon.trim();
    if icon
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return String::new();
    }
    icon.to_string()
}

fn steps_and_tip(steps: &[String], tip: &str) -> String {
    let items: Vec<&str> = steps
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut doc = RichTextDoc::new();
    if !items.is_empty() {
        doc = doc.bullet_list(items);
    }
    if !tip.trim().is_empty() {
        doc = doc.paragraph(tip.trim());
    }
    doc.ensure_non_empty().to_json_string()
}
