//! Le formulaire du voyageur, dérivé du signalement : catégories offertes, photo, urgence.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::FormFlow;
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Button, Icon, Stack, Text};

use super::note;
use crate::{compact, ModuleConfig, Report};

#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "issue-report/form",
    label_key = "nav.issue-report"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let header = Stack::new()
        .direction(StackDirection::Horizontal)
        .gap(12.0)
        .child(
            Icon::new()
                .name(IconName::DangerTriangle)
                .size(24.0)
                .tone(Tone::Primary),
        )
        .child(
            Stack::new()
                .gap(4.0)
                .child(
                    Text::new()
                        .text("i18n:form.head.title")
                        .variant(TextVariant::Title),
                )
                .child(
                    Text::new()
                        .text("i18n:form.head.lead")
                        .variant(TextVariant::Caption)
                        .emphasis(Emphasis::Subtle),
                ),
        );
    let mut form = FormFlow::<Report>::new(&ctx)
        .header(header)
        .only("category", config.categories());
    if !config.photo_allowed.unwrap_or(true) {
        form = form.without("photo");
    }
    if let Some(category) = ctx.input_str("category") {
        form = form.prefill("category", category);
    }
    if let Some(summary) = ctx.input_str("summary") {
        form = form.prefill("summary", summary.chars().take(120).collect::<String>());
    }
    if config.response_time() != "none" {
        form = form.footer(note(&format!(
            "i18n:form.responseTime.{}",
            config.response_time()
        )));
    }
    let phone = config
        .urgent_phone
        .unwrap_or(true)
        .then(|| {
            Some(compact(&config.urgent_number))
                .filter(|p| !p.is_empty())
                .or_else(|| {
                    ctx.host
                        .as_ref()
                        .and_then(|h| h.phone.as_deref())
                        .map(compact)
                        .filter(|p| !p.is_empty())
                })
        })
        .flatten();
    form = match phone {
        Some(phone) => form.footer(note("i18n:form.urgent.callNote")).footer(
            Button::new()
                .label("i18n:form.urgent.call")
                .variant(ButtonVariant::Ghost)
                .action(Action::external(format!("tel:{phone}"))),
        ),
        None => form.footer(note("i18n:form.urgent.note")).footer(
            Button::new()
                .label("i18n:form.urgent.link")
                .variant(ButtonVariant::Ghost)
                .action(Action::navigate(NavigateTarget::path("aide"), None)),
        ),
    };
    Ok(Surface::new(form.build()).with_id(GUEST_FORM))
}
