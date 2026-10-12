//! Module queries — the sections in the reader's language, and what blocks « Publier ».

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::ModuleConfig;
use crate::i18n::text;
use crate::model::SectionView;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSectionsArgs {
    pub locale: Option<String>,
}

#[portaki_sdk::query(
    name = "listSections",
    example(label = "Sections en français", input = r#"{"locale":"fr-FR"}"#),
    example(label = "Sections en anglais", input = r#"{"locale":"en-US"}"#)
)]
pub fn list_sections(ctx: Context, args: ListSectionsArgs) -> Result<Vec<SectionView>> {
    let mut reader = ctx.clone();
    if let Some(locale) = args.locale.filter(|l| !l.trim().is_empty()) {
        reader.locale = locale;
    }
    let config = ModuleConfig::load(&ctx)?;
    Ok(config
        .sections()
        .into_iter()
        .map(|row| SectionView {
            title: row.title.for_ctx(&reader).to_string(),
            body_markdown: row.body.for_ctx(&reader).to_string(),
            photo: Some(row.photo.trim().to_string()).filter(|p| !p.is_empty()),
            id: row.id,
        })
        .collect())
}

/// The rules of the card, checked again before « Publier ». Each point names its field
/// (`config.sections[1].title`) for the dashboard to scroll to.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(PublishReadiness {
        items: config
            .problems()
            .into_iter()
            .map(|problem| PublishCheck {
                id: problem.id,
                level: if problem.blocking {
                    PublishLevel::Required
                } else {
                    PublishLevel::Recommended
                },
                ok: false,
                label: text(problem.label),
                hint: problem.message,
            })
            .collect(),
    })
}
