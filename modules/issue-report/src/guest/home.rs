//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::common::{BadgeSpec, Leading, Tone, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::IssueReport;

/// `open` : le formulaire est proposé à cette période du séjour ; sinon la carte renvoie vers
/// l'hôte. `auto_reply` : la confirmation de l'hôte, ou celle du module.
pub fn build_home_card(reports: &[IssueReport], open: bool, auto_reply: Option<String>) -> Surface {
    let open_form = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::guest::form::GUEST_FORM,
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
        children.push(
            Text::new()
                .text(auto_reply.unwrap_or_else(|| "i18n:home.card.thanks".to_string()))
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

    // Hors des périodes choisies : pas de formulaire, l'hôte se contacte directement (§9.4).
    if !open {
        children.push(
            Text::new()
                .text("i18n:home.card.closed")
                .variant(TextVariant::Caption)
                .into(),
        );
        return Surface::new(
            Card::new()
                .icon(IconName::DangerTriangle)
                .title("i18n:home.card.title")
                .child(Stack::new().gap(12.0).children(children)),
        )
        .with_id(crate::guest::HOME_CARD);
    }

    children.push(
        ListItem::new()
            .title("i18n:home.card.openForm")
            .leading(Leading::Icon("danger-triangle".into()))
            .chevron(true)
            .action(open_form.clone())
            .into(),
    );

    Surface::new(
        Card::new()
            .icon(IconName::DangerTriangle)
            .title("i18n:home.card.title")
            .action(open_form)
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::guest::HOME_CARD)
}

/// Une ligne de l'historique, marquée « Résolu » dès que l'hôte l'a clos (§9 #5).
fn report_list_item(report: &IssueReport) -> ListItem {
    let subtitle = category::category_label_key(report.category.as_str());
    let item = ListItem::new()
        .title(report.summary.clone())
        .subtitle(format!("i18n:{subtitle}"));
    if report.resolved_at.is_none() {
        return item;
    }
    item.trailing(Trailing::Visual(Box::new(TrailingVisual {
        badge: Some(BadgeSpec {
            label: "i18n:home.card.resolved".to_string(),
            tone: Tone::Success,
            dot: false,
        }),
        ..TrailingVisual::default()
    })))
}
