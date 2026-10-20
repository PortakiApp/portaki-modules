//! La reprise des sections d'avant la collection : une ligne `SectionItem` par section (son
//! rang) et une ligne `SectionItemLocale` par (section, langue) deviennent les sections de la
//! collection, une seule fois — `#[collection(legacy = …)]`.
//!
//! L'id de la section reste l'id de la ligne ; l'ordre est celui du livret (`sort_order`, puis la
//! date de création) ; une section sans aucun texte, que l'ancien livret cachait, n'est pas reprise.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::resource::NoMeta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Section;

/// L'ancienne section (son rang), gardée déclarée pour être lue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct SectionItem {
    pub id: Uuid,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(SectionItem)]
#[allow(dead_code, reason = "read by the migration generator, kept as on main")]
pub const SECTION_ITEM_INDEXES: &[&str] = &["sort_order"];

/// Le titre et le texte d'une ancienne section dans une langue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct SectionItemLocale {
    pub id: Uuid,
    pub section_id: Uuid,
    pub lang: String,
    pub title: String,
    pub body_markdown: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(SectionItemLocale)]
#[allow(dead_code, reason = "read by the migration generator, kept as on main")]
pub const SECTION_ITEM_LOCALE_INDEXES: &[&str] = &["section_id", "lang"];

/// Les sections d'avant, s'il y en a. Les plafonds de lecture sont ceux de l'ancien module.
pub fn earlier_sections() -> portaki_sdk::Result<Option<(Vec<Section>, NoMeta)>> {
    let items = find::<SectionItem, SectionItem>(Query::new().limit(200))?.items;
    let locales = find::<SectionItemLocale, SectionItemLocale>(Query::new().limit(500))?.items;
    let sections = convert(items, &locales);
    Ok((!sections.is_empty()).then_some((sections, NoMeta {})))
}

/// Les sections des anciennes lignes, dans l'ordre du livret, sans les sections vides.
pub fn convert(mut items: Vec<SectionItem>, locales: &[SectionItemLocale]) -> Vec<Section> {
    items.sort_by_key(|item| (item.sort_order, item.created_at));
    items
        .iter()
        .filter_map(|item| section(item.id, locales))
        .collect()
}

fn section(id: Uuid, locales: &[SectionItemLocale]) -> Option<Section> {
    let mut title = I18nText::default();
    let mut body = I18nText::default();
    for locale in locales.iter().filter(|locale| locale.section_id == id) {
        let lang = locale.lang.trim().to_string();
        if !locale.title.trim().is_empty() {
            title = title.with(lang.clone(), locale.title.trim().to_string());
        }
        if !locale.body_markdown.trim().is_empty() {
            body = body.with(lang, locale.body_markdown.trim().to_string());
        }
    }
    if title.is_blank() && body.is_blank() {
        return None;
    }
    Some(Section {
        id: id.to_string(),
        title,
        body,
    })
}
