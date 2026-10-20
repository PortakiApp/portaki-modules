//! Statistiques du logement : « Bons plans consultés » sur la période — adresses et activités
//! ouvertes, lues dans les consultations du livret que la plateforme joint (`views`).
//!
//! La planche Bons plans n'a pas de statistiques : la tuile suit celle des Événements, le détail
//! sépare adresses et activités et compte les sorties du livret (itinéraire, appel, site). Jamais
//! par adresse : la plateforme compte par surface du manifeste. Sans statistiques du livret
//! (`views` absent), la tuile affiche « — » et le détail le dit.

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::stats::{self, StatsSummary, StatsSummaryArgs, StatsViews};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Grid, Page, Stat};
use portaki_sdk::sdui::surface::Surface;

/// La fiche d'une adresse.
const SPOT: &str = "explore.item";
/// La fiche d'une activité : la vôtre, ou un lien Viator / Tiqets mis en avant.
const ACTIVITIES: [&str; 2] = ["explore.activity", "explore.link"];
/// La liste des bons plans.
const LIST: &str = "explore.detail";
/// Les sorties du livret, dans l'ordre du détail.
const CLICKS: [&str; 4] = ["directions", "call", "link", "mail"];

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Bons plans sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"guide-views","views":{"opens":9,"surfaces":{"explore.item":5,"explore.activity":2,"explore.detail":2},"clicks":{"directions":3}}}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let value = args
        .views
        .map_or_else(|| "—".to_owned(), |views| viewed(&views).to_string());
    Ok(stats::summary(value, bundle_text!("stats.tile")))
}

#[portaki_sdk::surface(
    host,
    id = "guide-views",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.guide-views",
    icon = IconName::MapPin
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let body = match ctx.stats_views() {
        None => empty("i18n:stats.views.unavailable"),
        Some(views) if views.opens == 0 && views.total_clicks() == 0 => {
            empty("i18n:stats.views.empty")
        }
        Some(views) => tiles(&views, ctx.stats_period().window(&ctx.locale)),
    };
    Surface::new(Page::new().child(body)).with_id(GUIDE_VIEWS)
}

/// Adresses et activités ouvertes.
fn viewed(views: &StatsViews) -> u64 {
    views.surface(SPOT) + activities(views)
}

fn activities(views: &StatsViews) -> u64 {
    ACTIVITIES.iter().map(|id| views.surface(id)).sum()
}

fn empty(title: &str) -> Component {
    EmptyState::new().title(title).icon(IconName::MapPin).into()
}

fn tiles(views: &StatsViews, window: String) -> Component {
    let stat = |label: String, value: u64| -> Component {
        Stat::new()
            .label(label)
            .value(value.to_string())
            .delta(window.clone())
            .into()
    };
    let mut children = vec![
        stat("i18n:catalog.host.guide-views".into(), viewed(views)),
        stat("i18n:stats.views.spots".into(), views.surface(SPOT)),
        stat("i18n:stats.views.activities".into(), activities(views)),
        stat("i18n:stats.views.list".into(), views.surface(LIST)),
    ];
    children.extend(
        CLICKS
            .iter()
            .filter(|kind| views.clicks_of(kind) > 0)
            .map(|kind| {
                stat(
                    format!("i18n:stats.views.clicks.{kind}"),
                    views.clicks_of(kind),
                )
            }),
    );
    Grid::new()
        .minColumnWidth(170.0)
        .gap(12.0)
        .children(children)
        .into()
}
