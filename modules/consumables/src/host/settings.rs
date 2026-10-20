//! La page des réglages (§2) : les produits et les réglages des demandes, dérivés de la
//! collection, puis les demandes reçues, qui ont leurs propres commandes.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::CollectionEditor;
use portaki_sdk::sdui::primitives::{
    Button, DataTable, EmptyState, Form, InlineNotice, List, Page, Section, Stack, Text,
};
use serde_json::{json, Value};
use uuid::Uuid;

use super::report_ui::{build_report_block, format_relative_when, short_date};
use crate::entities::ConsumableReport;
use crate::{status, storage, suggest, Product, Products};

/// « Demandés récemment » : une demande dans les trente derniers jours.
const RECENT_DAYS: i64 = 30;

/// La nav dit au dashboard d'appeler `timelineTasks` : « Café demandé · Marie » dans À venir.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Package
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::ConsumablesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Package
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let products = Products::load(&ctx)?;
    let reports = storage::list_all().unwrap_or_default();
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let last = last_requests(&reports);
    let (locale, when, filter) = (ctx.locale.clone(), last.clone(), last);
    let host_locale = ctx.locale.clone();
    let editor = CollectionEditor::new(&ctx, &products)
        .hint("host.products.help")
        .search("host.products.search")
        .filter_by("requested")
        .column("label")
        .column("location")
        .computed("last", "host.column.lastRequest")
        .cell("label", move |p: &Product| shown(p, &locale))
        .cell("last", move |p: &Product| match when.get(&p.uuid()) {
            Some(at) => format_relative_when(*at, now, &host_locale),
            None => "—".to_string(),
        })
        .cell("requested", move |p: &Product| {
            requested(filter.get(&p.uuid()), now)
        })
        .when_empty(starter())
        .settings("host.requests.title", Vec::new())
        .settings_last();
    let received = received_section(&products, &reports, now, &ctx.locale);
    let page = Stack::new()
        .gap(16.0)
        .child(Form::new().children(editor.build()))
        .child(received);
    Ok(Surface::new(Page::new().child(page)))
}

/// Première ouverture, liste vide : la liste de départ (§4), en un clic.
fn starter() -> Component {
    let help = Text::new()
        .text("i18n:host.starter.help")
        .variant(TextVariant::Caption);
    let add = Button::new()
        .label("i18n:host.starter.add")
        .variant(ButtonVariant::Outline)
        .icon(IconName::Sparkles)
        .action(crate::ids::module_id().command_empty(crate::commands::SEED_DEFAULTS));
    Stack::new().gap(8.0).child(help).child(add).into()
}

/// « ☕ Café ».
fn shown(product: &Product, locale: &str) -> String {
    let name = product.name(locale);
    match suggest::shown(&product.emoji, &name) {
        emoji if emoji.is_empty() || name.is_empty() => name,
        emoji => format!("{emoji} {name}"),
    }
}

/// La dernière demande de chaque produit.
fn last_requests(reports: &[ConsumableReport]) -> HashMap<Uuid, DateTime<Utc>> {
    let mut last: HashMap<Uuid, DateTime<Utc>> = HashMap::new();
    for report in reports {
        let at = last.entry(report.item_id).or_insert(report.created_at);
        *at = (*at).max(report.created_at);
    }
    last
}

/// Le filtre : jamais demandé, ou demandé récemment ; plus ancien, seulement dans « Tous ».
fn requested(last: Option<&DateTime<Utc>>, now: DateTime<Utc>) -> String {
    match last {
        None => "i18n:host.filter.never",
        Some(at) if (now - *at).num_days() < RECENT_DAYS => "i18n:host.filter.recent",
        Some(_) => "",
    }
    .to_string()
}

/// Les demandes reçues : la table de toutes, puis celles à traiter avec leurs actions.
fn received_section(
    products: &portaki_sdk::resource::Collection<Product, crate::Requests>,
    reports: &[ConsumableReport],
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let enabled = products.meta().enabled();
    let mut children: Vec<Component> = Vec::new();
    if !enabled {
        let notice = InlineNotice::new().message("i18n:host.received.disabled");
        children.push(notice.into());
    }
    let recent: Vec<&ConsumableReport> = reports.iter().take(40).collect();
    if recent.is_empty() && enabled {
        let empty = EmptyState::new()
            .title("i18n:host.received.empty")
            .description("i18n:host.received.empty.help")
            .icon(IconName::Package);
        children.push(empty.into());
    }
    if !recent.is_empty() {
        children.push(received_table(products.rows(), &recent, now, locale));
        let pending: Vec<Component> = recent
            .iter()
            .filter(|report| status::is_pending(&report.status))
            .map(|report| build_report_block(report, now, locale))
            .collect();
        if !pending.is_empty() {
            let title = Text::new()
                .text("i18n:host.received.toHandle")
                .variant(TextVariant::Caption);
            children.push(title.into());
            children.push(List::new().children(pending).into());
        }
    }
    Section::new()
        .title("i18n:host.received.title")
        .children(children)
        .into()
}

fn received_table(
    products: &[Product],
    recent: &[&ConsumableReport],
    now: DateTime<Utc>,
    locale: &str,
) -> Component {
    let emoji_of = |id: Uuid| {
        products
            .iter()
            .find(|p| p.uuid() == id)
            .map(|p| suggest::shown(&p.emoji, &p.name(locale)))
            .unwrap_or_default()
    };
    let rows = recent
        .iter()
        .map(|report| {
            let guest = match report.guest_name.trim() {
                "" => "—",
                name => name,
            };
            json!({
                "id": report.id,
                "date": format_relative_when(report.created_at, now, locale),
                "stay": stay_dates(report, locale),
                "guest": guest,
                "product": format!("{} {}", emoji_of(report.item_id), report.item_label).trim().to_string(),
                "status": format!("i18n:{}", status::status_label_key(&report.status)),
            })
        })
        .collect();
    DataTable::new()
        .columns(json!([
            { "key": "date", "label": "i18n:host.column.date" },
            { "key": "stay", "label": "i18n:host.column.stay" },
            { "key": "guest", "label": "i18n:host.column.guest" },
            { "key": "product", "label": "i18n:host.column.product" },
            { "key": "status", "label": "i18n:host.column.status", "kind": "pill" },
        ]))
        .rows(Value::Array(rows))
        .into()
}

/// « 12 oct. – 15 oct. », ou « — » pour une demande d'avant la colonne.
fn stay_dates(report: &ConsumableReport, locale: &str) -> String {
    match (report.stay_check_in, report.stay_check_out) {
        (Some(from), Some(to)) => {
            format!("{} – {}", short_date(from, locale), short_date(to, locale))
        }
        (Some(from), None) => short_date(from, locale),
        _ => "—".to_string(),
    }
}
