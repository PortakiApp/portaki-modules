//! `secrets-leak` de la batterie plateforme : le lien d'un calendrier (`calendars[].url`) est un
//! secret de ligne. Semé comme la batterie le sème — une ligne `{ id, url: <sentinelle> }` —, il
//! ne ressort d'aucune sortie : arbre SDUI, réponse, journal, e-mail.
//!
//! Une seule exception, le contrat de la synchro : `listSources` rend le lien à la plateforme,
//! qui le relève (`SyncPropertyModuleUseCaseImpl.buildFeedParams`). À l'hôte seulement : un
//! appel voyageur est refusé.

use ical_sync::{
    apply_feeds, list_sources, publish_readiness, render_host_main, render_host_stats,
    render_host_stay, stats_summary, timeline_tasks, ApplyFeedsArgs, FeedBody,
};
use portaki_sdk::capability;
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use serial_test::serial;

const SENTINEL: &str = "ptk-sentinel-0123456789abcdef";

fn seeded() -> Value {
    json!({
        "calendars": [{ "id": "row-1", "url": SENTINEL }],
        "missing_email": "notify",
    })
}

#[test]
#[serial]
fn the_feed_link_never_comes_out() {
    MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::core::MODULES_SCHEDULED_SYNC,
        ])
        .with_config(&seeded())
        .run_with(|ctx, host| {
            let mut outputs = vec![
                json!(render_host_main(ctx.clone()).ok()),
                json!(render_host_stats(ctx.clone()).ok()),
                json!(render_host_stay(ctx.clone()).ok()),
                json!(publish_readiness(ctx.clone()).ok()),
            ];
            outputs.push(json!(stats_summary(
                ctx.clone(),
                portaki_sdk::contracts::stats::StatsSummaryArgs {
                    property_id: ctx.property_id,
                    period: 30,
                    key: "calendar-sync".into(),
                    views: None,
                },
            )
            .ok()));
            outputs.push(json!(timeline_tasks(
                ctx.clone(),
                serde_json::from_value(json!({
                    "propertyId": ctx.property_id,
                    "from": "2026-06-01T00:00:00Z",
                    "to": "2026-06-08T00:00:00Z",
                }))
                .expect("timeline args"),
            )
            .ok()));
            for body in ["", "BEGIN:VCALENDAR\nEND:VCALENDAR\n"] {
                outputs.push(json!(apply_feeds(
                    ctx.clone(),
                    ApplyFeedsArgs {
                        guest_lang: "fr".into(),
                        feeds: vec![FeedBody {
                            id: "row-1".into(),
                            provider: None,
                            ics_body: body.into(),
                        }],
                    },
                )
                .ok()));
            }
            let leaks: Vec<String> = outputs
                .iter()
                .map(Value::to_string)
                .filter(|output| output.contains(SENTINEL))
                .collect();
            assert!(leaks.is_empty(), "{leaks:#?}");
            assert!(!format!("{:?}", host.logs()).contains(SENTINEL));
            let emails = host.sent_emails();
            assert!(!emails.is_empty(), "the failing run sends its e-mail");
            assert!(!format!("{emails:?}").contains(SENTINEL), "{emails:?}");
        });
}

/// La synchro relève le lien : il revient à la plateforme, appelante hôte.
#[test]
#[serial]
fn the_platform_gets_the_link_to_fetch() {
    MockContext::host().with_config(&seeded()).run(|ctx| {
        let sources = list_sources(ctx).expect("sources");
        assert_eq!(sources.sources[0].url, SENTINEL);
    });
}

/// Un voyageur n'obtient jamais les liens (sonde « mauvaise audience » de la batterie).
#[test]
#[serial]
fn a_guest_never_gets_the_links() {
    MockContext::guest().with_config(&seeded()).run(|ctx| {
        let listed = list_sources(ctx);
        assert!(listed.is_err(), "{listed:?}");
    });
}
