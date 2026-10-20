//! Statistiques du logement : « Itinéraires consultés » sur la période, lus dans les consultations
//! du livret que la plateforme joint (`views`).
//!
//! La planche Randonnées n'a pas de statistiques : la tuile suit celle des Événements. Jamais par
//! itinéraire : la plateforme compte par surface du manifeste (`explore.item`). Sans statistiques
//! du livret (`views` absent), la tuile affiche « — » et le détail le dit.

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::stats::{self, StatsSummary, StatsSummaryArgs, StatsViews};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Grid, Page, Stat};
use portaki_sdk::sdui::surface::Surface;

/// La fiche d'un itinéraire : ce que la tuile compte.
const ITEM: &str = "explore.item";
/// La liste des randonnées.
const LIST: &str = "explore.detail";
/// Les sorties du livret, dans l'ordre du détail.
const CLICKS: [&str; 4] = ["directions", "link", "call", "mail"];

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Itinéraires sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"trails-views","views":{"opens":9,"surfaces":{"explore.item":7,"explore.detail":2},"clicks":{"link":3}}}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let value = args
        .views
        .map_or_else(|| "—".to_owned(), |views| views.surface(ITEM).to_string());
    Ok(stats::summary(value, bundle_text!("stats.tile")))
}

#[portaki_sdk::surface(
    host,
    id = "trails-views",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.trails-views",
    icon = IconName::Mountain
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let body = match ctx.stats_views() {
        None => empty("i18n:stats.views.unavailable"),
        Some(views) if views.opens == 0 && views.total_clicks() == 0 => {
            empty("i18n:stats.views.empty")
        }
        Some(views) => tiles(&views, ctx.stats_period().window(&ctx.locale)),
    };
    Surface::new(Page::new().child(body)).with_id(TRAILS_VIEWS)
}

fn empty(title: &str) -> Component {
    EmptyState::new()
        .title(title)
        .icon(IconName::Mountain)
        .into()
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
        stat("i18n:catalog.host.trails-views".into(), views.surface(ITEM)),
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
