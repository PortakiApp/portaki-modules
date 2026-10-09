//! Stay detail encart (spec Checklist §1) : « 8 / 12 étapes · non terminée » ou « Terminée le
//! 29/08 à 09:12 », d'après ce que le voyageur de ce séjour a coché.

use chrono::{DateTime, Datelike, Timelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::guest::depart::offset_for_iana;
use crate::storage;

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::ListChecks
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let line = match ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok())
    {
        None => "i18n:host.stay.missingStay".to_string(),
        Some(stay_id) => {
            let items = crate::queries::guest_items()?;
            let ticks: Vec<DateTime<Utc>> = storage::list_completions(Some(stay_id))?
                .into_iter()
                .filter(|row| items.iter().any(|item| item.id == row.item_id))
                .map(|row| row.completed_at)
                .collect();
            line(items.len(), &ticks, &ctx.property.timezone)
        }
    };
    let card = Card::new()
        .icon(IconName::ListChecks)
        .title("i18n:host.stay.title")
        .child(Text::new().text(line).variant(TextVariant::Body));
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}

/// `ticks`: when each ticked guest step was ticked.
fn line(total: usize, ticks: &[DateTime<Utc>], timezone: &str) -> String {
    if total == 0 {
        return "i18n:host.stay.noList".to_string();
    }
    let done = ticks.len();
    match ticks.iter().max() {
        Some(at) if done >= total => {
            let local = at.with_timezone(&offset_for_iana(timezone, *at));
            let (day, month) = (
                format!("{:02}", local.day()),
                format!("{:02}", local.month()),
            );
            let (hour, minute) = (
                format!("{:02}", local.hour()),
                format!("{:02}", local.minute()),
            );
            match t!(
                "host.stay.done",
                day = &day,
                month = &month,
                hour = &hour,
                minute = &minute
            ) {
                Ok(text) if text.contains(&day) => text,
                _ => format!("Terminée le {day}/{month} à {hour}:{minute}"),
            }
        }
        _ => {
            let (done, total) = (done.to_string(), total.to_string());
            match t!("host.stay.progress", done = &done, total = &total) {
                Ok(text) if text.contains(&total) => text,
                _ => format!("{done} / {total} étapes · non terminée"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn progress_then_completion_in_the_property_time() {
        let ticks = [at("2026-08-29T06:00:00Z"), at("2026-08-29T07:12:00Z")];
        assert_eq!(
            line(3, &ticks, "Europe/Paris"),
            "2 / 3 étapes · non terminée"
        );
        assert_eq!(line(2, &ticks, "Europe/Paris"), "Terminée le 29/08 à 09:12");
        assert_eq!(line(0, &[], "Europe/Paris"), "i18n:host.stay.noList");
    }
}
