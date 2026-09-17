//! Guest-email checkout tips for Portaki `lost-found` (and future checkout mails).

use portaki_sdk::prelude::*;

use crate::labels::{get_label, labels_from_item, pick_label};
use crate::storage;

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
#[portaki_sdk::query(name = "emailContext")]
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

    let mut items = storage::list_items()?;
    items.sort_by_key(|item| item.sort_order);

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
#[allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible
mod tests {
    use super::*;
    use crate::storage::seed_test_items;
    use chrono::Utc;
    use portaki_test_utils::MockContext;

    #[test]
    fn when_lost_found_then_returns_top_tips() {
        seed_test_items(
            Utc::now(),
            &[
                ("Vider le frigo", "Empty the fridge"),
                ("Fermer les volets", "Close shutters"),
            ],
        );

        let (ctx, _host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .build();
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
        assert!(tips.contains("Vider le frigo"));
        assert!(tips.contains("Fermer les volets"));
    }
}
