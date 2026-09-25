//! Guest-email teaser for Portaki `stay-link`, `arrival`, `post-arrival`.

use portaki_sdk::prelude::*;

use crate::queries::{get_content, GetContentArgs};

const MAX_RULES: usize = 3;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready house-rules contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub house_rules_teaser: Option<String>,
}

/// Short house-rules lines for Portaki guest templates.
#[portaki_sdk::query(
    name = "emailContext",
    example(label = "E-mail d'arrivée", input = r#"{"templateKey":"arrival"}"#),
    example(
        label = "E-mail du lien de séjour, en anglais",
        input = r#"{"templateKey":"stay-link","locale":"en-US"}"#
    )
)]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[
        EmailTemplateKey::StayLink,
        EmailTemplateKey::Arrival,
        EmailTemplateKey::PostArrival,
    ]) {
        return Ok(EmailContextResponse {
            house_rules_teaser: None,
        });
    }

    let locale = args.locale_or(ctx.locale.as_str());

    let view = get_content(
        ctx.clone(),
        GetContentArgs {
            locale: Some(locale.to_string()),
        },
    )?;

    let lines: Vec<String> = view
        .items
        .iter()
        .filter_map(|item| {
            let title = item.title.trim();
            if title.is_empty() {
                return None;
            }
            let subtitle = item.subtitle.trim();
            if subtitle.is_empty() {
                Some(title.to_string())
            } else {
                Some(format!("{title} — {subtitle}"))
            }
        })
        .take(MAX_RULES)
        .collect();

    if lines.is_empty() {
        return Ok(EmailContextResponse {
            house_rules_teaser: None,
        });
    }

    Ok(EmailContextResponse {
        house_rules_teaser: Some(lines.join("\n")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{RuleItem, RulesPayload};
    use crate::store::{reset_test_store, save_content_row};
    use portaki_test_utils::MockContext;

    #[test]
    fn when_arrival_then_teaser_top_rules() {
        use portaki_sdk::host::with_host;

        reset_test_store();
        let payload = RulesPayload {
            items: vec![
                RuleItem {
                    icon: "clock".into(),
                    title: "Silence après 22h".into(),
                    subtitle: "Respectez le voisinage".into(),
                },
                RuleItem {
                    icon: "paw".into(),
                    title: "Animaux non admis".into(),
                    subtitle: String::new(),
                },
            ],
        };

        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .build();
        with_host(host, ctx.clone(), || {
            save_content_row(payload.to_json_string().unwrap(), String::new()).unwrap();
            let out = build_email_context(
                ctx.clone(),
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    locale: Some("fr".into()),
                    ..Default::default()
                },
            )
            .unwrap();

            let teaser = out.house_rules_teaser.expect("teaser");
            assert!(teaser.contains("Silence après 22h"));
            assert!(teaser.contains("Animaux non admis"));
        });
    }

    #[test]
    fn when_wrong_template_then_empty() {
        reset_test_store();
        let (ctx, _host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .build();
        let out = build_email_context(
            ctx,
            EmailContextArgs {
                template_key: Some(EmailTemplateKey::ArrivalDay),
                locale: None,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.house_rules_teaser.is_none());
    }
}
