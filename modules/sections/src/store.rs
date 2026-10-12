//! The sections of before the platform held them: `section_item` + `section_item_locale`, read
//! only (spec §8). The host's first save writes `sections` in the config, which wins from then on.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::repo::{find, Query};
use portaki_sdk::prelude::*;

use crate::config::SectionRow;
use crate::entities::{SectionItem, SectionItemLocale};
use crate::model::lang_code;
use uuid::Uuid;

use std::cell::RefCell;

thread_local! {
    static TEST_ITEMS: RefCell<Vec<SectionItem>> = const { RefCell::new(Vec::new()) };
    static TEST_LOCALES: RefCell<Vec<SectionItemLocale>> = const { RefCell::new(Vec::new()) };
}

pub fn reset_test_store() {
    TEST_ITEMS.with(|rows| rows.borrow_mut().clear());
    TEST_LOCALES.with(|rows| rows.borrow_mut().clear());
}

/// Seeds one section of before, `(lang, title, body)` per language — tests only.
#[doc(hidden)]
pub fn seed_legacy_section(sort_order: i32, locales: &[(&str, &str, &str)]) -> Uuid {
    let id = Uuid::new_v4();
    let now = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH;
    TEST_ITEMS.with(|rows| {
        rows.borrow_mut().push(SectionItem {
            id,
            sort_order,
            created_at: now,
            updated_at: now,
        })
    });
    TEST_LOCALES.with(|rows| {
        rows.borrow_mut()
            .extend(locales.iter().map(|(lang, title, body)| SectionItemLocale {
                id: Uuid::new_v4(),
                section_id: id,
                lang: (*lang).into(),
                title: (*title).into(),
                body_markdown: (*body).into(),
                created_at: now,
                updated_at: now,
            }))
    });
    id
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

/// The sections of the tables, in their order, one `I18nText` per field: each language row
/// becomes that language of the text.
pub fn legacy_rows() -> Result<Vec<SectionRow>> {
    let mut items = load_items()?;
    items.sort_by_key(|item| item.sort_order);
    let locales = load_locales()?;
    Ok(items
        .into_iter()
        .map(|item| {
            let mut title = I18nText::default();
            let mut body = I18nText::default();
            for row in locales.iter().filter(|l| l.section_id == item.id) {
                let lang = lang_code(&row.lang);
                title = set(title, &lang, &row.title);
                body = set(body, &lang, &row.body_markdown);
            }
            SectionRow {
                id: item.id.to_string(),
                title,
                body,
                photo: String::new(),
            }
        })
        .collect())
}

fn set(mut text: I18nText, lang: &str, value: &str) -> I18nText {
    match lang {
        "fr" => text.fr = value.to_string(),
        "en" => text.en = value.to_string(),
        other => {
            text.others.insert(other.to_string(), value.to_string());
        }
    }
    text
}

fn load_items() -> Result<Vec<SectionItem>> {
    if in_memory_enabled() {
        return Ok(TEST_ITEMS.with(|rows| rows.borrow().clone()));
    }
    let page = find::<SectionItem, SectionItem>(Query::<SectionItem>::new().limit(200))?;
    Ok(page.items)
}

fn load_locales() -> Result<Vec<SectionItemLocale>> {
    if in_memory_enabled() {
        return Ok(TEST_LOCALES.with(|rows| rows.borrow().clone()));
    }
    let page =
        find::<SectionItemLocale, SectionItemLocale>(Query::<SectionItemLocale>::new().limit(500))?;
    Ok(page.items)
}
