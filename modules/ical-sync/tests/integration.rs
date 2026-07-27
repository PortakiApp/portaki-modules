//! Integration-style unit tests with `portaki-test-utils`.

use ical_sync::{
    apply_feeds, get_config, list_sources, parse_stay_rows, update_config, ApplyFeedsArgs,
    CalendarFormat, CalendarInput, FeedBody, UpdateConfigArgs,
};
use portaki_sdk::capability;
use portaki_test_utils::MockContext;
use serial_test::serial;

#[test]
#[serial]
fn update_config_and_list_sources_many_calendars() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE, capability::core::ICAL_IMPORT])
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
                        },
                        CalendarInput {
                            id: "".into(),
                            url: "  ".into(),
                            label: "".into(),
                            format: "".into(),
                        },
                        CalendarInput {
                            id: "booking".into(),
                            url: "https://admin.booking.com/hotel/hoteladmin/ical.html?t=abc"
                                .into(),
                            label: "Booking".into(),
                            format: "booking".into(),
                        },
                        CalendarInput {
                            id: "vrbo".into(),
                            url: "https://www.vrbo.com/calendar/ical/9.ics".into(),
                            label: "".into(),
                            format: "abritel_vrbo".into(),
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
            assert_eq!(
                sources.sources[2].provider.as_deref(),
                Some("abritel_vrbo")
            );

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
        .with_capabilities(&[capability::core::STORAGE, capability::core::ICAL_IMPORT])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "auto".into(),
                        url: "https://www.airbnb.com/calendar/ical/99.ics".into(),
                        label: "".into(),
                        format: "".into(),
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
        .with_capabilities(&[capability::core::STORAGE, capability::core::ICAL_IMPORT])
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
        .with_capabilities(&[capability::core::STORAGE, capability::core::ICAL_IMPORT])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://example.com/a.ics".into(),
                        label: "".into(),
                        format: "airbnb".into(),
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
        .with_capabilities(&[capability::core::STORAGE, capability::core::ICAL_IMPORT])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    calendars: vec![CalendarInput {
                        id: "primary".into(),
                        url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                        label: "".into(),
                        format: "airbnb".into(),
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
        CalendarFormat::Generic,
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].guest_name, "A");
}
