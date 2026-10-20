//! Statistiques du logement (planche Appareils « Stats ») : « Modes d'emploi ouverts » sur la
//! période, lus dans les consultations du livret que la plateforme joint (`views`).
//!
//! Ni l'appareil en tête ni le tableau par appareil de la planche : la plateforme compte les
//! ouvertures par surface du manifeste (`explore.item`), jamais par appareil. Pas de « signalements
//! liés » non plus, qui dépendraient d'un lien Signaler → appareil. Sans statistiques du livret
//! (`views` absent), la tuile affiche « — » et le détail le dit.

use portaki_sdk::contracts::stats::StatsViews;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Grid, Page, Stat};
use portaki_sdk::sdui::surface::Surface;

/// La fiche d'un appareil : ce que la tuile compte.
pub(crate) const ITEM: &str = "explore.item";
/// La liste des appareils.
const LIST: &str = "explore.detail";
/// Les sorties du livret, dans l'ordre du détail.
const CLICKS: [&str; 4] = ["link", "call", "directions", "mail"];

#[portaki_sdk::surface(
    host,
    id = "appliances-views",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.appliances-views",
    icon = IconName::Plug
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let body = match ctx.stats_views() {
        None => empty("i18n:stats.views.unavailable"),
        Some(views) if views.opens == 0 && views.total_clicks() == 0 => {
            empty("i18n:stats.views.empty")
        }
        Some(views) => tiles(&views, ctx.stats_period().window(&ctx.locale)),
    };
    Surface::new(Page::new().child(body)).with_id(APPLIANCES_VIEWS)
}

fn empty(title: &str) -> Component {
    EmptyState::new().title(title).icon(IconName::Plug).into()
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
        stat(
            "i18n:catalog.host.appliances-views".into(),
            views.surface(ITEM),
        ),
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
