//! Livret : ce que la carte et la feuille partagent — les sections visibles, leurs blocs, la
//! signature de l'hôte.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::rich_text;
use portaki_sdk::sdui::primitives::{Divider, EmptyState, Text};

use crate::{Section, Sections};

pub(crate) fn visible(ctx: &Context) -> Result<Vec<Section>> {
    Ok(Sections::load(ctx)?
        .rows()
        .iter()
        .filter(|s| {
            !s.title.for_ctx(ctx).trim().is_empty()
                || rich_text::component(s.body.for_ctx(ctx)).is_some()
        })
        .cloned()
        .collect())
}

/// Le titre d'une section n'est répété que si plusieurs se partagent le bloc : la carte a déjà le
/// sien.
pub(crate) fn blocks(ctx: &Context, sections: &[Section], limit: usize) -> Vec<Component> {
    let titled = sections.len() > 1;
    let mut out = Vec::new();
    for (index, section) in sections.iter().take(limit).enumerate() {
        if index > 0 {
            out.push(Divider::new().into());
        }
        let title = section.title.for_ctx(ctx).trim();
        if titled && !title.is_empty() {
            out.push(Text::new().text(title).variant(TextVariant::Caption).into());
        }
        out.extend(rich_text::component(section.body.for_ctx(ctx)));
    }
    if out.is_empty() {
        let none = Text::new().text("i18n:home.card.empty.description");
        out.push(none.variant(TextVariant::Body).into());
    }
    out
}

/// « {prénom}, {rôle} » — vide sans hôte (un aperçu) : pas de virgule orpheline.
pub(crate) fn host_byline(ctx: &Context) -> String {
    let Some(host) = ctx.host.as_ref() else {
        return String::new();
    };
    let role = host.role.as_deref().map(str::trim).unwrap_or_default();
    [host.name.trim(), role]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn empty(surface: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Home),
    )
    .with_id(surface)
}
