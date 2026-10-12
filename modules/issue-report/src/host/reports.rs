//! L'onglet du logement (spec Signaler §1, `property-workspace-tab`) : les signalements, filtrés
//! À traiter (Nouveau et En cours) · Résolus · Tous, et « Ajouter un signalement ». Une ligne
//! ouvre le détail : statut, séjour, réponse au voyageur, historique.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, FilterBar, FilterChip, Page, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::add::{form, AddStay};
use crate::{status, storage};

/// `input.filter` : `open` (défaut), `resolved` ou `all`.
const FILTERS: &[&str] = &["open", "resolved", "all"];

#[portaki_sdk::surface(
    host,
    id = "reports",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.reports",
    icon = IconName::DangerTriangle
)]
pub fn render_host_reports(ctx: HostContext) -> Surface {
    if let Some(detail) = super::detail_from_input(&ctx) {
        return Surface::new(detail).with_id(REPORTS);
    }
    // « Ajouter un signalement » ouvre la même surface dans la fenêtre du dashboard.
    if ctx.input_str("add").is_some() {
        return Surface::new(Page::new().child(add_body(&ctx))).with_id(REPORTS);
    }
    let mut reports = storage::list_since(DateTime::<Utc>::UNIX_EPOCH).unwrap_or_default();
    reports.sort_by_key(|report| std::cmp::Reverse(report.created_at));
    let filter = ctx
        .input_str("filter")
        .and_then(|raw| FILTERS.iter().copied().find(|key| *key == raw))
        .unwrap_or("open");
    let to_handle = reports
        .iter()
        .filter(|report| status::of(report) != status::RESOLVED)
        .count();
    let shown: Vec<_> = reports
        .iter()
        .filter(|report| match filter {
            "open" => status::of(report) != status::RESOLVED,
            "resolved" => status::of(report) == status::RESOLVED,
            _ => true,
        })
        .collect();

    let chips = FILTERS
        .iter()
        .map(|key| {
            let label = match *key {
                "open" => {
                    t!("host.tab.filter.open", count = to_handle.to_string()).unwrap_or_default()
                }
                other => format!("i18n:host.tab.filter.{other}"),
            };
            FilterChip::new()
                .label(label)
                .selected(*key == filter)
                .action(Action::emit(
                    crate::ids::HOST_SURFACE_INPUT,
                    Some(json!({ "filter": key })),
                ))
                .into()
        })
        .collect();
    let add = Button::new()
        .label("i18n:catalog.host.add")
        .icon(IconName::Plus)
        .action(Action::emit(
            crate::ids::HOST_SURFACE_OVERLAY,
            Some(json!({
                "input": { "add": "1" },
                "icon": "danger-triangle",
                "title": t!("catalog.host.add").unwrap_or_default(),
            })),
        ));
    let header = Stack::new()
        .direction(StackDirection::Horizontal)
        .gap(12.0)
        .child(FilterBar::new().children(chips))
        .child(add);

    let body: Component = if shown.is_empty() {
        EmptyState::new()
            .title(format!("i18n:host.tab.empty.{filter}"))
            .description("i18n:host.main.emptyRecent.help")
            .icon(IconName::DangerTriangle)
            .into()
    } else {
        let now = super::host_now();
        Stack::new()
            .gap(0.0)
            .children(
                shown
                    .iter()
                    .map(|report| super::report_row(report, now, &ctx.locale))
                    .collect(),
            )
            .into()
    };
    Surface::new(
        Page::new().child(
            Stack::new()
                .gap(16.0)
                .child(header)
                .child(Card::new().title("i18n:catalog.host.reports").child(body)),
        ),
    )
    .with_id(REPORTS)
}

/// Le formulaire d'ajout, avec le choix du séjour parmi ceux que le dashboard passe à l'onglet.
fn add_body(ctx: &HostContext) -> Component {
    let stays: Vec<(String, String)> = ctx
        .input
        .get("stays")
        .and_then(|stays| stays.as_array())
        .into_iter()
        .flatten()
        .filter_map(|stay| {
            let id = stay.get("id")?.as_str()?.to_string();
            let label = stay.get("label")?.as_str()?.to_string();
            Some((id, label))
        })
        .collect();
    if stays.is_empty() {
        return Text::new()
            .text("i18n:host.add.noStay")
            .variant(TextVariant::Caption)
            .into();
    }
    form(AddStay::Choose(stays)).into()
}
