//! La reprise du règlement d'avant la collection : la ligne `rules_content` (un paquet JSON par
//! langue) devient les règles de la collection, une seule fois — `#[collection(legacy = …)]`.
//!
//! Trois formes ont été stockées : le paquet multilingue `{ by_lang, card_limit }` dans
//! `content_fr`, et avant lui un paquet `{ items }` par langue dans `content_fr` / `content_en`.
//! Les langues s'alignent par l'identifiant d'une règle ; une règle d'avant les identifiants
//! prend `r{rang}`, comme l'ancien module le faisait à la lecture.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::resource::Choice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Display, Rule, RuleStatus, Theme};

/// L'ancienne ligne, gardée déclarée pour être lue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct RulesContent {
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
    card_limit: Option<u32>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Payload {
    items: Vec<Item>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct Item {
    id: String,
    title: String,
    subtitle: String,
    status: String,
    theme: String,
    hours: String,
}

/// Le règlement d'avant, s'il y en a un.
pub fn earlier_rules() -> portaki_sdk::Result<Option<(Vec<Rule>, Display)>> {
    let page = find::<RulesContent, RulesContent>(Query::new().limit(1))?;
    Ok(page
        .items
        .into_iter()
        .next()
        .and_then(|row| convert(&row.content_fr, &row.content_en)))
}

/// Les règles et la limite de la carte d'une ancienne ligne ; `None` sans règle nommée.
pub fn convert(content_fr: &str, content_en: &str) -> Option<(Vec<Rule>, Display)> {
    let bundle = bundle_of(content_fr, content_en);
    let mut langs: Vec<(&String, Vec<Item>)> = bundle
        .by_lang
        .iter()
        .map(|(lang, payload)| (lang, with_ids(&payload.items)))
        .collect();
    // Le français mène l'ordre, comme dans le livret ; une règle d'une autre langue suit.
    langs.sort_by_key(|(lang, _)| lang.as_str() != "fr");
    let mut order: Vec<String> = Vec::new();
    for (_, items) in &langs {
        for item in items {
            if !order.contains(&item.id) {
                order.push(item.id.clone());
            }
        }
    }
    let rules: Vec<Rule> = order.iter().filter_map(|id| rule(id, &langs)).collect();
    if rules.is_empty() {
        return None;
    }
    let display = Display {
        card_limit: bundle.card_limit.unwrap_or_default(),
    };
    Some((rules, display))
}

fn bundle_of(content_fr: &str, content_en: &str) -> Bundle {
    let value = serde_json::from_str::<Value>(content_fr.trim()).unwrap_or(Value::Null);
    if value.get("by_lang").is_some() {
        return serde_json::from_value(value).unwrap_or_default();
    }
    let payload = |raw: &str| serde_json::from_str::<Payload>(raw.trim()).unwrap_or_default();
    let mut by_lang = BTreeMap::new();
    for (lang, raw) in [("fr", content_fr), ("en", content_en)] {
        let payload = payload(raw);
        if !payload.items.is_empty() {
            by_lang.insert(lang.to_string(), payload);
        }
    }
    Bundle {
        by_lang,
        card_limit: None,
    }
}

fn with_ids(items: &[Item]) -> Vec<Item> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let mut item = item.clone();
            if item.id.trim().is_empty() {
                item.id = format!("r{index}");
            }
            item
        })
        .collect()
}

/// La règle `id`, ses textes de chaque langue ; ce qui est partagé vient de la première langue.
fn rule(id: &str, langs: &[(&String, Vec<Item>)]) -> Option<Rule> {
    let mut title = I18nText::default();
    let mut detail = I18nText::default();
    let mut shared: Option<&Item> = None;
    for (lang, items) in langs {
        let Some(item) = items.iter().find(|item| item.id == id) else {
            continue;
        };
        shared.get_or_insert(item);
        if !item.title.trim().is_empty() {
            title = title.with((*lang).clone(), item.title.trim().to_string());
        }
        if !item.subtitle.trim().is_empty() {
            detail = detail.with((*lang).clone(), item.subtitle.trim().to_string());
        }
    }
    let shared = shared?;
    if title.is_blank() {
        return None;
    }
    let status = match shared.status.trim() {
        "ok" => RuleStatus::Allowed,
        wire => RuleStatus::from_wire(wire).unwrap_or_default(),
    };
    Some(Rule {
        id: id.to_string(),
        title,
        detail,
        status,
        // Un ancien thème libre n'a pas de place dans la liste : « Autre ».
        theme: Theme::from_wire(&shared.theme).unwrap_or_default(),
        hours: shared
            .hours
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .replace(['–', '—'], "-"),
    })
}
