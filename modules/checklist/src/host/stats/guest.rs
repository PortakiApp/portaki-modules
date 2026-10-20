//! Les statistiques des listes du livret : ce que les voyageurs ont coché.

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

/// Guest ticks of one stay over the guest items that still exist.
pub(super) struct GuestStay {
    ticked: Vec<Uuid>,
    total: usize,
    last_at: DateTime<Utc>,
    /// `(check-in, check-out)` when the platform passed the stay.
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl GuestStay {
    fn complete(&self) -> bool {
        self.ticked.len() >= self.total
    }
}

pub(super) fn complete_count(stays: &[GuestStay]) -> usize {
    stays.iter().filter(|stay| stay.complete()).count()
}

/// Guest ticks per stay since `since`.
pub(super) fn ticks_by_stay(since: DateTime<Utc>) -> Result<BTreeMap<Uuid, GuestStay>> {
    let items = crate::queries::guest_items()?;
    let mut by_stay: BTreeMap<Uuid, GuestStay> = BTreeMap::new();
    for row in storage::list_completions(None)? {
        if row.completed_at < since || !items.iter().any(|item| item.uuid() == row.item_id) {
            continue;
        }
        let stay = by_stay.entry(row.stay_id).or_insert(GuestStay {
            ticked: Vec::new(),
            total: items.len(),
            last_at: row.completed_at,
            window: None,
        });
        stay.ticked.push(row.item_id);
        stay.last_at = stay.last_at.max(row.completed_at);
    }
    Ok(by_stay)
}

/// Stays that ticked something since `since`, newest first.
pub(super) fn guest_stays(since: DateTime<Utc>) -> Result<Vec<GuestStay>> {
    let mut stays: Vec<GuestStay> = ticks_by_stay(since)?.into_values().collect();
    stays.sort_by_key(|stay| std::cmp::Reverse(stay.last_at));
    Ok(stays)
}

/// Every departure of `(since, now]`, newest first, with what its guest ticked — none for a list
/// never opened. Ticks are read from a week before the period: a guest ticks during the stay.
pub(super) fn departures(
    stays: &[TimelineStay],
    mut ticks: BTreeMap<Uuid, GuestStay>,
    total: usize,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<GuestStay> {
    let mut out: Vec<GuestStay> = stays
        .iter()
        .filter(|stay| stay.check_out > since && stay.check_out <= now)
        .map(|stay| {
            let mut guest = ticks.remove(&stay.id).unwrap_or(GuestStay {
                ticked: Vec::new(),
                total,
                last_at: stay.check_out,
                window: None,
            });
            guest.window = Some((stay.check_in, stay.check_out));
            guest
        })
        .collect();
    out.sort_by_key(|stay| std::cmp::Reverse(stay.window.map(|(_, out)| out)));
    out
}

#[portaki_sdk::surface(
    host,
    id = "checklist",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.checklist",
    icon = IconName::CheckCircle
)]
pub fn render_stats_checklist(ctx: HostContext) -> Surface {
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let since = now - Duration::days(period_days(&ctx));
    let fr = is_fr(&ctx.locale);
    let items = crate::queries::guest_items().unwrap_or_default();
    let stays = match period_stays(&ctx) {
        Some(period) => departures(
            &period,
            ticks_by_stay(since - Duration::days(30)).unwrap_or_default(),
            items.len(),
            since,
            now,
        ),
        None => guest_stays(since).unwrap_or_default(),
    };

    let body: Component = if stays.is_empty() {
        empty("i18n:stats.checklist.empty", IconName::CheckCircle)
    } else {
        let rates = item_rates(&items, |item| {
            stays
                .iter()
                .filter(|stay| stay.ticked.contains(&item.uuid()))
                .count()
        });
        let tiles = checklist_tiles(&stays, items.len(), &rates, fr);

        let recent: Vec<&GuestStay> = stays.iter().take(8).collect();
        let per_stay =
            recent
                .iter()
                .rev()
                .enumerate()
                .map(|(index, stay)| {
                    ChartPoint::new((index + 1).to_string(), stay.ticked.len() as f64)
                        .display(format!("{}/{}", stay.ticked.len(), stay.total))
                })
                .collect();

        let feed = stays
            .iter()
            .take(10)
            .map(|stay| stay_row(stay, fr))
            .collect();

        Stack::new()
            .gap(16.0)
            .children(vec![
                Grid::new()
                    .minColumnWidth(170.0)
                    .gap(12.0)
                    .children(tiles)
                    .into(),
                panels(
                    (
                        "stats.checklist.byItem",
                        rate_chart(&rates, stays.len(), Swatch::Green),
                    ),
                    (
                        "stats.checklist.perStay",
                        Chart::new()
                            .kind(ChartKind::Bars)
                            .swatch(Swatch::Green)
                            .points(per_stay)
                            .into(),
                    ),
                ),
                feed_card("stats.checklist.feed", feed),
            ])
            .into()
    };
    Surface::new(Page::new().child(body)).with_id(CHECKLIST)
}

/// One stay in the feed: its dates, how much was ticked, and its status.
pub(super) fn stay_row(stay: &GuestStay, fr: bool) -> Component {
    let (label, tone) = if stay.complete() {
        ("i18n:stats.checklist.status.complete", Tone::Success)
    } else if stay.ticked.is_empty() {
        ("i18n:stats.checklist.status.unfilled", Tone::Neutral)
    } else {
        ("i18n:stats.checklist.status.incomplete", Tone::Warning)
    };
    let title = match stay.window {
        Some((check_in, check_out)) => t!(
            "stats.checklist.row.stay",
            from = short_date(check_in, fr),
            to = short_date(check_out, fr)
        ),
        None => t!(
            "stats.checklist.row.title",
            date = short_date(stay.last_at, fr)
        ),
    }
    .unwrap_or_default();
    let date = match stay.window {
        Some((_, check_out)) => t!(
            "stats.checklist.row.departure",
            date = short_date(check_out, fr)
        )
        .unwrap_or_default(),
        None => format_time(stay.last_at),
    };
    FeedItem::new()
        .title(title)
        .meta(
            t!(
                "stats.checklist.row.meta",
                done = stay.ticked.len(),
                total = stay.total
            )
            .unwrap_or_default(),
        )
        .date(date)
        .dotTone(tone)
        .status(FeedStatus::new(label, tone))
        .into()
}

/// Finished, ticked on average, most forgotten.
pub(super) fn checklist_tiles(
    stays: &[GuestStay],
    total: usize,
    rates: &[(&Step, usize)],
    fr: bool,
) -> Vec<Component> {
    let ticked: usize = stays.iter().map(|stay| stay.ticked.len()).sum();
    let average = ticked as f64 / stays.len() as f64;
    let mut tiles = vec![
        Stat::new()
            .label("i18n:stats.checklist.completed")
            .icon(IconName::CheckCircle)
            .value(percent(complete_count(stays), stays.len()))
            .delta(t!("stats.checklist.completed.note", count = stays.len()).unwrap_or_default())
            .into(),
        Stat::new()
            .label("i18n:stats.checklist.ticked")
            .icon(IconName::Clipboard)
            .value(format!("{} / {}", decimal(average, fr), total))
            .delta("i18n:stats.checklist.ticked.note")
            .into(),
    ];
    tiles.extend(most_forgotten(rates, stays.len(), fr, "stats.checklist"));
    tiles
}
