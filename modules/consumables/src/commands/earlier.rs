//! La reprise du catalogue d'avant la collection : les lignes `consumable_item` deviennent les
//! produits, et les réglages en KV (`restock_delay`, `settings`) le meta, une seule fois —
//! `#[collection(legacy = …)]`.
//!
//! Un produit garde l'id de sa ligne : `ConsumableReport.item_id` le vise. Le nom et « où le
//! trouver » sont une carte JSON par langue, ou, avant elle, un texte français en clair ; l'ancien
//! `label_en` comble l'anglais. Les demandes restent dans leur table.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::kv;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::resource::Choice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Channel, Product, Requests};

/// L'ancienne ligne du catalogue, gardée déclarée pour être lue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 5)]
pub struct ConsumableItem {
    pub id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
    #[serde(default)]
    pub low_threshold: i32,
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub location: String,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ConsumableItem)]
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const CONSUMABLE_ITEM_INDEXES: &[&str] = &["sort_order"];

/// Les anciens réglages des demandes, la valeur KV `settings`.
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub requests_enabled: Option<bool>,
    pub max_requests: Option<u32>,
    pub notify: Option<Vec<String>>,
}

/// Le catalogue et les réglages d'avant, s'il y en a.
pub fn earlier_products() -> portaki_sdk::Result<Option<(Vec<Product>, Requests)>> {
    // ponytail: une page de 200, l'ancien module n'en lisait pas plus.
    let items = find::<ConsumableItem, ConsumableItem>(Query::new().limit(200))?.items;
    let restock_delay = kv::get("restock_delay")?.and_then(|b| serde_json::from_slice(&b).ok());
    let settings = kv::get("settings")?.and_then(|b| serde_json::from_slice(&b).ok());
    Ok(convert(items, restock_delay, settings))
}

/// Les produits dans l'ordre de l'ancien catalogue, et les réglages ; `None` sans rien d'enregistré.
pub fn convert(
    mut items: Vec<ConsumableItem>,
    restock_delay: Option<I18nText>,
    settings: Option<Settings>,
) -> Option<(Vec<Product>, Requests)> {
    if items.is_empty() && restock_delay.is_none() && settings.is_none() {
        return None;
    }
    items.sort_by_key(|a| (a.sort_order, a.created_at));
    let products = items
        .iter()
        .map(product)
        // Un produit sans nom n'était pas montré au voyageur, ni demandable.
        .filter(|p| !p.label.is_blank())
        .collect();
    let settings = settings.unwrap_or_default();
    let requests = Requests {
        requests_enabled: settings.requests_enabled,
        restock_delay: restock_delay.unwrap_or_default(),
        notify: settings.notify.map(|channels| {
            channels
                .iter()
                .filter_map(|c| Channel::from_wire(c))
                .collect()
        }),
        max_requests: settings.max_requests,
    };
    Some((products, requests))
}

fn product(item: &ConsumableItem) -> Product {
    Product {
        id: item.id.to_string(),
        label: decode(&item.label_fr, &item.label_en),
        emoji: item.emoji.trim().to_string(),
        location: decode(&item.location, ""),
    }
}

/// Une carte JSON `{"fr":…,"en":…}`, ou un texte français d'avant la carte, plus le repli anglais
/// — la lecture de l'ancien module.
fn decode(raw: &str, legacy_en: &str) -> I18nText {
    let raw = raw.trim();
    let mut out = BTreeMap::new();
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(map)) => {
            for (lang, text) in map {
                if let Some(text) = text.as_str().map(str::trim).filter(|t| !t.is_empty()) {
                    out.insert(lang_code(&lang), text.to_string());
                }
            }
        }
        _ if !raw.is_empty() => {
            out.insert("fr".to_string(), raw.to_string());
        }
        _ => {}
    }
    if !legacy_en.trim().is_empty() {
        out.entry("en".to_string())
            .or_insert_with(|| legacy_en.trim().to_string());
    }
    serde_json::from_value(serde_json::to_value(out).unwrap_or_default()).unwrap_or_default()
}

fn lang_code(locale: &str) -> String {
    let lower = locale.trim().to_ascii_lowercase();
    match lower.split(['-', '_']).next().map(str::trim) {
        Some(base) if !base.is_empty() => base.to_string(),
        _ => "fr".to_string(),
    }
}
