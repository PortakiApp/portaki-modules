//! Guest-email local tip for Portaki `arrival-day` / `post-arrival`.

use portaki_sdk::prelude::*;

use crate::config::load_config;
use crate::nearby::resolve_events;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready events contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_tip: Option<String>,
}

#[portaki_sdk::query(name = "emailContext")]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[EmailTemplateKey::ArrivalDay, EmailTemplateKey::PostArrival]) {
        return Ok(EmailContextResponse { local_tip: None });
    }

    let locale = args.locale_or(ctx.locale.as_str());
    let property_locale = ctx.property.locale.as_str();

    let config = load_config().unwrap_or_default();
    let events = resolve_events(&ctx, &config, true)?;
    let Some(event) = events.first() else {
        return Ok(EmailContextResponse { local_tip: None });
    };

    let title = event.title.pick_with_fallback(locale, property_locale);
    let title = title.trim();
    if title.is_empty() {
        return Ok(EmailContextResponse { local_tip: None });
    }

    let mut tip = title.to_string();
    let place = event.place.pick_with_fallback(locale, property_locale);
    let place = place.trim();
    if !place.is_empty() {
        tip.push_str(" · ");
        tip.push_str(place);
    }

    Ok(EmailContextResponse {
        local_tip: Some(tip),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_sdk::host::with_host;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    #[serial_test::serial]
    fn when_arrival_day_then_returns_first_upcoming_event() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_kv(
                "config",
                serde_json::to_vec(&json!({
                    "events": [{
                        "id": "evt-1",
                        "title": {"fr": "Marché provençal", "en": "Provence market"},
                        "place": {"fr": "Place du marché", "en": "Market square"},
                        "starts_at": "2099-07-25T08:00:00Z"
                    }]
                }))
                .unwrap(),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::ArrivalDay),
                    locale: Some("fr".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                out.local_tip.as_deref(),
                Some("Marché provençal · Place du marché")
            );
        });
    }
}
