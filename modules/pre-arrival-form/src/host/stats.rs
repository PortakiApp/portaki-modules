//! Tuile de statistiques (spec Pré-arrivée §1) : « Formulaires remplis » sur la période.
//!
//! Un nombre, pas le pourcentage de la spec : `statsSummary` ne reçoit que la période, pas les
//! séjours (la plateforme ne les joint qu'à une page `property-stats-detail`, que la spec écarte),
//! et le module ne connaît un séjour qu'une fois son formulaire rempli. Le pourcentage viendra
//! quand la plateforme passera les séjours de la période à `statsSummary`.

use portaki_sdk::contracts::stats::{self, Period, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Page, Stat};
use portaki_sdk::sdui::surface::Surface;

use crate::{i18n, storage};

/// Formulaires remplis sur la période.
fn filled(period: Period) -> Result<usize> {
    Ok(storage::list_since(period.since(time::now()?))?.len())
}

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Formulaires sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"forms"}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    Ok(stats::summary(
        filled(args.period())?.to_string(),
        i18n::text("stats.tile"),
    ))
}

#[portaki_sdk::surface(
    host,
    id = "forms",
    placement = HostPlacement::PropertyStatsCard,
    label_key = "catalog.host.forms",
    icon = IconName::CheckCircle
)]
pub fn render_host_stats(ctx: HostContext) -> Result<Surface> {
    let period = ctx.stats_period();
    let count = filled(period)?;
    let body: Component = if count == 0 {
        EmptyState::new()
            .title("i18n:stats.empty")
            .description("i18n:stats.empty.help")
            .icon(IconName::CheckCircle)
            .into()
    } else {
        Stat::new()
            .label("i18n:catalog.host.forms")
            .icon(IconName::CheckCircle)
            .value(count.to_string())
            .into()
    };
    Ok(Surface::new(Page::new().child(body)).with_id(FORMS))
}
