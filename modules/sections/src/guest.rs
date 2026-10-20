//! Livret : la carte d'accueil montre le début du mot, la feuille le mot entier, signé de l'hôte.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::rich_text;
use portaki_sdk::sdui::common::Author;
use portaki_sdk::sdui::primitives::{Card, Divider, EmptyState, RichText, Stack, Text};

use crate::{Section, Sections};

/// Les sections que la carte montre.
const CARD_LIMIT: usize = 2;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let sections = visible(&ctx)?;
    if sections.is_empty() {
        return Ok(empty(HOME_CARD));
    }
    // La signature de l'hôte vient de la plateforme : « Claire, votre hôte depuis 2019 ».
    let card = Card::new()
        .icon(IconName::Home)
        .title("i18n:home.card.title")
        .subtitle(host_byline(&ctx))
        .action(Action::open_overlay(
            OverlayPresentation::BottomSheet,
            EXPLORE_SHEET,
            OverlayArgs::new()
                .icon(IconName::Home)
                .title("i18n:home.card.title"),
        ))
        .children(blocks(&ctx, &sections, CARD_LIMIT));
    Ok(Surface::new(card).with_id(HOME_CARD))
}

/// Le mot entier ; la signature va sous le texte, pas sous chaque section, et c'est la coquille
/// qui la remplit (`Author`, « always the host, filled by the platform »).
#[portaki_sdk::surface(guest, id = "explore.sheet")]
pub fn render_explore_sheet(ctx: GuestContext) -> Result<Surface> {
    let sections = visible(&ctx)?;
    if sections.is_empty() {
        return Ok(empty(EXPLORE_SHEET));
    }
    let all = Stack::new()
        .gap(12.0)
        .children(blocks(&ctx, &sections, sections.len()));
    let sheet = Stack::new().gap(14.0).children(vec![
        all.into(),
        RichText::new().signature(Author::default()).into(),
    ]);
    Ok(Surface::new(sheet).with_id(EXPLORE_SHEET))
}

fn visible(ctx: &Context) -> Result<Vec<Section>> {
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
fn blocks(ctx: &Context, sections: &[Section], limit: usize) -> Vec<Component> {
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
fn host_byline(ctx: &Context) -> String {
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

fn empty(surface: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Home),
    )
    .with_id(surface)
}
