//! Bouton du séjour (spec Nuki §1) : « Régénérer le code ». Le corps de la modale seulement — son
//! titre et « Fermer » sont au dashboard. La modale est la confirmation : elle dit ce qui change,
//! avertit quand le voyageur est sur place, et un seul bouton crée le code.

use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, InlineNotice, Page, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::commands::RegenerateArgs;
use crate::config::ModuleConfig;
use crate::stay_code;

#[portaki_sdk::surface(
    host,
    id = "regenerate",
    placement = HostPlacement::StayAction,
    label_key = "catalog.host.regenerate",
    icon = IconName::Refresh
)]
pub fn render_host_regenerate(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let children = body(&ctx, &config)?;
    Ok(
        Surface::new(Page::new().child(Stack::new().gap(12.0).children(children)))
            .with_id(REGENERATE),
    )
}

fn body(ctx: &HostContext, config: &ModuleConfig) -> Result<Vec<Component>> {
    // Le bouton est toujours au menu du séjour : sans code par séjour, la modale dit pourquoi et
    // mène aux réglages.
    if !stay_code::wanted(ctx, config) {
        return Ok(vec![
            caption("i18n:host.regenerate.unavailable"),
            Button::new()
                .label("i18n:host.regenerate.settings")
                .variant(ButtonVariant::Outline)
                .action(Action::navigate(
                    NavigateTarget::path(format!(
                        "/listings/{}/modules/{}",
                        ctx.property_id,
                        ctx.module_id.as_str()
                    )),
                    None,
                ))
                .into(),
        ]);
    }
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let (Some(stay_id), Some((check_in, check_out))) = (stay_id, super::stay::window(ctx)) else {
        return Ok(vec![caption("i18n:host.stay.unknown")]);
    };
    let now = time::now()?;
    if check_out <= now {
        return Ok(vec![caption("i18n:host.regenerate.ended")]);
    }
    let until = local(check_out, &ctx.property.timezone);
    let mut children = vec![Text::new()
        .text(t!("host.regenerate.body", until = &until)?)
        .variant(TextVariant::Body)
        .into()];
    if (check_in..check_out).contains(&now) {
        children.push(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message("i18n:host.regenerate.present")
                .into(),
        );
    }
    children.push(
        Button::new()
            .label("i18n:host.regenerate.submit")
            .action(ctx.module_id.command(
                crate::commands::REGENERATE_CODE,
                RegenerateArgs {
                    stay_id,
                    check_in,
                    check_out,
                },
            ))
            .into(),
    );
    Ok(children)
}

fn caption(key: &str) -> Component {
    Text::new().text(key).variant(TextVariant::Caption).into()
}

fn local(at: DateTime<Utc>, timezone: &str) -> String {
    match PropertyTz::parse(timezone) {
        Some(tz) => tz.to_local(at).format("%d/%m %H:%M").to_string(),
        None => at.format("%d/%m %H:%M").to_string(),
    }
}
