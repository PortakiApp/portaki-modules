//! Appliances payload — single-language device list (TipTap description).

use portaki_sdk::sdui::common::RichTextDoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Le nombre d'appareils qu'un hôte peut décrire (§2.4 : `devices`, 0 → 60).
///
/// Dix était un chiffre en dur qui refusait le onzième : une villa avec sa cuisine, ses trois
/// salles de bain et sa buanderie y arrivait sans effort, et le cas « 30 appareils » de la
/// maquette était tout simplement inatteignable. La borne reste — soixante tient dans une liste
/// groupée par pièce — mais c'est celle du contrat.
pub const MAX_APPLIANCES: usize = 60;
/// Les appareils mis en avant sur la carte d'accueil : quatre par défaut, de deux à six au choix
/// de l'hôte (spec Appareils §2.1). La carte d'accueil n'est pas la liste : elle donne un aperçu
/// et renvoie au reste, « Voir les N appareils » juste en dessous.
pub const DEFAULT_FEATURED: usize = 4;
pub const MIN_FEATURED: usize = 2;
pub const MAX_FEATURED: usize = 6;

/// Guest-visible vs host-only hidden.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApplianceStatus {
    #[default]
    Active,
    Hidden,
}

/// One appliance / device guide entry (v2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Appliance {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub emoji: String,
    /// TipTap JSON document string.
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub location: String,
    #[serde(default, rename = "manualUrl")]
    pub manual_url: String,
    #[serde(default, rename = "safetyNote")]
    pub safety_note: String,
    #[serde(default)]
    pub status: ApplianceStatus,
}

/// Root payload stored as JSON in `content_fr` (canonical; single language).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppliancesPayload {
    #[serde(default)]
    pub devices: Vec<Appliance>,
    /// Global TipTap JSON safety notice (shown above the guest device list).
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: String,
    /// Où les notices papier sont rangées, pour tous les appareils (§3.1).
    ///
    /// Au niveau du logement et non de l'appareil : les notices tiennent dans la même boîte, et le
    /// demander une fois par appareil ferait répéter « étagère du salon » quinze fois.
    #[serde(
        default,
        rename = "paperManualsLocation",
        alias = "paper_manuals_location"
    )]
    pub paper_manuals_location: String,
    /// « Appareils mis en avant » (§2.1). Absent : [`DEFAULT_FEATURED`].
    #[serde(
        default,
        rename = "featuredLimit",
        alias = "featured_limit",
        skip_serializing_if = "Option::is_none"
    )]
    pub featured_limit: Option<u32>,
}

impl AppliancesPayload {
    /// Combien d'appareils la carte d'accueil met en avant, borné.
    pub fn featured_limit(&self) -> usize {
        self.featured_limit.map_or(DEFAULT_FEATURED, |n| {
            (n as usize).clamp(MIN_FEATURED, MAX_FEATURED)
        })
    }

    /// Ce qui ne va pas, champ par champ — sous le champ ([`Self::error_of`]) et dans
    /// `publishReadiness`. Les bornes de la spec Appareils (§2.1, §2.2), avec ses messages.
    pub fn problems(&self) -> Vec<(String, portaki_sdk::contracts::i18n::I18nText)> {
        use portaki_sdk::config::check;
        let mut problems = Vec::new();
        if let Some(error) = check::max_chars(&self.paper_manuals_location, 120) {
            problems.push(("paperManualsLocation".to_string(), error));
        }
        // La valeur saisie, pas celle bornée à la lecture : un 9 enregistré se signale, il ne
        // devient pas 6 en silence.
        if let Some(error) = self
            .featured_limit
            .and_then(|n| check::between(n as f64, MIN_FEATURED as f64, MAX_FEATURED as f64))
        {
            problems.push(("featuredLimit".to_string(), error));
        }
        let featured = self
            .devices
            .iter()
            .filter(|d| d.featured && d.status == ApplianceStatus::Active)
            .count();
        if featured > self.featured_limit() {
            let limit = self.featured_limit().to_string();
            problems.push((
                "featuredLimit".to_string(),
                crate::i18n::text_with("host.featured.tooMany", &[("count", &limit)]),
            ));
        }
        if self.devices.len() > MAX_APPLIANCES {
            problems.push((
                "devices".to_string(),
                crate::i18n::text("host.devices.tooMany"),
            ));
        }
        for (index, device) in self.devices.iter().enumerate() {
            let name_required = device
                .name
                .trim()
                .is_empty()
                .then(|| crate::i18n::text("host.device.name.required"));
            for (key, error) in [
                (
                    "name",
                    name_required.or_else(|| check::max_chars(&device.name, 60)),
                ),
                ("location", check::max_chars(&device.location, 30)),
                ("safetyNote", check::max_chars(&device.safety_note, 280)),
                (
                    "description",
                    check::max_chars(&description_plain_text(&device.description), 3000),
                ),
                ("manualUrl", check::https_url(device.manual_url.trim())),
            ] {
                if let Some(error) = error {
                    problems.push((format!("devices.{index}.{key}"), error));
                }
            }
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<portaki_sdk::contracts::i18n::I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }

    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            return Self::default();
        };
        if looks_legacy(&value) {
            return migrate_legacy(&value);
        }
        serde_json::from_value(value).unwrap_or_default()
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn is_empty(&self) -> bool {
        !self.devices.iter().any(|d| !d.name.trim().is_empty())
    }

    pub fn is_empty_for_guest(&self) -> bool {
        self.guest_devices().is_empty()
    }

    pub fn find_device(&self, device_id: &str) -> Option<&Appliance> {
        self.devices.iter().find(|d| d.id == device_id)
    }

    /// Active, named devices for guest surfaces (hidden excluded). Sorted by `order`.
    pub fn guest_devices(&self) -> Vec<&Appliance> {
        let mut devices: Vec<&Appliance> = self
            .devices
            .iter()
            .filter(|d| d.status == ApplianceStatus::Active && !d.name.trim().is_empty())
            .collect();
        devices.sort_by_key(|d| d.order);
        devices
    }

    /// Featured + active for home card (at most [`Self::featured_limit`]).
    ///
    /// Aucun mis en avant : les `featured_limit` premiers actifs (spec §3, cas §9 #3). Sans ce
    /// repli, un hôte qui n'a rien coché avait une carte d'accueil vide.
    pub fn featured_guest_devices(&self) -> Vec<&Appliance> {
        let limit = self.featured_limit();
        let devices = self.guest_devices();
        let featured: Vec<&Appliance> = devices
            .iter()
            .copied()
            .filter(|d| d.featured)
            .take(limit)
            .collect();
        if featured.is_empty() {
            devices.into_iter().take(limit).collect()
        } else {
            featured
        }
    }

    /// Les appareils groupés par pièce, pour la liste complète (§2.4).
    ///
    /// Les pièces arrivent dans l'ordre où l'hôte a rangé ses appareils, et non par ordre
    /// alphabétique : un hôte qui ordonne salon, cuisine, chambre décrit un logement, et trier
    /// remplacerait sa logique par celle de l'alphabet.
    ///
    /// Un appareil sans pièce tombe dans un groupe sans nom, que la surface intitule « Autres » et
    /// place en dernier — un appareil que l'hôte n'a pas rangé reste visible, il n'ouvre pas la
    /// liste.
    pub fn guest_devices_by_room(&self) -> Vec<(Option<String>, Vec<&Appliance>)> {
        let mut rooms: Vec<(Option<String>, Vec<&Appliance>)> = Vec::new();
        let mut unplaced: Vec<&Appliance> = Vec::new();

        for device in self.guest_devices() {
            let room = device.location.trim();
            if room.is_empty() {
                unplaced.push(device);
                continue;
            }
            match rooms
                .iter_mut()
                .find(|(name, _)| name.as_deref().is_some_and(|existing| existing == room))
            {
                Some((_, devices)) => devices.push(device),
                None => rooms.push((Some(room.to_string()), vec![device])),
            }
        }

        if !unplaced.is_empty() {
            rooms.push((None, unplaced));
        }
        rooms
    }

    pub fn featured_count(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| d.featured && d.status != ApplianceStatus::Hidden)
            .count()
    }

    pub fn sort_by_order(&mut self) {
        self.devices.sort_by_key(|d| d.order);
    }
}

/// N-language storage written into `content_fr` (`content_en` cleared after migrate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppliancesBundle {
    #[serde(default)]
    pub by_lang: std::collections::BTreeMap<String, AppliancesPayload>,
}

impl AppliancesBundle {
    pub fn lang_code(locale: &str) -> String {
        let trimmed = locale.trim();
        if trimmed.is_empty() {
            return "fr".to_string();
        }
        let lower = trimmed.to_ascii_lowercase();
        let base = lower.split(['-', '_']).next().unwrap_or("fr").trim();
        if base.is_empty() {
            "fr".to_string()
        } else {
            base.to_string()
        }
    }

    pub fn from_row(content_fr: &str, content_en: &str) -> Self {
        if let Ok(value) = serde_json::from_str::<Value>(content_fr.trim()) {
            if value.get("by_lang").is_some() {
                if let Ok(bundle) = serde_json::from_value::<AppliancesBundle>(value.clone()) {
                    return bundle;
                }
            }
            if value.get("devices").is_some() || value.get("safetyNotice").is_some() {
                let mut bundle = Self::default();
                let fr = AppliancesPayload::parse(content_fr);
                if !fr.is_empty() || !fr.safety_notice.trim().is_empty() {
                    bundle.by_lang.insert("fr".into(), fr);
                }
                let en = AppliancesPayload::parse(content_en);
                if !en.is_empty() || !en.safety_notice.trim().is_empty() {
                    bundle.by_lang.insert("en".into(), en);
                }
                return bundle;
            }
        }
        let mut bundle = Self::default();
        let fr = AppliancesPayload::parse(content_fr);
        if !fr.is_empty() || !fr.safety_notice.trim().is_empty() {
            bundle.by_lang.insert("fr".into(), fr);
        }
        let en = AppliancesPayload::parse(content_en);
        if !en.is_empty() || !en.safety_notice.trim().is_empty() {
            bundle.by_lang.insert("en".into(), en);
        }
        bundle
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get(&self, lang: &str) -> AppliancesPayload {
        self.by_lang
            .get(&Self::lang_code(lang))
            .cloned()
            .unwrap_or_default()
    }

    pub fn set(&mut self, lang: &str, payload: AppliancesPayload) {
        let code = Self::lang_code(lang);
        self.by_lang.insert(code, payload);
    }

    /// Copy shared device fields from `source` into other language payloads (by device id).
    pub fn sync_shared_from(&mut self, source: &AppliancesPayload) {
        for payload in self.by_lang.values_mut() {
            for device in &mut payload.devices {
                if let Some(src) = source.find_device(&device.id) {
                    device.emoji = src.emoji.clone();
                    device.featured = src.featured;
                    device.order = src.order;
                    device.manual_url = src.manual_url.clone();
                    device.status = src.status;
                }
            }
            // Align device list order/ids with source when missing.
            for src in &source.devices {
                if !payload.devices.iter().any(|d| d.id == src.id) {
                    let mut clone = src.clone();
                    clone.name.clear();
                    clone.description.clear();
                    clone.location.clear();
                    clone.safety_note.clear();
                    payload.devices.push(clone);
                }
            }
            payload
                .devices
                .retain(|d| source.find_device(&d.id).is_some());
            payload.sort_by_order();
        }
    }

    pub fn pick(&self, guest_locale: &str, property_locale: &str) -> AppliancesPayload {
        let candidates = [
            Self::lang_code(guest_locale),
            Self::lang_code(property_locale),
            "fr".to_string(),
        ];
        let mut tried = std::collections::BTreeSet::new();
        for lang in &candidates {
            if !tried.insert(lang.clone()) {
                continue;
            }
            let payload = self.get(lang);
            if !payload.is_empty() || !payload.safety_notice.trim().is_empty() {
                return payload;
            }
        }
        for payload in self.by_lang.values() {
            if !payload.is_empty() || !payload.safety_notice.trim().is_empty() {
                return payload.clone();
            }
        }
        AppliancesPayload::default()
    }
}

fn looks_legacy(value: &Value) -> bool {
    let Some(devices) = value.get("devices").and_then(|d| d.as_array()) else {
        // Legacy empty payload that only carried a snake_case safety_notice.
        return value.get("safety_notice").is_some() && value.get("safetyNotice").is_none();
    };
    if devices.is_empty() {
        return value.get("safety_notice").is_some() && value.get("safetyNotice").is_none();
    }
    devices.iter().any(|device| {
        let has_title = device.get("title").is_some();
        let has_name = device.get("name").is_some();
        let has_steps = device.get("steps").is_some();
        (has_title && !has_name) || has_steps
    })
}

fn migrate_legacy(value: &Value) -> AppliancesPayload {
    let empty = Vec::new();
    let devices = value
        .get("devices")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty);

    let mut migrated = Vec::new();
    for (index, device) in devices.iter().enumerate() {
        let title = string_field(device, "title");
        let id = {
            let raw = string_field(device, "id");
            if raw.is_empty() {
                format!("device-{}", index + 1)
            } else {
                raw
            }
        };
        if title.trim().is_empty() && id.trim().is_empty() {
            continue;
        }
        let steps = device
            .get("steps")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let tip = string_field(device, "tip");
        let icon = string_field(device, "icon");

        migrated.push(Appliance {
            id,
            name: title,
            emoji: emoji_from_legacy_icon(&icon),
            description: steps_and_tip_to_tiptap(&steps, &tip),
            featured: false,
            order: index as i32,
            location: string_field(device, "subtitle"),
            manual_url: string_field(device, "manualUrl"),
            safety_note: String::new(),
            status: ApplianceStatus::Active,
        });
    }

    let legacy_safety = string_field(value, "safety_notice");
    AppliancesPayload {
        paper_manuals_location: string_field(value, "paperManualsLocation"),
        devices: migrated,
        safety_notice: plain_text_to_tiptap(&legacy_safety),
        featured_limit: None,
    }
}

fn plain_text_to_tiptap(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    RichTextDoc::new().paragraph(trimmed).to_json_string()
}

/// Lucide-style names stay empty; emoji / other glyphs are kept.
fn emoji_from_legacy_icon(icon: &str) -> String {
    let trimmed = icon.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return String::new();
    }
    trimmed.to_string()
}

fn steps_and_tip_to_tiptap(steps: &[String], tip: &str) -> String {
    let items: Vec<&str> = steps
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut doc = RichTextDoc::new();
    if !items.is_empty() {
        doc = doc.bullet_list(items);
    }
    let tip = tip.trim();
    if !tip.is_empty() {
        doc = doc.paragraph(tip);
    }
    doc.ensure_non_empty().to_json_string()
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Rough plain-text extract from TipTap JSON (host preview / fallbacks).
pub fn description_plain_text(description: &str) -> String {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return trimmed.to_string();
    };
    if value.get("type").and_then(|t| t.as_str()) != Some("doc") {
        return trimmed.to_string();
    }
    let mut out = Vec::new();
    collect_text(&value, &mut out);
    out.join("\n")
}

/// Une étape du mode d'emploi : sa consigne, et le schéma que l'hôte a glissé dedans.
///
/// Le schéma est facultatif, et c'est l'extension « étapes illustrées » du §2.4 : une poignée de
/// fenêtre oscillo-battante se montre, elle ne se décrit pas. Une étape sans image rend
/// exactement ce qu'elle rendait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowToStep {
    pub text: String,
    /// La référence `portaki-file:` ou l'URL de l'image, telle que l'éditeur l'a posée.
    pub image: Option<String>,
}

/// Extract ordered how-to steps from TipTap bullet/ordered lists (guest detail SDUI).
pub fn extract_howto_steps(description: &str) -> Vec<HowToStep> {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };
    if value.get("type").and_then(|t| t.as_str()) != Some("doc") {
        return Vec::new();
    }
    let mut steps = Vec::new();
    let Some(children) = value.get("content").and_then(|c| c.as_array()) else {
        return steps;
    };
    for child in children {
        let node_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if node_type != "bulletList" && node_type != "orderedList" {
            continue;
        }
        let Some(items) = child.get("content").and_then(|c| c.as_array()) else {
            continue;
        };
        for item in items {
            let mut parts = Vec::new();
            collect_text(item, &mut parts);
            let text = parts.join(" ").trim().to_string();
            if !text.is_empty() {
                steps.push(HowToStep {
                    text,
                    image: first_image(item),
                });
            }
        }
    }
    steps
}

/// Le `src` du premier nœud image de ce sous-arbre.
///
/// Le premier seulement : une étape montre un schéma, pas une galerie, et les suivants
/// descendraient la consigne suivante hors de l'écran.
fn first_image(node: &Value) -> Option<String> {
    if node.get("type").and_then(|t| t.as_str()) == Some("image") {
        let src = node
            .get("attrs")
            .and_then(|attrs| attrs.get("src"))
            .and_then(|src| src.as_str())
            .map(str::trim)
            .filter(|src| !src.is_empty());
        if let Some(src) = src {
            return Some(src.to_string());
        }
    }
    node.get("content")
        .and_then(|content| content.as_array())?
        .iter()
        .find_map(first_image)
}

fn collect_text(node: &Value, out: &mut Vec<String>) {
    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        if !text.is_empty() {
            out.push(text.to_string());
        }
    }
    if let Some(children) = node.get("content").and_then(|c| c.as_array()) {
        for child in children {
            collect_text(child, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_howto_steps, HowToStep};

    /// Une étape porte son schéma, et une étape sans image rend exactement ce qu'elle rendait.
    #[test]
    fn a_step_carries_the_diagram_the_host_dropped_in_it() {
        let doc = r#"{"type":"doc","content":[{"type":"orderedList","content":[
            {"type":"listItem","content":[
                {"type":"paragraph","content":[{"type":"text","text":"Poignée vers le bas"}]},
                {"type":"image","attrs":{"src":"portaki-file:abc"}}
            ]},
            {"type":"listItem","content":[
                {"type":"paragraph","content":[{"type":"text","text":"Poignée à l'horizontale"}]}
            ]}
        ]}]}"#;
        assert_eq!(
            extract_howto_steps(doc),
            vec![
                HowToStep {
                    text: "Poignée vers le bas".into(),
                    image: Some("portaki-file:abc".into()),
                },
                HowToStep {
                    text: "Poignée à l'horizontale".into(),
                    image: None,
                },
            ]
        );
    }

    /// Un `src` vide n'est pas une image : il ferait dessiner un cadre gris sous la consigne.
    #[test]
    fn an_empty_source_is_not_a_diagram() {
        let doc = r#"{"type":"doc","content":[{"type":"orderedList","content":[
            {"type":"listItem","content":[
                {"type":"paragraph","content":[{"type":"text","text":"Ouvrir"}]},
                {"type":"image","attrs":{"src":"  "}}
            ]}
        ]}]}"#;
        assert_eq!(extract_howto_steps(doc)[0].image, None);
    }

    use super::*;

    #[test]
    fn migrates_legacy_fr_slots() {
        let raw = r#"{
            "safety_notice": "Coupez l'eau.",
            "devices": [
                {
                    "id": "tv",
                    "icon": "📺",
                    "title": "Télévision",
                    "subtitle": "Salon",
                    "steps": ["Allumez", "HDMI 1"],
                    "tip": "Remote on stand"
                },
                {
                    "id": "washer",
                    "icon": "washing-machine",
                    "title": "Lave-linge",
                    "steps": ["ECO 30"]
                }
            ]
        }"#;
        let payload = AppliancesPayload::parse(raw);
        assert_eq!(payload.devices.len(), 2);
        assert_eq!(payload.devices[0].name, "Télévision");
        assert_eq!(payload.devices[0].emoji, "📺");
        assert!(!payload.devices[0].featured);
        assert_eq!(payload.devices[0].location, "Salon");
        assert!(payload.devices[0].description.contains("bulletList"));
        assert!(payload.devices[0].description.contains("Allumez"));
        assert!(payload.devices[0].description.contains("Remote on stand"));
        assert_eq!(payload.devices[1].emoji, "");
        assert_eq!(payload.devices[1].order, 1);
        assert!(payload.safety_notice.contains("Coupez l'eau."));
        assert!(payload.safety_notice.contains("paragraph"));
    }

    #[test]
    fn parses_v2_without_migration() {
        let raw = r#"{
            "safetyNotice": "{\"type\":\"doc\",\"content\":[{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"text\":\"Global\"}]}]}",
            "devices": [{
                "id": "a1",
                "name": "Oven",
                "emoji": "🔥",
                "description": "{\"type\":\"doc\",\"content\":[{\"type\":\"paragraph\"}]}",
                "featured": true,
                "order": 0,
                "location": "Kitchen",
                "manualUrl": "https://example.com",
                "safetyNote": "Hot",
                "status": "active"
            }]
        }"#;
        let payload = AppliancesPayload::parse(raw);
        assert_eq!(payload.devices.len(), 1);
        assert_eq!(payload.devices[0].name, "Oven");
        assert!(payload.devices[0].featured);
        assert_eq!(payload.devices[0].manual_url, "https://example.com");
        assert!(payload.safety_notice.contains("Global"));
    }

    #[test]
    fn guest_devices_exclude_hidden() {
        let payload = AppliancesPayload {
            devices: vec![
                Appliance {
                    id: "1".into(),
                    name: "Visible".into(),
                    status: ApplianceStatus::Active,
                    featured: true,
                    order: 1,
                    ..Default::default()
                },
                Appliance {
                    id: "2".into(),
                    name: "Hidden".into(),
                    status: ApplianceStatus::Hidden,
                    featured: true,
                    order: 0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let guest = payload.guest_devices();
        assert_eq!(guest.len(), 1);
        assert_eq!(guest[0].id, "1");
        assert_eq!(payload.featured_guest_devices().len(), 1);
    }

    /// Mis en avant : quatre par défaut, de deux à six ; au-delà, la publication le dit.
    #[test]
    fn the_featured_limit_is_bounded_and_checked() {
        let device = |featured| Appliance {
            name: "Four".into(),
            featured,
            ..Appliance::default()
        };
        let mut payload = AppliancesPayload {
            devices: vec![device(true), device(true), device(true)],
            ..AppliancesPayload::default()
        };
        assert_eq!(payload.featured_limit(), DEFAULT_FEATURED);
        payload.featured_limit = Some(9);
        assert_eq!(payload.featured_limit(), MAX_FEATURED);
        payload.featured_limit = Some(2);
        assert_eq!(payload.featured_guest_devices().len(), 2);
        let fields: Vec<String> = payload.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(fields, ["featuredLimit"]);
    }

    fn named(name: &str) -> Appliance {
        Appliance {
            name: name.into(),
            ..Appliance::default()
        }
    }

    /// Le message de `field`, en français.
    fn error_fr(payload: &AppliancesPayload, field: &str) -> Option<String> {
        payload.error_of(field).map(|e| e.get("fr").to_string())
    }

    /// Aucun mis en avant : la carte montre les `featured_limit` premiers actifs (§3, §9 #3).
    #[test]
    fn without_featured_the_card_shows_the_first_active_ones() {
        let mut devices: Vec<Appliance> = (0..6)
            .map(|i| Appliance {
                id: i.to_string(),
                order: 5 - i,
                ..named("Four")
            })
            .collect();
        devices[0].status = ApplianceStatus::Hidden;
        let mut payload = AppliancesPayload {
            devices,
            ..AppliancesPayload::default()
        };
        let ids = |p: &AppliancesPayload| -> Vec<String> {
            p.featured_guest_devices()
                .iter()
                .map(|d| d.id.clone())
                .collect()
        };
        assert_eq!(ids(&payload), ["5", "4", "3", "2"]);
        payload.featured_limit = Some(2);
        assert_eq!(ids(&payload), ["5", "4"]);
        // Un seul coché : lui seul, pas de repli.
        payload.devices[3].featured = true;
        assert_eq!(ids(&payload), ["3"]);
    }

    /// Hors de 2 à 6 : « Entre 2 et 6. » sous le champ, et la carte borne toujours à la lecture.
    #[test]
    fn a_featured_limit_out_of_range_is_reported_not_clamped() {
        let mut payload = AppliancesPayload {
            featured_limit: Some(9),
            ..AppliancesPayload::default()
        };
        assert_eq!(
            error_fr(&payload, "featuredLimit").as_deref(),
            Some("Entre 2 et 6.")
        );
        assert_eq!(payload.featured_limit(), MAX_FEATURED);
        payload.featured_limit = Some(0);
        assert_eq!(
            error_fr(&payload, "featuredLimit").as_deref(),
            Some("Entre 2 et 6.")
        );
        payload.featured_limit = Some(6);
        assert_eq!(error_fr(&payload, "featuredLimit"), None);
    }

    #[test]
    fn too_many_featured_says_the_spec_message() {
        let payload = AppliancesPayload {
            devices: vec![
                Appliance {
                    featured: true,
                    ..named("A")
                },
                Appliance {
                    featured: true,
                    ..named("B")
                },
                Appliance {
                    featured: true,
                    ..named("C")
                },
            ],
            featured_limit: Some(2),
            ..AppliancesPayload::default()
        };
        assert_eq!(
            error_fr(&payload, "featuredLimit").as_deref(),
            Some("Vous avez déjà mis 2 appareils en avant.")
        );
    }

    #[test]
    fn more_than_sixty_appliances_is_reported() {
        let mut payload = AppliancesPayload {
            devices: vec![named("Four"); MAX_APPLIANCES],
            ..AppliancesPayload::default()
        };
        assert_eq!(error_fr(&payload, "devices"), None);
        payload.devices.push(named("Four"));
        assert_eq!(
            error_fr(&payload, "devices").as_deref(),
            Some("60 appareils au maximum.")
        );
    }

    #[test]
    fn an_appliance_without_a_name_is_reported_under_its_name() {
        let payload = AppliancesPayload {
            devices: vec![named("Four"), named("  ")],
            ..AppliancesPayload::default()
        };
        assert_eq!(error_fr(&payload, "devices.0.name"), None);
        assert_eq!(
            error_fr(&payload, "devices.1.name").as_deref(),
            Some("Donnez un nom à l'appareil.")
        );
    }

    #[test]
    fn the_room_is_thirty_characters_at_most() {
        let mut payload = AppliancesPayload {
            devices: vec![Appliance {
                location: "x".repeat(30),
                ..named("Four")
            }],
            ..AppliancesPayload::default()
        };
        assert_eq!(error_fr(&payload, "devices.0.location"), None);
        payload.devices[0].location.push('x');
        assert_eq!(
            error_fr(&payload, "devices.0.location").as_deref(),
            Some("30 caractères au maximum.")
        );
    }

    #[test]
    fn the_safety_note_is_280_characters_at_most() {
        let payload = AppliancesPayload {
            devices: vec![Appliance {
                safety_note: "x".repeat(281),
                ..named("Four")
            }],
            ..AppliancesPayload::default()
        };
        assert_eq!(
            error_fr(&payload, "devices.0.safetyNote").as_deref(),
            Some("280 caractères au maximum.")
        );
    }

    /// Le texte du mode d'emploi, pas son JSON TipTap : le balisage ne compte pas.
    #[test]
    fn the_manual_text_is_3000_characters_of_text_at_most() {
        let doc = |text: String| RichTextDoc::new().paragraph(&text).to_json_string();
        let mut payload = AppliancesPayload {
            devices: vec![Appliance {
                description: doc("x".repeat(3000)),
                ..named("Four")
            }],
            ..AppliancesPayload::default()
        };
        assert!(payload.devices[0].description.chars().count() > 3000);
        assert_eq!(error_fr(&payload, "devices.0.description"), None);
        payload.devices[0].description = doc("x".repeat(3001));
        assert_eq!(
            error_fr(&payload, "devices.0.description").as_deref(),
            Some("3000 caractères au maximum.")
        );
    }
}
