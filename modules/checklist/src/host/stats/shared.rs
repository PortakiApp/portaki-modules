//! Ce que les deux détails partagent : taux par étape, graphiques, périodes.

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

/// `(item, how many stays or tasks ticked it)`, least ticked last.
pub(crate) fn item_rates(items: &[Step], ticked: impl Fn(&Step) -> usize) -> Vec<(&Step, usize)> {
    let mut rates: Vec<(&Step, usize)> = items.iter().map(|item| (item, ticked(item))).collect();
    rates.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    rates
}

pub(crate) fn rate_chart(rates: &[(&Step, usize)], of: usize, swatch: Swatch) -> Component {
    Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(swatch)
        .max(100.0)
        .points(
            rates
                .iter()
                .map(|(item, n)| {
                    let rate = (n * 100).checked_div(of).unwrap_or(0);
                    ChartPoint::new(item.label.fr.clone(), rate as f64).display(format!("{rate} %"))
                })
                .collect(),
        )
        .into()
}

/// « La plus oubliée » — only once something was missed.
pub(crate) fn most_forgotten(
    rates: &[(&Step, usize)],
    of: usize,
    fr: bool,
    prefix: &str,
) -> Option<Component> {
    let (item, ticked) = rates.last().filter(|(_, n)| *n < of)?;
    let label = item.label.clone();
    Some(
        Stat::new()
            .label(format!("i18n:{prefix}.forgotten"))
            .icon(IconName::DangerTriangle)
            .value(if fr { label.fr } else { label.en })
            .delta(
                t!(
                    &format!("{prefix}.forgotten.note"),
                    missed = of - ticked,
                    count = of
                )
                .unwrap_or_default(),
            )
            .into(),
    )
}

pub(crate) fn panels(left: (&str, Component), right: (&str, Component)) -> Component {
    let card = |(key, body): (&str, Component)| -> Component {
        Card::new()
            .title(format!("i18n:{key}.title"))
            .subtitle(format!("i18n:{key}.subtitle"))
            .child(body)
            .into()
    };
    Grid::new()
        .minColumnWidth(320.0)
        .gap(16.0)
        .children(vec![card(left), card(right)])
        .into()
}

pub(crate) fn feed_card(key: &str, rows: Vec<Component>) -> Component {
    Card::new()
        .title(format!("i18n:{key}.title"))
        .subtitle(format!("i18n:{key}.subtitle"))
        .child(Stack::new().gap(0.0).children(rows))
        .into()
}

pub(crate) fn empty(description: &str, icon: IconName) -> Component {
    EmptyState::new()
        .title("i18n:stats.empty")
        .description(description)
        .icon(icon)
        .into()
}

pub(crate) fn average(durations: &[Duration]) -> Option<Duration> {
    let n = i32::try_from(durations.len()).ok().filter(|n| *n > 0)?;
    Some(durations.iter().copied().sum::<Duration>() / n)
}

/// « 40 min », « 2 h 10 ».
pub(crate) fn format_duration(duration: Duration) -> String {
    let minutes = duration.num_minutes().max(0);
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m:02}"),
    }
}

pub(crate) fn decimal(value: f64, fr: bool) -> String {
    let text = format!("{value:.1}");
    if fr {
        text.replace('.', ",")
    } else {
        text
    }
}

/// `at` on the property's clock — what the host reads; UTC only when its zone is unknown.
pub(crate) fn local(at: DateTime<Utc>, tz: Option<PropertyTz>) -> NaiveDateTime {
    tz.map_or(at.naive_utc(), |tz| tz.to_local(at).naive_local())
}

pub(crate) fn format_time(at: NaiveDateTime) -> String {
    at.format("%H:%M").to_string()
}

pub(crate) fn short_date(at: NaiveDateTime, fr: bool) -> String {
    use chrono::Datelike;
    const FR: [&str; 12] = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ];
    const EN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = at.month0() as usize;
    if fr {
        format!("{} {}", at.day(), FR[month])
    } else {
        format!("{} {}", EN[month], at.day())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_on_the_property_clock() {
        let at = "2026-08-09T22:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let paris = PropertyTz::parse("Europe/Paris");
        // 22:30 UTC in August is 00:30 the next day in Paris (CEST).
        assert_eq!(format_time(local(at, paris)), "00:30");
        assert_eq!(short_date(local(at, paris), true), "10 août");
        let winter = "2026-01-09T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(format_time(local(winter, paris)), "13:00");
        assert_eq!(
            format_time(local(at, PropertyTz::parse("Mars/Olympus"))),
            "22:30"
        );
    }
}
