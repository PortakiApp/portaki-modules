//! Tableau de bord : les réglages du formulaire, et la boîte de réception dérivée du dépôt.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Inbox};
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::primitives::{
    Button, Field, FieldHint, InlineNotice, Page, Pill, Stack, Text,
};

use crate::{compact, ModuleConfig, Reports};

#[portaki_sdk::surface(host, id = "reports", placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.reports", icon = IconName::DangerTriangle)]
pub fn render_host_reports(ctx: HostContext) -> Result<Surface> {
    let inbox = Inbox::<Reports>::new(&ctx, Reports::DECL, "host")
        .icon(IconName::DangerTriangle)
        .tag(|r| format!("i18n:host.category.{}", r.category.wire()))
        .detail_extra(|e| {
            let dates = match (e.check_in, e.check_out) {
                (Some(from), Some(to)) => {
                    format!("{} → {}", from.format("%d/%m"), to.format("%d/%m"))
                }
                _ => String::new(),
            };
            let line = [e.guest_name.as_str(), dates.as_str()]
                .into_iter()
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join(" · ");
            let stay = e
                .stay_id
                .map(|id| NavigateTarget::path(format!("/stays/{id}")));
            vec![Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(12.0)
                .child(Text::new().text(line).variant(TextVariant::Caption))
                .child(
                    Button::new()
                        .label("i18n:host.detail.openStay")
                        .variant(ButtonVariant::Outline)
                        .action(Action::navigate(
                            stay.unwrap_or_else(|| NavigateTarget::path("/stays")),
                            None,
                        )),
                )
                .into()]
        });
    Ok(Surface::new(Page::new().child(inbox.build()?)))
}

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
