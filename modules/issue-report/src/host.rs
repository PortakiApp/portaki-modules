//! Tableau de bord : les réglages du formulaire, et la boîte de réception dérivée du dépôt.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Inbox};
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::primitives::{
    Button, Card, ChoiceList, Field, FieldHint, Form, InlineNotice, Page, PhoneInput, Pill, Stack,
    Text, TextArea, ToggleRow,
};

use crate::{compact, Category, ModuleConfig, Reports, PHASES};

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
    let field = |name: &str| match config.error_of(name) {
        Some(error) => Field::new()
            .name(name)
            .error(error.get(&ctx.locale).to_string()),
        None => Field::new().name(name),
    };
    let categories = config.categories();
    let mut category = field("categories")
        .label("i18n:host.category.title")
        .child(multi(
            "categories",
            Category::ALL
                .iter()
                .map(|c| ChoiceOption::new(c.wire(), format!("i18n:host.category.{}", c.wire())))
                .collect(),
            &categories,
        ));
    if categories.len() == 1 {
        category = category.child(FieldHint::new().text("i18n:host.category.one"));
    }
    let phases = field("phases")
        .label("i18n:host.phase.title")
        .child(multi(
            "phases",
            PHASES
                .iter()
                .map(|p| ChoiceOption::new(*p, format!("i18n:host.phase.{p}")))
                .collect(),
            &config.phases(),
        ))
        .child(FieldHint::new().text("i18n:host.phase.hint"));
    let form = Card::new()
        .title("i18n:host.form.title")
        .child(category)
        .child(phases)
        .child(
            ToggleRow::new()
                .name("photo_allowed")
                .label("i18n:host.photo.label")
                .description("i18n:host.photo.help")
                .checked(config.photo_allowed.unwrap_or(true)),
        )
        .child(
            ToggleRow::new()
                .name("urgent_phone")
                .label("i18n:host.urgent.label")
                .description("i18n:host.urgent.hint")
                .checked(config.urgent_phone.unwrap_or(true)),
        )
        .child(
            field("urgent_number")
                .label("i18n:host.urgentNumber.label")
                .visibleWhen("urgent_phone=true")
                .child(
                    PhoneInput::new()
                        .name("urgent_number")
                        .value(compact(&config.urgent_number)),
                )
                .child(FieldHint::new().text("i18n:host.urgentNumber.hint")),
        )
        .child(
            Field::new().visibleWhen("urgent_phone=false").child(
                InlineNotice::new()
                    .tone(Tone::Neutral)
                    .message("i18n:host.urgent.off"),
            ),
        );
    let response = ctx
        .input_str("response_time")
        .unwrap_or_else(|| config.response_time());
    let reception = Card::new()
        .title("i18n:host.reception.title")
        .child(
            Field::new().label("i18n:host.notify.label").child(
                Stack::new()
                    .direction(StackDirection::Horizontal)
                    .gap(6.0)
                    .child(Pill::new().label("i18n:host.notify.email"))
                    .child(Pill::new().label("i18n:host.notify.app"))
                    .child(Pill::new().label("i18n:host.notify.sms")),
            ),
        )
        .child(
            field("auto_reply")
                .label("i18n:host.autoReply.label")
                .child(
                    TextArea::new()
                        .name("auto_reply")
                        .rows(2)
                        .value(config.auto_reply.host_value(&ctx))
                        .placeholder("i18n:host.autoReply.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.autoReply.hint")),
        )
        .child(
            Field::new()
                .name("response_time")
                .label("i18n:host.responseTime.label")
                .child(
                    ChoiceList::new()
                        .name("response_time")
                        .layout(ChoiceListLayout::Segmented)
                        .emitOnChange(true)
                        .choices(
                            ["within_hour", "within_day", "none"]
                                .iter()
                                .map(|k| {
                                    ChoiceOption::new(
                                        *k,
                                        format!("i18n:host.responseTime.label.{k}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(response),
                )
                .child(FieldHint::new().text(format!("i18n:host.responseTime.hint.{response}"))),
        );
    Ok(Surface::new(Page::new().child(Form::new().child(
        Stack::new().gap(16.0).child(form).child(reception),
    ))))
}

fn multi(name: &str, choices: Vec<ChoiceOption>, chosen: &[&str]) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .multi(true)
        .layout(ChoiceListLayout::Compact)
        .choices(choices)
        .value(serde_json::to_string(chosen).unwrap_or_default())
}
