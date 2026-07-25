//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::IssueReport;

pub fn build_home_card(reports: &[IssueReport]) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::ids::GUEST_FORM,
        OverlayArgs::new()
            .icon("danger-triangle")
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

    children.push(
        ListItem::new()
            .title("i18n:home.card.openForm")
            .leading("danger-triangle")
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon("danger-triangle")
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
}

fn report_list_item(report: &IssueReport) -> ListItem {
    let subtitle = category::category_label_key(report.category.as_str());
    ListItem::new()
        .title(report.summary.clone())
        .subtitle(format!("i18n:{subtitle}"))
}
