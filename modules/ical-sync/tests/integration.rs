//! Integration-style unit tests with `portaki-test-utils`.

use ical_sync::{
    apply_feeds, get_config, list_sources, parse_stay_rows, update_config, ApplyFeedsArgs,
    CalendarFormat, CalendarInput, FeedBody, FeedParseContext, UpdateConfigArgs,
};
use portaki_sdk::capability;
use portaki_sdk::contracts::booking_channel::{BookingChannel, ChannelSignal};
use portaki_sdk::host::email::SendEmailArgs;
use portaki_test_utils::MockContext;
use serial_test::serial;

#[test]
#[serial]
fn update_config_and_list_sources_many_calendars() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![
                        CalendarInput {
                            id: "airbnb".into(),
                            url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                            label: "Airbnb".into(),
                            format: "airbnb".into(),
                            channel: String::new(),
                        },
                        CalendarInput {
                            id: "".into(),
                            url: "  ".into(),
                            label: "".into(),
                            format: "".into(),
                            channel: String::new(),
                        },
                        CalendarInput {
                            id: "booking".into(),
                            url: "https://admin.booking.com/hotel/hoteladmin/ical.html?t=abc"
                                .into(),
                            label: "Booking".into(),
                            format: "booking".into(),
                            channel: String::new(),
                        },
                        CalendarInput {
                            id: "vrbo".into(),
                            url: "https://www.vrbo.com/calendar/ical/9.ics".into(),
                            label: "".into(),
                            format: "abritel_vrbo".into(),
                            channel: String::new(),
                        },
                    ],
                    ..Default::default()
                },
            )
            .expect("update");

            let sources = list_sources(ctx.clone()).expect("sources");
            assert_eq!(sources.sources.len(), 3);
            assert_eq!(sources.sources[0].id, "airbnb");
            assert_eq!(sources.sources[0].provider.as_deref(), Some("airbnb"));
            assert_eq!(sources.sources[1].provider.as_deref(), Some("booking"));
            assert_eq!(sources.sources[2].provider.as_deref(), Some("abritel_vrbo"));

            let config = get_config(ctx).expect("config");
            assert_eq!(config.calendars.len(), 3);
            assert_eq!(config.calendars[0].format, CalendarFormat::Airbnb);
            assert!(config.calendars[0].url.contains("airbnb.com"));
            let json = serde_json::to_value(&config).expect("serialize");
            assert!(json.get("ical_url_primary").is_none());
        });
}

#[test]
#[serial]
fn update_config_detects_format_from_url_when_omitted() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "auto".into(),
                        url: "https://www.airbnb.com/calendar/ical/99.ics".into(),
                        label: "".into(),
                        format: "".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let config = get_config(ctx).expect("config");
            assert_eq!(config.calendars[0].format, CalendarFormat::Airbnb);
        });
}

#[test]
#[serial]
fn legacy_primary_secondary_still_accepted() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    ical_url_primary: "https://example.com/a.ics".into(),
                    ical_url_secondary: "https://example.com/b.ics".into(),
                    ..Default::default()
                },
            )
            .expect("update");

            let sources = list_sources(ctx.clone()).expect("sources");
            assert_eq!(sources.sources.len(), 2);
            assert_eq!(sources.sources[0].provider.as_deref(), Some("generic"));

            let config = get_config(ctx).expect("config");
            assert_eq!(config.calendars.len(), 2);
            let json = serde_json::to_value(&config).expect("serialize");
            assert!(json.get("ical_url_primary").is_none());
            assert!(json.get("ical_url_secondary").is_none());
        });
}

#[test]
#[serial]
fn apply_feeds_parses_ics_and_updates_summary() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://example.com/a.ics".into(),
                        label: "".into(),
                        format: "airbnb".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:u1\n\
DTSTART;VALUE=DATE:20260801\nDTEND;VALUE=DATE:20260805\n\
SUMMARY:Reserved\nDESCRIPTION:Name: Sofia Rossi\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:u2\n\
DTSTART;VALUE=DATE:20260810\nDTEND;VALUE=DATE:20260812\n\
SUMMARY:Reserved - Not available\nEND:VEVENT\nEND:VCALENDAR\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("airbnb".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert!(result.ok);
            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.rows[0].guest_name, "Sofia Rossi");
            assert_eq!(result.rows[0].ical_uid, "u1");
            // UID `u1` is opaque and the feed has no PRODID — the declared
            // Airbnb feed shape carries the channel.
            assert_eq!(result.rows[0].booking_channel, BookingChannel::Airbnb);
            assert_eq!(
                result.rows[0].booking_channel_signal,
                ChannelSignal::FeedFormatDeclared
            );
            assert!(result.updated_plain_config.last_sync_at.is_some());
            assert!(result
                .updated_plain_config
                .sync_summary
                .as_deref()
                .unwrap_or("")
                .contains("1 stay"));
        });
}

#[test]
#[serial]
fn apply_feeds_blocks_only_still_succeeds() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                        label: "".into(),
                        format: "airbnb".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let ics = "BEGIN:VEVENT\nUID:b1\nDTSTART;VALUE=DATE:20260801\n\
DTEND;VALUE=DATE:20260802\nSUMMARY:Not available\nEND:VEVENT\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("airbnb".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert!(result.ok);
            assert_eq!(result.succeeded, 1);
            assert_eq!(result.failed, 0);
            assert!(result.rows.is_empty());
        });
}

#[test]
fn parse_stay_rows_unit() {
    let rows = parse_stay_rows(
        "BEGIN:VEVENT\nUID:x\nDTSTART;VALUE=DATE:20260101\nDTEND;VALUE=DATE:20260103\nSUMMARY:A\nEND:VEVENT\n",
        "en",
        10,
        &FeedParseContext::from_format(CalendarFormat::Generic),
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].guest_name, "A");
    assert_eq!(rows[0].booking_channel, BookingChannel::Unknown);
    assert_eq!(rows[0].booking_channel_signal, ChannelSignal::None);
}

#[test]
#[serial]
fn apply_feeds_empty_body_counts_as_failed() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://example.com/a.ics".into(),
                        label: "Booking".into(),
                        format: "booking".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("booking".into()),
                        ics_body: String::new(),
                    }],
                },
            )
            .expect("apply");

            assert!(!result.ok);
            assert_eq!(result.failed, 1);
            assert_eq!(result.succeeded, 0);
        });
}

#[test]
#[serial]
fn apply_feeds_second_pass_is_idempotent_for_same_uids() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://example.com/a.ics".into(),
                        label: "".into(),
                        format: "generic".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let ics = "BEGIN:VEVENT\nUID:stable-1\nDTSTART;VALUE=DATE:20260801\n\
DTEND;VALUE=DATE:20260805\nSUMMARY:Ada Lovelace\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:stable-2\nDTSTART;VALUE=DATE:20260810\n\
DTEND;VALUE=DATE:20260812\nSUMMARY:Tom Weber\nEND:VEVENT\n";

            let first = apply_feeds(
                ctx.clone(),
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("generic".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply first");
            assert_eq!(first.rows.len(), 2);

            let second = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("generic".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply second");
            assert_eq!(second.rows.len(), 2);
            assert!(second.ok);
        });
}

#[test]
#[serial]
fn apply_feeds_reports_the_channel_on_every_row() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://example.com/a.ics".into(),
                        label: "".into(),
                        format: "generic".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:r1@airbnb.com\n\
DTSTART;VALUE=DATE:20260801\nDTEND;VALUE=DATE:20260805\nSUMMARY:Ada\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:r2\nDTSTART;VALUE=DATE:20260810\n\
DTEND;VALUE=DATE:20260812\nSUMMARY:Tom\nEND:VEVENT\nEND:VCALENDAR\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "primary".into(),
                        provider: Some("generic".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert_eq!(result.rows.len(), 2);
            assert_eq!(result.rows[0].booking_channel, BookingChannel::Airbnb);
            assert_eq!(
                result.rows[0].booking_channel_signal,
                ChannelSignal::IcalUidSuffix
            );
            assert_eq!(result.rows[1].booking_channel, BookingChannel::Unknown);
            assert_eq!(result.rows[1].booking_channel_signal, ChannelSignal::None);

            // Both keys are on the wire for both rows, never omitted.
            let wire = serde_json::to_value(&result.rows).expect("serialize");
            assert_eq!(wire[0]["bookingChannel"], "airbnb");
            assert_eq!(wire[0]["bookingChannelSignal"], "ical-uid-suffix");
            assert_eq!(wire[1]["bookingChannel"], "unknown");
            assert_eq!(wire[1]["bookingChannelSignal"], "none");
        });
}

#[test]
#[serial]
fn google_mirrored_feed_reports_unknown_not_google() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "mirror".into(),
                        url: "https://calendar.google.com/calendar/ical/x/basic.ics".into(),
                        label: "".into(),
                        format: "google".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let config = get_config(ctx.clone()).expect("config");
            assert_eq!(config.calendars[0].format, CalendarFormat::Google);
            assert_eq!(config.calendars[0].channel, BookingChannel::Unknown);

            let ics = "BEGIN:VCALENDAR\nVERSION:2.0\n\
PRODID:-//Google Inc//Google Calendar 70.9054//EN\nBEGIN:VEVENT\n\
UID:9fk3@google.com\nDTSTART;VALUE=DATE:20260901\nDTEND;VALUE=DATE:20260903\n\
SUMMARY:Famille Bernard\nEND:VEVENT\nEND:VCALENDAR\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "mirror".into(),
                        provider: Some("google".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.rows[0].booking_channel, BookingChannel::Unknown);
            assert_eq!(result.rows[0].booking_channel_signal, ChannelSignal::None);
        });
}

#[test]
#[serial]
fn host_declared_platform_carries_an_opaque_channel_manager_feed() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "beds24".into(),
                        url: "https://api.beds24.com/ical/9931.ics".into(),
                        label: "Beds24".into(),
                        format: "generic".into(),
                        channel: "booking".into(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let config = get_config(ctx.clone()).expect("config");
            assert_eq!(config.calendars[0].channel, BookingChannel::Booking);
            assert_eq!(
                config.calendars[0].channel_signal,
                ChannelSignal::HostOverride
            );

            let ics = "BEGIN:VCALENDAR\nPRODID:-//Beds24//Calendar//EN\nBEGIN:VEVENT\n\
UID:bd24-9931-7\nDTSTART;VALUE=DATE:20260901\nDTEND;VALUE=DATE:20260903\n\
SUMMARY:Nina Faure\nEND:VEVENT\nEND:VCALENDAR\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "beds24".into(),
                        provider: Some("generic".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.rows[0].booking_channel, BookingChannel::Booking);
            assert_eq!(
                result.rows[0].booking_channel_signal,
                ChannelSignal::HostOverride
            );
        });
}

#[test]
#[serial]
fn airbnb_url_prefills_the_platform_when_the_host_leaves_it_blank() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "explicit-generic".into(),
                        url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                        label: "".into(),
                        format: "generic".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            let config = get_config(ctx.clone()).expect("config");
            assert_eq!(config.calendars[0].format, CalendarFormat::Generic);
            assert_eq!(config.calendars[0].channel, BookingChannel::Airbnb);
            assert_eq!(
                config.calendars[0].channel_signal,
                ChannelSignal::FeedUrlHost
            );

            let ics = "BEGIN:VEVENT\nUID:opaque-9\nDTSTART;VALUE=DATE:20260901\n\
DTEND;VALUE=DATE:20260903\nSUMMARY:Lou Girard\nEND:VEVENT\n";

            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds: vec![FeedBody {
                        id: "explicit-generic".into(),
                        provider: Some("generic".into()),
                        ics_body: ics.into(),
                    }],
                },
            )
            .expect("apply");

            assert_eq!(result.rows[0].booking_channel, BookingChannel::Airbnb);
            assert_eq!(
                result.rows[0].booking_channel_signal,
                ChannelSignal::FeedUrlHost
            );
        });
}

fn failing_feed(id: &str) -> FeedBody {
    FeedBody {
        id: id.into(),
        provider: Some("booking".into()),
        ics_body: String::new(),
    }
}

/// Runs `applyFeeds` over `feeds` as one invocation on a frozen day: (failed, succeeded, emails).
fn apply_in_one_invocation(feeds: Vec<FeedBody>) -> (i32, i32, Vec<SendEmailArgs>) {
    let now = chrono::DateTime::parse_from_rfc3339("2026-09-14T06:00:00Z")
        .expect("instant")
        .with_timezone(&chrono::Utc);
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .with_now(now)
        .run_with(|ctx, host| {
            let result = apply_feeds(
                ctx,
                ApplyFeedsArgs {
                    guest_lang: "fr".into(),
                    feeds,
                },
            )
            .expect("apply");
            (result.failed, result.succeeded, host.sent_emails())
        })
}

fn listed_feeds(body: &str) -> usize {
    body.lines().filter(|line| line.starts_with("• ")).count()
}

/// Eight feeds fail beside a working one: one `sync-failed` email names all eight, and the run
/// sends two emails in all (failure + stay-imported), under the per-invocation cap.
#[test]
#[serial]
fn eight_failed_feeds_share_one_failure_email() {
    let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:u1\n\
DTSTART;VALUE=DATE:20260801\nDTEND;VALUE=DATE:20260805\n\
SUMMARY:Reserved\nDESCRIPTION:Name: Sofia Rossi\nEND:VEVENT\nEND:VCALENDAR\n";
    let mut feeds: Vec<FeedBody> = (1..=8)
        .map(|index| failing_feed(&format!("broken-{index}")))
        .collect();
    feeds.push(FeedBody {
        id: "primary".into(),
        provider: Some("airbnb".into()),
        ics_body: ics.into(),
    });

    let (failed, succeeded, sent) = apply_in_one_invocation(feeds);
    assert_eq!((failed, succeeded), (8, 1));
    let ids: Vec<&str> = sent.iter().map(|email| email.email_id.as_str()).collect();
    assert_eq!(sent.len(), 2, "{ids:?}");
    assert!(ids.iter().any(|id| id.starts_with("stay-imported-")));

    let failures: Vec<&SendEmailArgs> = sent
        .iter()
        .filter(|email| email.email_id.starts_with("sync-failed-"))
        .collect();
    assert_eq!(failures.len(), 1, "{ids:?}");
    let failure = failures[0];
    assert!(failure.content.subject.en.starts_with("8 calendars"));
    assert!(failure.content.subject.fr.starts_with("8 calendriers"));
    assert_eq!(listed_feeds(&failure.content.body.fr), 8);
    assert_eq!(listed_feeds(&failure.content.body.en), 8);
    assert!(!failure.content.body.en.contains("… and"));
}

/// The multi-feed email id is keyed on the day and the set of failed feeds, not their order; a
/// lone failure keeps its per-feed id; past ten feeds the list names ten and counts the rest.
#[test]
#[serial]
fn failure_email_is_keyed_on_the_failed_set_and_bounded() {
    let email_id = |names: &[&str]| -> String {
        let (_, _, sent) =
            apply_in_one_invocation(names.iter().copied().map(failing_feed).collect());
        assert_eq!(sent.len(), 1);
        sent[0].email_id.clone()
    };
    let abc = email_id(&["a", "b", "c"]);
    assert!(abc.starts_with("sync-failed-2026-09-14-"), "{abc}");
    assert_eq!(abc, email_id(&["c", "a", "b"]));
    assert_ne!(abc, email_id(&["a", "b"]));
    assert_eq!(email_id(&["a"]), "sync-failed-a-2026-09-14");

    let twelve = (1..=12)
        .map(|index| failing_feed(&format!("broken-{index}")))
        .collect();
    let (failed, _, sent) = apply_in_one_invocation(twelve);
    assert_eq!(failed, 12);
    assert_eq!(sent.len(), 1);
    let body = &sent[0].content.body.en;
    assert_eq!(listed_feeds(body), 10);
    assert!(body.contains("… and 2 more"), "{body}");
    assert!(sent[0].content.body.fr.contains("… et 2 autre(s)"));
}

#[test]
#[serial]
fn stats_card_reads_history_conflicts_and_channels() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-07-25T09:00:00Z")
        .expect("now")
        .with_timezone(&chrono::Utc);
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .with_now(now)
        .run(|mut ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                        label: "".into(),
                        format: "airbnb".into(),
                        channel: String::new(),
                    }],
                    ..Default::default()
                },
            )
            .expect("update");

            // Two stays sharing the night of Aug 4 — one date conflict.
            let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:u1@airbnb.com\n\
DTSTART;VALUE=DATE:20260801\nDTEND;VALUE=DATE:20260805\n\
SUMMARY:Reserved\nDESCRIPTION:Name: Sofia Rossi\nEND:VEVENT\n\
BEGIN:VEVENT\nUID:u3@airbnb.com\n\
DTSTART;VALUE=DATE:20260804\nDTEND;VALUE=DATE:20260806\n\
SUMMARY:Reserved\nDESCRIPTION:Name: Leo Martin\nEND:VEVENT\nEND:VCALENDAR\n";
            let feed = |body: &str| ApplyFeedsArgs {
                guest_lang: "fr".into(),
                feeds: vec![FeedBody {
                    id: "primary".into(),
                    provider: Some("airbnb".into()),
                    ics_body: body.into(),
                }],
            };
            apply_feeds(ctx.clone(), feed(ics)).expect("apply");
            apply_feeds(ctx.clone(), feed("")).expect("failed run");

            ctx.input = serde_json::json!({ "periodDays": 30 });
            let text = serde_json::to_value(ical_sync::render_host_stats(ctx))
                .expect("surface json")
                .to_string();
            assert!(text.contains(r#""delta":"sur 30 jours","label":"i18n:stats.imported","type":"Stat","value":"2""#), "{text}");
            assert!(text.contains(r#""delta":"i18n:stats.conflicts.note","label":"i18n:stats.conflicts","type":"Stat","value":"1""#));
            assert!(text.contains(r#""delta":"i18n:stats.incomplete.note","label":"i18n:stats.incomplete","type":"Stat","value":"2""#));
            assert!(text.contains(r#""highlight":13,"kind":"bars""#));
            assert!(text.contains(r#""display":"1 réussie · 1 en échec","label":"25","value":2.0"#));
            assert!(text.contains(r#""display":"2 séjours","label":"Airbnb","value":2.0"#));
        });
}
