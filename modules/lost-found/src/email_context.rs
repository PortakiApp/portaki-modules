//! Declaration descriptions for the Portaki `lost-found` guest template.

use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::description;
use crate::entities::LostFoundReport;
use crate::storage;

/// Gateway `emailContext` args — shared SDK wire type.
pub use portaki_sdk::EmailContextArgs;

/// Email-ready lost-found contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    /// Joined plain descriptions from stay declarations — empty when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lost_item_description: Option<String>,
    /// `true` when at least one [`LostFoundReport`] exists for the stay.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_declaration: bool,
}

#[portaki_sdk::query(
    name = "emailContext",
    example(
        label = "E-mail objets perdus",
        input = r#"{"templateKey":"lost-found"}"#
    )
)]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    if !args.allows_template(&[EmailTemplateKey::LostFound]) {
        return Ok(EmailContextResponse {
            lost_item_description: None,
            has_declaration: false,
        });
    }

    let stay_id = resolve_stay_id(&ctx, &args);
    let reports = stay_id
        .map(storage::list_by_stay)
        .transpose()?
        .unwrap_or_default();
    let has_declaration = !reports.is_empty();
    let lost_item_description = join_descriptions(&reports);

    Ok(EmailContextResponse {
        lost_item_description,
        has_declaration,
    })
}

fn resolve_stay_id(ctx: &Context, args: &EmailContextArgs) -> Option<Uuid> {
    if let Some(guest) = ctx.guest.as_ref() {
        return Some(guest.session_id);
    }
    if let Some(stay) = ctx.stay.as_ref() {
        return Some(stay.stay_id);
    }
    args.stay_id
        .as_deref()
        .and_then(|raw| Uuid::parse_str(raw.trim()).ok())
}

fn join_descriptions(reports: &[LostFoundReport]) -> Option<String> {
    let parts: Vec<String> = reports
        .iter()
        .filter_map(|report| {
            let plain = description::to_plain_text(&report.item_description);
            let text = if plain.is_empty() {
                report.item_description.trim().to_string()
            } else {
                plain
            };
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        })
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_sdk::host::with_host;
    use portaki_test_utils::MockContext;

    #[test]
    #[serial_test::serial]
    fn when_wrong_template_then_empty() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .build();

        with_host(host, ctx.clone(), || {
            let out = build_email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    locale: None,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(out.lost_item_description.is_none());
            assert!(!out.has_declaration);
        });
    }
}
