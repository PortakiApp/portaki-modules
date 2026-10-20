//! La page des réglages (§2) : la carte d'accueil, puis le tableau des appareils, dérivés de la
//! collection. Ce qui dépasse une borne s'affiche sous son champ et bloque « Publier ».

use portaki_sdk::prelude::*;
use portaki_sdk::resource::CollectionEditor;
use portaki_sdk::sdui::primitives::{Form, Page};

use crate::config::too_many_featured;
use crate::{Device, Devices};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AppliancesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Plug
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let devices = Devices::load(&ctx)?;
    let limit = devices.meta().limit();
    let too_many = too_many_featured(limit).get(&ctx.locale).to_string();
    let (a, b) = (ctx.locale.clone(), ctx.locale.clone());
    let mut editor = CollectionEditor::new(&ctx, &devices)
        .settings("host.section.homeCard", Vec::new())
        .search("host.search.placeholder")
        .filter_by("location")
        .column("name")
        .column("location")
        .check_at_most("featured", limit, too_many)
        .check("hidden")
        .pill("manual", "host.column.manual")
        .cell("name", move |d: &Device| named(&a, d))
        .cell("location", move |d: &Device| room(&b, d))
        .cell("manual", |d: &Device| manual(d));
    if !devices.is_empty() && !devices.rows().iter().any(|d| d.featured) {
        editor = editor.hint("host.featured.none");
    }
    Ok(Surface::new(
        Page::new().child(Form::new().children(editor.build())),
    ))
}

/// « 🍳 Plaques à induction ».
fn named(locale: &str, device: &Device) -> String {
    let name = device.name.get(locale).trim();
    match device.shown_emoji(name) {
        emoji if emoji.is_empty() || name.is_empty() => name.to_string(),
        emoji => format!("{emoji} {name}"),
    }
}

fn room(locale: &str, device: &Device) -> String {
    match device.location.get(locale).trim() {
        "" => "i18n:explore.detail.room.other".to_string(),
        room => room.to_string(),
    }
}

/// « Sans mode d'emploi » — pas pour une ligne tout juste ajoutée, qui n'a encore rien.
fn manual(device: &Device) -> String {
    let empty = device.steps.is_empty()
        && portaki_sdk::resource::rich_text::plain_text(device.description.get("fr")).is_empty()
        && device.manual_file.is_empty()
        && device.manual_url.is_empty();
    if empty && !device.name.get("fr").trim().is_empty() {
        "i18n:host.manual.missing".to_string()
    } else {
        String::new()
    }
}
