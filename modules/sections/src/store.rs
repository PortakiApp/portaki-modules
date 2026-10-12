//! Section item + locale persistence via `host::repo` (+ in-memory for tests).

use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{SectionItem, SectionItemLocale};
use crate::model::{lang_code, pick_locale_fields, SectionLocaleInput, SectionView};

use std::cell::RefCell;

thread_local! {
    static TEST_ITEMS: RefCell<Vec<SectionItem>> = const { RefCell::new(Vec::new()) };
    static TEST_LOCALES: RefCell<Vec<SectionItemLocale>> = const { RefCell::new(Vec::new()) };
}

pub fn reset_test_store() {
    TEST_ITEMS.with(|rows| rows.borrow_mut().clear());
    TEST_LOCALES.with(|rows| rows.borrow_mut().clear());
}

fn in_memory_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}

pub fn list_all(locale: &str, property_locale: &str) -> Result<Vec<SectionView>> {
    let mut items = load_items()?;
    items.sort_by_key(|item| item.sort_order);
    let locales = load_locales()?;
    Ok(items
        .into_iter()
        .map(|item| {
            let item_locales: Vec<SectionLocaleInput> = locales
                .iter()
                .filter(|l| l.section_id == item.id)
                .map(|l| SectionLocaleInput {
                    lang: l.lang.clone(),
                    title: l.title.clone(),
                    body_markdown: l.body_markdown.clone(),
                })
                .collect();
            let (title, body_markdown) = pick_locale_fields(&item_locales, locale, property_locale);
            SectionView {
                id: item.id,
                sort_order: item.sort_order,
                title,
                body_markdown,
                locales: item_locales,
            }
        })
        .collect())
}

/// Upsert one section and merge provided locale rows (other langs kept).
pub fn save_section(
    id: Option<Uuid>,
    sort_order: Option<i32>,
    locales: Vec<SectionLocaleInput>,
) -> Result<SectionView> {
    let now = time::now()?;
    let items = load_items()?;
    let section_id = id.unwrap_or_else(Uuid::new_v4);
    let order = sort_order.unwrap_or_else(|| {
        items
            .iter()
            .map(|i| i.sort_order)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    });

    let item = if let Some(existing) = items.into_iter().find(|i| i.id == section_id) {
        let mut updated = existing;
        updated.sort_order = order;
        updated.updated_at = now;
        updated
    } else {
        SectionItem {
            id: section_id,
            sort_order: order,
            created_at: now,
            updated_at: now,
        }
    };
    persist_item(item.clone())?;

    for locale in &locales {
        let lang = lang_code(&locale.lang);
        if lang.is_empty() {
            continue;
        }
        upsert_locale_row(
            section_id,
            &lang,
            locale.title.clone(),
            locale.body_markdown.clone(),
            now,
        )?;
    }

    let views = list_all("fr-FR", "fr-FR")?;
    views
        .into_iter()
        .find(|v| v.id == section_id)
        .ok_or_else(|| PortakiError::Storage("section missing after save".into()))
}

fn upsert_locale_row(
    section_id: Uuid,
    lang: &str,
    title: String,
    body_markdown: String,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let existing = load_locales()?
        .into_iter()
        .find(|row| row.section_id == section_id && lang_code(&row.lang) == lang);
    if let Some(mut row) = existing {
        row.title = title;
        row.body_markdown = body_markdown;
        row.updated_at = now;
        persist_locale(row)?;
        return Ok(());
    }
    persist_locale(SectionItemLocale {
        id: Uuid::new_v4(),
        section_id,
        lang: lang.to_string(),
        title,
        body_markdown,
        created_at: now,
        updated_at: now,
    })
}

pub fn delete_section(id: Uuid) -> Result<()> {
    delete_locales_for(id)?;
    if in_memory_enabled() {
        TEST_ITEMS.with(|rows| rows.borrow_mut().retain(|item| item.id != id));
        return Ok(());
    }
    repo::delete::<SectionItem>(id)?;
    Ok(())
}

pub fn reorder(ordered_ids: Vec<Uuid>) -> Result<()> {
    let now = time::now()?;
    let mut items = load_items()?;
    for (index, id) in ordered_ids.iter().enumerate() {
        if let Some(item) = items.iter_mut().find(|i| i.id == *id) {
            item.sort_order = index as i32;
            item.updated_at = now;
            persist_item(item.clone())?;
        }
    }
    Ok(())
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

fn persist_item(row: SectionItem) -> Result<()> {
    if in_memory_enabled() {
        TEST_ITEMS.with(|rows| {
            let mut guard = rows.borrow_mut();
            if let Some(index) = guard.iter().position(|item| item.id == row.id) {
                guard[index] = row;
            } else {
                guard.push(row);
            }
        });
        return Ok(());
    }
    let _ = repo::create::<SectionItem, SectionItem, SectionItem>(row)?;
    Ok(())
}

fn persist_locale(row: SectionItemLocale) -> Result<()> {
    if in_memory_enabled() {
        TEST_LOCALES.with(|rows| {
            let mut guard = rows.borrow_mut();
            if let Some(index) = guard.iter().position(|item| item.id == row.id) {
                guard[index] = row;
            } else {
                guard.push(row);
            }
        });
        return Ok(());
    }
    let _ = repo::create::<SectionItemLocale, SectionItemLocale, SectionItemLocale>(row)?;
    Ok(())
}

fn delete_locales_for(section_id: Uuid) -> Result<()> {
    if in_memory_enabled() {
        TEST_LOCALES.with(|rows| {
            rows.borrow_mut()
                .retain(|locale| locale.section_id != section_id);
        });
        return Ok(());
    }
    let page = find::<SectionItemLocale, SectionItemLocale>(
        Query::<SectionItemLocale>::new()
            .r#where(eq("section_id", section_id))
            .limit(50),
    )?;
    for locale in page.items {
        repo::delete::<SectionItemLocale>(locale.id)?;
    }
    Ok(())
}
