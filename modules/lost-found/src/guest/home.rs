//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;
use crate::description;
use crate::entities::LostFoundReport;
use crate::kind;

pub fn build_home_card(data: &GuestData) -> Surface {
    let reports = &data.reports;
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::guest::form::GUEST_FORM,
        OverlayArgs::new()
            .icon(IconName::Search)
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
        for report in reports {
            children.push(report_list_item(report).into());
        }
    }

    // Le délai passé, le formulaire disparaît : proposer un signalement qui ne sera pas traité
    // serait pire que de dire franchement que c'est trop tard (§10).
    if data.window_closed {
        children.push(
            Text::new()
                .text("i18n:home.card.windowClosed")
                .variant(TextVariant::Caption)
                .into(),
        );
        return Surface::new(
            Card::new()
                .icon(IconName::Search)
                .title("i18n:home.card.title")
                .child(Stack::new().gap(12.0).children(children)),
        )
        .with_id(crate::guest::HOME_CARD);
    }

    // Ce que l'hôte propose vraiment, au lieu d'une phrase qui annonçait les trois options à tout
    // le monde. Le voyageur sait avant d'écrire si son écharpe peut lui être renvoyée.
    for option in &data.return_options {
        let mut row = ListItem::new()
            .title(format!("i18n:guest.return.{option}"))
            .leading(Leading::Icon(return_icon(option).into()));
        if *option == "ship" && data.shipping_paid_by_guest {
            row = row.subtitle("i18n:guest.return.ship.paidByGuest");
        }
        children.push(row.into());
    }

    children.push(
        ListItem::new()
            .title("i18n:home.card.openForm")
            .leading(Leading::Icon("search".into()))
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon(IconName::Search)
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::guest::HOME_CARD)
}

/// L'icône d'une option de restitution.
fn return_icon(option: &str) -> &'static str {
    match option {
        "ship" => "package",
        "pickup" => "map-pin",
        _ => "gift",
    }
}

fn report_list_item(report: &LostFoundReport) -> ListItem {
    let subtitle = kind::kind_label_key(report.kind.as_str());
    let title = description::to_plain_text(&report.item_description);
    let title = if title.is_empty() {
        report.item_description.clone()
    } else {
        title
    };
    ListItem::new()
        .title(title)
        .subtitle(format!("i18n:{subtitle}"))
}
