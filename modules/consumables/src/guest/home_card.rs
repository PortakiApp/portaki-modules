//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::Choice;

use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::primitives::{Card, ListItem, Pill, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::empty::empty_catalog_card;
use super::load::{load_guest_consumables, GuestConsumablesData};
use crate::{Entry, Status};

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "consumables",
    label_key = "nav.consumables"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    Ok(match load_guest_consumables(&ctx)? {
        Some(data) => build_home_card(&data),
        None => empty_catalog_card(HOME_CARD),
    })
}

pub fn build_home_card(data: &GuestConsumablesData) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::guest::form::GUEST_FORM,
        OverlayArgs::new()
            .icon(IconName::Package)
            .title("i18n:home.card.title"),
    );

    let mut children: Vec<Component> = Vec::new();

    if data.reports.is_empty() {
        children.push(
            Text::new()
                .text("i18n:home.card.intro")
                .variant(TextVariant::Body)
                .into(),
        );
    } else {
        children.push(
            Text::new()
                .text("i18n:home.card.thanks")
                .variant(TextVariant::Body)
                .into(),
        );
        children.push(
            Text::new()
                .text("i18n:home.card.yourReports")
                .variant(TextVariant::Caption)
                .into(),
        );
        for report in &data.reports {
            children.push(report_list_item(report).into());
        }
    }

    // Demandes fermées, ou plafond du séjour atteint : la carte informe, l'hôte se contacte.
    if !data.can_request {
        children.push(
            Text::new()
                .text(if data.requests_disabled {
                    "i18n:home.card.requestsClosed"
                } else {
                    "i18n:home.card.requestsLimit"
                })
                .variant(TextVariant::Caption)
                .into(),
        );
        return Surface::new(
            Card::new()
                .icon(IconName::Package)
                .title("i18n:home.card.title")
                .child(Stack::new().gap(12.0).children(children)),
        )
        .with_id(HOME_CARD);
    }

    children.push(
        ListItem::new()
            .title("i18n:home.card.openForm")
            .leading(Leading::Icon("package".into()))
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon(IconName::Package)
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(HOME_CARD)
}

/// La demande, où en est l'hôte (« Prévu », « Livré ») et ce qu'il a répondu.
fn report_list_item(report: &Entry) -> ListItem {
    let mut row = ListItem::new()
        .title(report.item.item_label.clone())
        .subtitle(report.item.level.label_key());
    if report.status != Status::Open {
        row = row.child(Pill::new().label(report.status.label_key()));
    }
    if let Some(reply) = report.notes().last().filter(|r| !r.trim().is_empty()) {
        row = row.child(
            Text::new()
                .text(
                    t!("home.card.reply", reply = reply.trim().to_string())
                        .unwrap_or_else(|_| reply.trim().to_string()),
                )
                .variant(TextVariant::Caption),
        );
    }
    row
}
