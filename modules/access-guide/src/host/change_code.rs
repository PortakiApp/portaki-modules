//! Bouton du séjour (spec Accès §1) : « Changer le code d'entrée ». Le corps de la modale
//! seulement — son titre et « Fermer » sont au dashboard. La modale est la confirmation : elle
//! avertit quand le voyageur voit déjà son code (§5), et un bouton enregistre.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Field, FieldHint, Form, InlineNotice, Page, Stack, Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::commands::StayCodeArgs;
use crate::config::ModuleConfig;
use crate::reveal::evaluate_reveal;

#[portaki_sdk::surface(
    host,
    id = "change-code",
    placement = HostPlacement::StayAction,
    label_key = "catalog.host.changeCode",
    icon = IconName::Key
)]
pub fn render_host_change_code(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::read(&ctx)?;
    let children = body(&ctx, &config)?;
    Ok(
        Surface::new(Page::new().child(Stack::new().gap(12.0).children(children)))
            .with_id(CHANGE_CODE),
    )
}

fn body(ctx: &HostContext, config: &ModuleConfig) -> Result<Vec<Component>> {
    // Le bouton est toujours au menu du séjour : sans code par séjour, la modale dit pourquoi et
    // mène aux réglages.
    if !config.code_per_stay {
        return Ok(vec![
            caption("i18n:host.changeCode.unavailable"),
            Button::new()
                .label("i18n:host.changeCode.settings")
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
    let (Some(stay_id), Some((check_in, check_out))) =
        (super::stay::stay_id(ctx), super::stay::window(ctx))
    else {
        return Ok(vec![caption("i18n:host.changeCode.unknownStay")]);
    };
    let now = time::now()?;
    if check_out <= now {
        return Ok(vec![caption("i18n:host.changeCode.ended")]);
    }
    let mut children = Vec::new();
    // Déjà révélé : le voyageur verra le nouveau code tout de suite (§5).
    let decision = evaluate_reveal(
        config.reveal_policy,
        config.reveal_hours,
        now,
        Some(check_in),
        Some(check_out),
        &ctx.property.timezone,
    );
    if decision.revealed {
        children.push(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message("i18n:host.changeCode.revealed")
                .into(),
        );
    }
    let save = ctx.module_id.command(
        crate::commands::SET_STAY_CODE,
        StayCodeArgs {
            stay_id,
            code: None,
            generate: false,
        },
    );
    let generate = ctx.module_id.command(
        crate::commands::SET_STAY_CODE,
        StayCodeArgs {
            stay_id,
            code: None,
            generate: true,
        },
    );
    children.push(
        Form::new()
            .child(
                Field::new()
                    .name("code")
                    .label("i18n:host.changeCode.code")
                    .required(true)
                    .child(TextInput::new().name("code").placeholder("4821"))
                    .child(FieldHint::new().text("i18n:host.changeCode.code.hint")),
            )
            .child(
                Button::new()
                    .label("i18n:host.changeCode.submit")
                    .action(save),
            )
            .into(),
    );
    children.push(
        Button::new()
            .label("i18n:host.changeCode.generate")
            .variant(ButtonVariant::Outline)
            .action(generate)
            .into(),
    );
    Ok(children)
}

fn caption(key: &str) -> Component {
    Text::new().text(key).variant(TextVariant::Caption).into()
}
