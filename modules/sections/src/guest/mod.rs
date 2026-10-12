//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod home;
mod sheet;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use sheet::build_sheet_surface;

use crate::model::SectionView;
use crate::queries::{list_sections, ListSectionsArgs};

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    // La signature de l'hôte vient de la plateforme : « Claire, votre hôte depuis 2019 ». Un
    // module qui redemanderait son nom en aurait un différent dans chaque module.
    let byline = host_byline(&ctx);
    render_with_sections(&ctx, HOME_CARD, move |sections| {
        build_home_card(sections, &byline)
    })
}

/// « {prénom}, {rôle} » — vide quand la plateforme ne sert pas d'hôte (un aperçu, un espace sans
/// propriétaire). Mieux vaut pas de sous-titre qu'une virgule orpheline.
fn host_byline(ctx: &GuestContext) -> String {
    let Some(host) = ctx.host.as_ref() else {
        return String::new();
    };
    let name = host.name.trim();
    match host
        .role
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
    {
        Some(role) if !name.is_empty() => format!("{name}, {role}"),
        Some(role) => role.to_string(),
        None => name.to_string(),
    }
}

#[portaki_sdk::surface(guest, id = "explore.sheet")]
pub fn render_explore_sheet(ctx: GuestContext) -> Result<Surface> {
    render_with_sections(&ctx, EXPLORE_SHEET, build_sheet_surface)
}

fn render_with_sections(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: impl FnOnce(&[SectionView]) -> Surface,
) -> Result<Surface> {
    let sections = list_sections(
        ctx.clone(),
        ListSectionsArgs {
            locale: Some(ctx.locale.clone()),
        },
    )?;
    let visible: Vec<SectionView> = sections.into_iter().filter(|s| !s.is_blank()).collect();
    if visible.is_empty() {
        return Ok(no_sections_state(surface_id));
    }
    Ok(build(&visible))
}

/// No section written yet.
fn no_sections_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Home),
    )
    .with_id(surface_id)
}
