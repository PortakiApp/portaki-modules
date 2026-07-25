//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, InfoBanner, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;
use crate::description;
use crate::entities::LostFoundReport;
use crate::kind;

pub fn build_home_card(reports: &[LostFoundReport]) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::ids::GUEST_FORM,
        OverlayArgs::new()
            .icon("search")
            .title("i18n:home.card.title"),
    );

    let config = load_config().unwrap_or_default();
    let mut children: Vec<Component> = Vec::new();

    if let Some(note) = config.host_note_text() {
        let plain = description::to_plain_text(note);
        if !plain.is_empty() {
            children.push(InfoBanner::new().message(plain).into());
        }
    }

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
            .leading("search")
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon("search")
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
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
