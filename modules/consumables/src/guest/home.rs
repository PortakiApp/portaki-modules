//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestConsumablesData;
use crate::entities::ConsumableReport;
use crate::level;

pub fn build_home_card(data: &GuestConsumablesData) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::ids::GUEST_FORM,
        OverlayArgs::new()
            .icon("package")
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

    children.push(
        ListItem::new()
            .title("i18n:home.card.openForm")
            .leading("package")
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon("package")
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
}

fn report_list_item(report: &ConsumableReport) -> ListItem {
    let subtitle = level::level_label_key(report.level.as_str());
    ListItem::new()
        .title(report.item_label.clone())
        .subtitle(format!("i18n:{subtitle}"))
}
