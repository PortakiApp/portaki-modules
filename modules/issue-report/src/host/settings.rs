//! Les réglages du formulaire, dérivés de `#[config(form)]`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Field, FieldHint, InlineNotice, Page, Pill, Stack};

use crate::{compact, ModuleConfig};

#[portaki_sdk::nav(placement = HostPlacement::WorkspaceTimelineTask, path = "tasks",
    label_key = "catalog.host.tasks", icon = IconName::DangerTriangle)]
#[portaki_sdk::surface(host, id = "main", placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main", icon = IconName::MessageCircle)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let response = ctx
        .input_str("response_time")
        .unwrap_or_else(|| config.response_time());
    let pills =
        ["email", "app", "sms"].map(|k| Pill::new().label(format!("i18n:host.notify.{k}")).into());
    let mut form = config
        .form(&ctx)
        .errors(|key| config.error_of(key))
        .value("categories", config.categories())
        .value("phases", config.phases())
        .value("photo_allowed", config.photo_allowed.unwrap_or(true))
        .value("urgent_phone", config.urgent_phone.unwrap_or(true))
        .value("urgent_number", compact(&config.urgent_number))
        .value("response_time", response)
        .after(
            "urgent_number",
            Field::new().visibleWhen("urgent_phone=false").child(
                InlineNotice::new()
                    .tone(Tone::Neutral)
                    .message("i18n:host.urgent.off"),
            ),
        )
        .before(
            "auto_reply",
            Field::new().label("i18n:host.notify.label").child(
                Stack::new()
                    .direction(StackDirection::Horizontal)
                    .gap(6.0)
                    .children(pills.to_vec()),
            ),
        )
        .inside(
            "response_time",
            FieldHint::new().text(format!("i18n:host.responseTime.hint.{response}")),
        );
    if config.categories().len() == 1 {
        form = form.inside(
            "categories",
            FieldHint::new().text("i18n:host.category.one"),
        );
    }
    Ok(Surface::new(Page::new().child(form.build())))
}
