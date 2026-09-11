//! Module queries — list editorial sections.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::model::SectionView;
use crate::store;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSectionsArgs {
    pub locale: Option<String>,
}

#[portaki_sdk::query(name = "listSections")]
pub fn list_sections(ctx: Context, args: ListSectionsArgs) -> Result<Vec<SectionView>> {
    let locale = args.locale.unwrap_or_else(|| ctx.locale.clone());
    store::list_all(&locale, &ctx.property.locale)
}
