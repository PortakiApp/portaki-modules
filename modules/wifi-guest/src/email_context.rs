//! Guest-email Wi-Fi network name for Portaki arrival templates.

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready wifi-guest contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_name: Option<String>,
}

#[portaki_sdk::query(
    name = "emailContext",
    example(
        label = "E-mail de la veille d'arrivée",
        input = r#"{"templateKey":"arrival"}"#
    ),
    example(
        label = "E-mail du jour d'arrivée",
        input = r#"{"templateKey":"arrival-day"}"#
    )
)]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[EmailTemplateKey::Arrival, EmailTemplateKey::ArrivalDay]) {
        return Ok(EmailContextResponse { wifi_name: None });
    }

    let config = ModuleConfig::load(&ctx)?;
    let ssid = config.ssid.trim();
    Ok(EmailContextResponse {
        wifi_name: if ssid.is_empty() {
            None
        } else {
            Some(ssid.to_string())
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
    fn when_arrival_then_returns_wifi_name() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({
                "ssid": "Belledonne_Guest",
                "password": "secret"
            }))
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
            assert_eq!(out.wifi_name.as_deref(), Some("Belledonne_Guest"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn when_wrong_template_then_empty() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&json!({ "ssid": "Belledonne_Guest" }))
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
            assert!(out.wifi_name.is_none());
        });
    }
}
