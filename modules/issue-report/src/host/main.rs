//! Host dashboard surface — quelles catégories le formulaire propose.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, FieldHint, Form, Page, Toggle};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::MessageCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let offered = config.categories();

    let mut card = Card::new()
        .title("i18n:host.category.title")
        .subtitle("i18n:host.category.subtitle")
        .icon(IconName::MessageCircle);
    for wire in crate::category::WIRE_VALUES {
        let name = format!("category_{wire}");
        card = card.child(
            Field::new()
                .name(name.clone())
                .label(format!("i18n:host.category.{wire}"))
                .child(Toggle::new().name(name).checked(offered.contains(wire))),
        );
    }
    card = card.child(FieldHint::new().text("i18n:host.category.hint"));

    Ok(Surface::new(Page::new().child(Form::new().child(card))).with_id(MAIN))
}
