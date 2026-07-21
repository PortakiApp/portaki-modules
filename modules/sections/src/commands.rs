//! Module commands — save / delete / reorder sections.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{lang_code, SectionLocaleInput, SectionView};
use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSectionArgs {
    pub id: Option<Uuid>,
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub locales: Vec<SectionLocaleInput>,
    /// Active host locale content (preferred).
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body_markdown: String,
    /// Explicit lang for single-locale fields (`fr`, `en`, …). Defaults to `ctx.locale`.
    #[serde(default)]
    pub lang: String,
    /// Legacy dual-locale convenience fields.
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub body_markdown_fr: String,
    #[serde(default)]
    pub body_markdown_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSectionArgs {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderArgs {
    pub ordered_ids: Vec<Uuid>,
}

#[portaki_sdk::command(name = "saveSection")]
pub fn save_section(ctx: Context, args: SaveSectionArgs) -> Result<SectionView> {
    let mut locales = args.locales;
    if !args.title.trim().is_empty() || !args.body_markdown.trim().is_empty() {
        let lang = if args.lang.trim().is_empty() {
            lang_code(&ctx.locale)
        } else {
            lang_code(&args.lang)
        };
        upsert_locale(&mut locales, &lang, args.title, args.body_markdown);
    }
    if !args.title_fr.trim().is_empty() || !args.body_markdown_fr.trim().is_empty() {
        upsert_locale(&mut locales, "fr", args.title_fr, args.body_markdown_fr);
    }
    if !args.title_en.trim().is_empty() || !args.body_markdown_en.trim().is_empty() {
        upsert_locale(&mut locales, "en", args.title_en, args.body_markdown_en);
    }
    store::save_section(args.id, args.sort_order, locales)
}

#[portaki_sdk::command(name = "deleteSection")]
pub fn delete_section(_ctx: Context, args: DeleteSectionArgs) -> Result<()> {
    store::delete_section(args.id)
}

#[portaki_sdk::command(name = "reorder")]
pub fn reorder(_ctx: Context, args: ReorderArgs) -> Result<()> {
    store::reorder(args.ordered_ids)
}

fn upsert_locale(
    locales: &mut Vec<SectionLocaleInput>,
    lang: &str,
    title: String,
    body_markdown: String,
) {
    let lang = lang_code(lang);
    if let Some(existing) = locales.iter_mut().find(|l| lang_code(&l.lang) == lang) {
        existing.title = title;
        existing.body_markdown = body_markdown;
    } else {
        locales.push(SectionLocaleInput {
            lang,
            title,
            body_markdown,
        });
    }
}
