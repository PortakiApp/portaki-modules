//! Encart du séjour (spec Calendriers §1) : « Importé d'Airbnb · mis à jour il y a 12 min » ; si
//! incomplet, « Sans e-mail voyageur ».
//!
//! La plateforme joint `input.stay` (id, arrivée, départ) mais pas l'UID iCal : le séjour est
//! retrouvé dans le dernier relevé par ses dates.

use chrono::{DateTime, NaiveDate, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use super::stats::parse;
use crate::sync_state::{load_sync_state, SeenStay, SyncState};

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Calendar
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let state = load_sync_state().unwrap_or_default();
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let lines: Vec<Component> = lines(&state, window(&ctx), now, &ctx.locale)?
        .into_iter()
        .map(|line| Text::new().text(line).variant(TextVariant::Body).into())
        .collect();
    let card = Card::new()
        .icon(IconName::Calendar)
        .title("i18n:host.stay.title")
        .children(lines);
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}

fn lines(
    state: &SyncState,
    window: Option<(NaiveDate, NaiveDate)>,
    now: DateTime<Utc>,
    lang: &str,
) -> Result<Vec<String>> {
    let Some((check_in, check_out)) = window else {
        return Ok(vec!["i18n:host.stay.notImported".into()]);
    };
    // ponytail: rapprochement par dates d'arrivée et de départ (UTC) ; l'UID iCal dans
    // `input.stay` le rendrait exact.
    let matches: Vec<&SeenStay> = state
        .uids
        .values()
        .filter(|s| {
            day(&s.check_in_at) == Some(check_in) && day(&s.check_out_at) == Some(check_out)
        })
        .collect();
    let stay = match matches.as_slice() {
        [] => return Ok(vec!["i18n:host.stay.notImported".into()]),
        [stay] => stay,
        // Deux calendriers sur les mêmes dates : c'est le conflit, pas une source.
        _ => return Ok(vec!["i18n:host.stay.ambiguous".into()]),
    };
    let ago = state
        .last_success_at
        .as_deref()
        .and_then(parse)
        .map(|at| time::ago(at, now, lang));
    // « d'Airbnb », « de Booking.com » : l'élision vit dans la traduction.
    let from = t!(&format!("host.stay.from.{}", stay.channel.as_str()))?;
    let first = match ago {
        Some(ago) => t!("host.stay.imported", from = &from, ago = &ago)?,
        None => t!("host.stay.imported.plain", from = &from)?,
    };
    let mut lines = vec![first];
    if !stay.has_guest_email {
        lines.push("i18n:host.stay.noEmail".into());
    }
    Ok(lines)
}

fn day(raw: &str) -> Option<NaiveDate> {
    parse(raw).map(|at| at.date_naive())
}

/// `input.stay` : la fenêtre que la plateforme joint à une surface de fiche séjour.
fn window(ctx: &HostContext) -> Option<(NaiveDate, NaiveDate)> {
    let stay = ctx.input.get("stay")?;
    let date = |key: &str| stay.get(key).and_then(|v| v.as_str()).and_then(day);
    Some((date("checkIn")?, date("checkOut")?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use portaki_sdk::contracts::booking_channel::BookingChannel;

    use super::*;

    fn seen(check_in: &str, check_out: &str, channel: BookingChannel, email: bool) -> SeenStay {
        SeenStay {
            check_in_at: check_in.into(),
            check_out_at: check_out.into(),
            channel,
            has_guest_email: email,
            ..SeenStay::default()
        }
    }

    fn window(check_in: &str, check_out: &str) -> Option<(NaiveDate, NaiveDate)> {
        Some((day(check_in)?, day(check_out)?))
    }

    fn state(stays: Vec<SeenStay>) -> SyncState {
        SyncState {
            uids: stays
                .into_iter()
                .enumerate()
                .map(|(i, s)| (format!("uid-{i}"), s))
                .collect::<BTreeMap<_, _>>(),
            last_success_at: Some("2026-08-20T09:48:00Z".into()),
            ..SyncState::default()
        }
    }

    fn now() -> DateTime<Utc> {
        parse("2026-08-20T10:00:00Z").unwrap()
    }

    #[test]
    #[serial_test::serial]
    fn an_imported_stay_names_its_platform_and_a_missing_email() {
        portaki_test_utils::MockContext::host().run(|_| {
            let state = state(vec![seen(
                "2026-08-21T00:00:00+00:00",
                "2026-08-24T00:00:00+00:00",
                BookingChannel::Airbnb,
                false,
            )]);
            // La plateforme tronque à la seconde, en `Z`.
            let found = window("2026-08-21T00:00:00Z", "2026-08-24T00:00:00Z");
            let lines = lines(&state, found, now(), "fr").unwrap();
            assert_eq!(lines.len(), 2);
            assert!(lines[0].contains("host.stay.imported"), "{}", lines[0]);
            assert_eq!(lines[1], "i18n:host.stay.noEmail");
        });
    }

    #[test]
    fn a_stay_not_in_the_feeds_says_so() {
        let state = state(vec![]);
        let lines = lines(
            &state,
            window("2026-08-21T00:00:00Z", "2026-08-24T00:00:00Z"),
            now(),
            "fr",
        )
        .unwrap();
        assert_eq!(lines, ["i18n:host.stay.notImported"]);
        assert_eq!(
            super::lines(&state, None, now(), "fr").unwrap(),
            ["i18n:host.stay.notImported"]
        );
    }

    #[test]
    fn two_feeds_on_the_same_dates_name_no_source() {
        let state = state(vec![
            seen(
                "2026-08-21T00:00:00Z",
                "2026-08-24T00:00:00Z",
                BookingChannel::Airbnb,
                true,
            ),
            seen(
                "2026-08-21T00:00:00Z",
                "2026-08-24T00:00:00Z",
                BookingChannel::Booking,
                true,
            ),
        ]);
        let lines = lines(
            &state,
            window("2026-08-21T00:00:00Z", "2026-08-24T00:00:00Z"),
            now(),
            "fr",
        )
        .unwrap();
        assert_eq!(lines, ["i18n:host.stay.ambiguous"]);
    }
}
