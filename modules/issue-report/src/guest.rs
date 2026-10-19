//! Livret : la carte d'accueil (mes signalements, leurs réponses) et le formulaire dérivé.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, FormFlow};
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::common::{BadgeSpec, Emphasis, Leading, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Button, Card, Icon, ListItem, Stack, Text};

use crate::{compact, ModuleConfig, Report, Reports, Status};

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

#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "issue-report",
    label_key = "nav.issue-report"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let stay = ctx.guest.as_ref().map(|g| g.session_id);
    let reports = match stay {
        Some(stay) => Reports::for_stay(&ctx, stay)?,
        None => Vec::new(),
    };
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        GUEST_FORM,
        OverlayArgs::new()
            .icon(IconName::DangerTriangle)
            .title("i18n:home.card.title"),
    );
    let mut children: Vec<Component> = Vec::new();
    if reports.is_empty() {
        children.push(
            Text::new()
                .text("i18n:home.card.intro")
                .variant(TextVariant::Body)
                .into(),
        );
    } else {
        let thanks =
            Some(config.auto_reply.for_ctx(&ctx).to_string()).filter(|t| !t.trim().is_empty());
        children.push(
            Text::new()
                .text(thanks.unwrap_or_else(|| "i18n:home.card.thanks".into()))
                .variant(TextVariant::Body)
                .into(),
        );
        children.push(caption("i18n:home.card.yourReports"));
        for entry in &reports {
            let mut item = ListItem::new()
                .title(entry.item.summary.clone())
                .subtitle(entry.item.category.label_key());
            let badge = match entry.status {
                Status::InProgress => Some(("i18n:home.card.inProgress", Tone::Info)),
                Status::Resolved => Some(("i18n:home.card.resolved", Tone::Success)),
                Status::New => None,
            };
            if let Some((label, tone)) = badge {
                item = item.trailing(Trailing::Visual(Box::new(TrailingVisual {
                    badge: Some(BadgeSpec::new(label, tone)),
                    ..TrailingVisual::default()
                })));
            }
            for reply in entry.notes() {
                item = item.child(caption(
                    &t!("home.card.reply", reply = reply.to_string())
                        .unwrap_or_else(|_| reply.to_string()),
                ));
            }
            children.push(item.into());
        }
    }
    let stay_ctx = ctx.stay.as_ref();
    let open = config.open_at(
        portaki_sdk::host::time::now()?,
        stay_ctx.and_then(|s| s.checkin_at),
        stay_ctx.and_then(|s| s.checkout_at),
    );
    let mut card = Card::new()
        .icon(IconName::DangerTriangle)
        .title("i18n:home.card.title");
    if open {
        children.push(
            ListItem::new()
                .title("i18n:home.card.openForm")
                .leading(Leading::Icon("danger-triangle".into()))
                .chevron(true)
                .action(open_form.clone())
                .into(),
        );
        card = card.action(open_form);
    } else {
        children.push(caption("i18n:home.card.closed"));
    }
    Ok(Surface::new(card.child(Stack::new().gap(12.0).children(children))).with_id(HOME_CARD))
}

fn caption(text: &str) -> Component {
    Text::new().text(text).variant(TextVariant::Caption).into()
}

fn note(text: &str) -> Component {
    Text::new()
        .text(text)
        .variant(TextVariant::Caption)
        .emphasis(Emphasis::Subtle)
        .into()
}
