//! La carte d'accueil : mes signalements, leurs réponses, et l'ouverture du formulaire.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::Choice;
use portaki_sdk::sdui::common::{BadgeSpec, Leading, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};

use super::caption;
use super::form::GUEST_FORM;
use crate::{ModuleConfig, Reports, Status};

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
