//! Guest-email checkout tips for Portaki `lost-found` (and future checkout mails).

use portaki_sdk::prelude::*;

use crate::labels::{get_label, labels_from_item, pick_label};

const MAX_TIPS: usize = 3;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready checklist contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_tips: Option<String>,
}

/// Top checklist labels for Portaki guest templates.
#[portaki_sdk::query(
    name = "emailContext",
    example(
        label = "E-mail après l'arrivée",
        input = r#"{"templateKey":"post-arrival"}"#
    ),
    example(
        label = "E-mail objet oublié, en anglais",
        input = r#"{"templateKey":"lost-found","locale":"en-US"}"#
    )
)]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[EmailTemplateKey::LostFound, EmailTemplateKey::PostArrival]) {
        return Ok(EmailContextResponse {
            checkout_tips: None,
        });
    }

    let locale = args.locale_or(ctx.locale.as_str());
    let property_locale = ctx.property.locale.as_str();

    let items = crate::queries::guest_items()?;

    let tips: Vec<String> = items
        .iter()
        .filter_map(|item| {
            let labels = labels_from_item(item);
            let label = pick_label(&labels, locale, property_locale);
            let trimmed = label.trim();
            if trimmed.is_empty() {
                let fallback = get_label(item, locale);
                let fallback = fallback.trim();
                if fallback.is_empty() {
                    None
                } else {
                    Some(fallback.to_string())
                }
            } else {
                Some(trimmed.to_string())
            }
        })
        .take(MAX_TIPS)
        .collect();

    if tips.is_empty() {
        return Ok(EmailContextResponse {
            checkout_tips: None,
        });
    }

    Ok(EmailContextResponse {
        checkout_tips: Some(tips.join("\n")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lists, storage};
    use portaki_test_utils::MockContext;

    #[test]
    fn when_lost_found_then_returns_top_tips() {
        storage::reset_test_store();
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .run(|ctx| {
                storage::create_from_template(lists::template("departure").unwrap()).unwrap();
                let out = build_email_context(
                    ctx,
                    EmailContextArgs {
                        template_key: Some(EmailTemplateKey::LostFound),
                        locale: Some("fr".into()),
                        ..Default::default()
                    },
                )
                .unwrap();
                let tips = out.checkout_tips.expect("tips");
                assert!(tips.contains("Fermer les volets"));
            });
    }
}
