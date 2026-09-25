//! Guest-email host phone for Portaki arrival / post-arrival / lost-found.

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready emergency-contacts contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_phone: Option<String>,
}

/// Host-visible phone for Portaki guest templates.
#[portaki_sdk::query(name = "emailContext")]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[
        EmailTemplateKey::Arrival,
        EmailTemplateKey::ArrivalDay,
        EmailTemplateKey::PostArrival,
        EmailTemplateKey::LostFound,
    ]) {
        return Ok(EmailContextResponse { host_phone: None });
    }

    let config = ModuleConfig::read(&ctx)?;
    let phone = config.host_visible_phone.trim();
    Ok(EmailContextResponse {
        host_phone: if phone.is_empty() {
            None
        } else {
            Some(phone.to_string())
        },
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
    fn when_arrival_then_returns_host_phone() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({ "host_visible_phone": "+33 6 12 34 56 78" }))
            .build();

        with_host(host, ctx.clone(), || {
            let out = build_email_context(
                ctx.clone(),
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    locale: None,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(out.host_phone.as_deref(), Some("+33 6 12 34 56 78"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn when_wrong_template_then_empty() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({ "host_visible_phone": "+33 6 12 34 56 78" }))
            .build();

        with_host(host, ctx.clone(), || {
            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::StayLink),
                    locale: None,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(out.host_phone.is_none());
        });
    }
}
