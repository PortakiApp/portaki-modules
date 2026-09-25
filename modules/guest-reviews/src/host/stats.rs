//! Property statistics « Avis » — the tile (`statsSummary`) and the `property-stats-detail`
//! surface (`reviews`), from the ratings left in the booklet.
//!
//! Those reviews go to the host only: all of them are private. Whether a guest then followed the
//! Airbnb link is not known here. The platform passes the period's stays to the detail page
//! (`input.stays`): the response rate is the reviews of the period over its departures.

use portaki_sdk::contracts::stats::{self, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};

use crate::commands::{load_reviews, StoredReview};
use crate::i18n;

/// Words that bring a theme up, French and English, matched in lower case.
const THEMES: &[(&str, &[&str])] = &[
    (
        "location",
        &[
            "emplacement",
            "situé",
            "placé",
            "quartier",
            "location",
            "located",
            "area",
        ],
    ),
    ("cleanliness", &["propre", "propreté", "clean"]),
    ("welcome", &["accueil", "hôte", "welcome", "host"]),
    ("noise", &["bruit", "bruyant", "noise", "noisy", "loud"]),
];

/// Reviews of the last `days`; a review stored before dates were kept counts in every period.
fn reviews_since(days: i64) -> Result<Vec<StoredReview>> {
    let now = time::now()?.timestamp();
    Ok(load_reviews()?
        .into_iter()
        .filter(|r| r.at.is_none_or(|at| now - at.timestamp() <= days * 86_400))
        .collect())
}

/// Departures of the last `days` among the stays the platform passed; `None` when it passed none.
fn departures(ctx: &HostContext, days: i64) -> Option<usize> {
    #[derive(serde::Deserialize)]
    struct Stay {
        #[serde(rename = "checkOut")]
        check_out: DateTime<Utc>,
    }
    let stays: Vec<Stay> = serde_json::from_value(ctx.input.get("stays")?.clone()).ok()?;
    let now = time::now().ok()?.timestamp();
    Some(
        stays
            .iter()
            .filter(|stay| (0..days * 86_400).contains(&(now - stay.check_out.timestamp())))
            .count(),
    )
}

fn period_key(days: i64) -> i64 {
    match days {
        90 | 365 => days,
        _ => 30,
    }
}

/// « 4,8 » (French) or « 4.8 ».
fn average(reviews: &[StoredReview], fr: bool) -> Option<String> {
    if reviews.is_empty() {
        return None;
    }
    let sum: u32 = reviews.iter().map(|r| u32::from(r.rating)).sum();
    let text = format!("{:.1}", f64::from(sum) / reviews.len() as f64);
    Some(if fr { text.replace('.', ",") } else { text })
}

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Avis sur l'année",
        input = r#"{"propertyId":"8c0e6f2a-1d3b-4a7e-b5c9-0f4e2d1a6b38","period":365,"key":"reviews"}"#
    )
)]
pub fn stats_summary(ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let days = period_key(i64::from(args.period));
    let reviews = reviews_since(days)?;
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let value = average(&reviews, fr).map_or_else(|| "—".to_string(), |avg| format!("{avg} ★"));
    let count = reviews.len().to_string();
    Ok(stats::summary(
        value,
        i18n::text(&format!("stats.tile.{days}"), &[("count", &count)]),
    ))
}

#[portaki_sdk::surface(
    host,
    id = "reviews",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.reviews",
    icon = IconName::Star
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let days = period_key(ctx.input_u64("periodDays").unwrap_or(30) as i64);
    let mut reviews = reviews_since(days).unwrap_or_default();
    reviews.reverse();

    if reviews.is_empty() {
        return Surface::new(
            Page::new().child(
                EmptyState::new()
                    .title("i18n:stats.empty")
                    .description("i18n:stats.empty.help")
                    .icon(IconName::Star),
            ),
        )
        .with_id(crate::ids::HOST_STATS);
    }

    let count = reviews.len();
    let mut tiles = vec![Stat::new()
        .label("i18n:stats.average")
        .icon(IconName::Star)
        .value(format!("{} / 5", average(&reviews, fr).unwrap_or_default()))
        .delta(t!(&format!("stats.average.note.{days}"), count = count).unwrap_or_default())
        .into()];
    if let Some(departed) = departures(&ctx, days).filter(|n| *n > 0) {
        // ponytail: un avis laissé au début de la période peut venir d'un départ d'avant ; plafonné à 100 %.
        let rate = (count * 100 / departed).min(100);
        tiles.push(
            Stat::new()
                .label("i18n:stats.responseRate")
                .icon(IconName::Message)
                .value(format!("{rate} %"))
                .delta(t!("stats.responseRate.note", count = departed).unwrap_or_default())
                .into(),
        );
    }
    tiles.push(
        Stat::new()
            .label("i18n:stats.private")
            .icon(IconName::Lock)
            .value(count.to_string())
            .delta("i18n:stats.private.note")
            .into(),
    );
    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(tiles);

    let stars = |rating: u8| reviews.iter().filter(|r| r.rating == rating).count();
    let low = reviews.iter().filter(|r| r.rating <= 2).count();
    let unit = |n: usize| t!("stats.unit.reviews", count = n).unwrap_or_else(|_| n.to_string());
    let ratings = Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(Swatch::Orange)
        .points(vec![
            ChartPoint::new("i18n:stats.ratings.5", stars(5) as f64).display(unit(stars(5))),
            ChartPoint::new("i18n:stats.ratings.4", stars(4) as f64).display(unit(stars(4))),
            ChartPoint::new("i18n:stats.ratings.3", stars(3) as f64).display(unit(stars(3))),
            ChartPoint::new("i18n:stats.ratings.low", low as f64).display(unit(low)),
        ])
        .into();

    let mut themes: Vec<(&str, usize)> = THEMES
        .iter()
        .map(|(theme, words)| {
            let n = reviews
                .iter()
                .filter(|r| {
                    let text = r.comment.to_lowercase();
                    words.iter().any(|word| text.contains(word))
                })
                .count();
            (*theme, n)
        })
        .filter(|(_, n)| *n > 0)
        .collect();
    themes.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let themes: Component = if themes.is_empty() {
        EmptyState::new()
            .title("i18n:stats.empty")
            .description("i18n:stats.themes.empty")
            .icon(IconName::Message)
            .into()
    } else {
        Chart::new()
            .kind(ChartKind::HorizontalBars)
            .swatch(Swatch::Black)
            .points(
                themes
                    .into_iter()
                    .map(|(theme, n)| {
                        ChartPoint::new(format!("i18n:stats.themes.{theme}"), n as f64).display(
                            t!("stats.unit.mentions", count = n).unwrap_or_else(|_| n.to_string()),
                        )
                    })
                    .collect(),
            )
            .into()
    };

    let rows = reviews
        .iter()
        .take(10)
        .map(|review| {
            let stars = t!("stats.row.stars", count = review.rating).unwrap_or_default();
            let meta = match &review.guest_name {
                Some(name) => format!("{name} · {stars}"),
                None => stars,
            };
            let mut row = FeedItem::new()
                .title(if review.comment.is_empty() {
                    "i18n:stats.row.noComment".to_string()
                } else {
                    format!("« {} »", review.comment)
                })
                .tag(format!("{} ★", review.rating))
                .meta(meta)
                .dotTone(Tone::Warning)
                .status(FeedStatus::new("i18n:stats.status.private", Tone::Warning));
            if let Some(at) = review.at {
                row = row.date(at.format("%d/%m/%Y").to_string());
            }
            row.into()
        })
        .collect();

    let panel = |key: &str, body: Component| -> Component {
        Card::new()
            .title(format!("i18n:stats.{key}.title"))
            .subtitle(format!("i18n:stats.{key}.subtitle"))
            .child(body)
            .into()
    };
    Surface::new(Page::new().child(Stack::new().gap(16.0).children(vec![
        tiles.into(),
        Grid::new()
            .minColumnWidth(320.0)
            .gap(16.0)
            .children(vec![panel("ratings", ratings), panel("themes", themes)])
            .into(),
        panel("feed", Stack::new().gap(0.0).children(rows).into()),
    ])))
    .with_id(crate::ids::HOST_STATS)
}
