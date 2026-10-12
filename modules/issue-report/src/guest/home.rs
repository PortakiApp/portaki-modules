//! Guest home booklet card — teaser + open form overlay.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::common::{BadgeSpec, Leading, Tone, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Card, ListItem, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::{IssueReport, IssueReportEvent};
use crate::status;

/// `open` : le formulaire est proposé à cette période du séjour ; sinon la carte renvoie vers
/// l'hôte. `auto_reply` : la confirmation de l'hôte, ou celle du module. `history` : les lignes
/// d'historique du séjour, dont les réponses de l'hôte.
pub fn build_home_card(
    reports: &[IssueReport],
    history: &[IssueReportEvent],
    open: bool,
    auto_reply: Option<String>,
) -> Surface {
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
            children.push(report_list_item(report, history).into());
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

/// Une ligne de l'historique : son statut dès que l'hôte l'a pris en charge (« En cours »,
/// « Résolu », §9 #5), puis les réponses de l'hôte.
fn report_list_item(report: &IssueReport, history: &[IssueReportEvent]) -> ListItem {
    let subtitle = category::category_label_key(report.category.as_str());
    let mut item = ListItem::new()
        .title(report.summary.clone())
        .subtitle(format!("i18n:{subtitle}"));
    let badge = match status::of(report) {
        status::IN_PROGRESS => Some(("i18n:home.card.inProgress", Tone::Info)),
        status::RESOLVED => Some(("i18n:home.card.resolved", Tone::Success)),
        _ => None,
    };
    if let Some((label, tone)) = badge {
        item = item.trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(BadgeSpec {
                label: label.to_string(),
                tone,
                dot: false,
            }),
            ..TrailingVisual::default()
        })));
    }
    for reply in history
        .iter()
        .filter(|event| event.report_id == report.id && event.kind == "reply")
        .filter_map(|event| event.text.as_deref())
    {
        item = item.child(
            Text::new()
                .text(
                    t!("home.card.reply", reply = reply.to_string())
                        .unwrap_or_else(|_| reply.to_string()),
                )
                .variant(TextVariant::Caption),
        );
    }
    item
}
