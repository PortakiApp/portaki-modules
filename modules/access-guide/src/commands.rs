//! `onConfigUpdated` — the platform's signal after a save: guests hear of a new code.

use portaki_sdk::host::email::{
    self, EmailAudience, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;

/// The keys that hold a code a guest types.
const CODE_KEYS: &[&str] = &[
    "keybox_code",
    "door_code",
    "smart_lock_manual_code",
    "building_access_gate_code",
    "parking_code",
];

/// What the platform sends after a save that changed keys: their names, never their values.
#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct ConfigUpdatedArgs {
    #[serde(default)]
    pub property_id: Option<Uuid>,
    #[serde(default)]
    pub changed_keys: Vec<String>,
}

impl ConfigUpdatedArgs {
    fn changes_a_code(&self) -> bool {
        self.changed_keys
            .iter()
            .any(|key| CODE_KEYS.contains(&key.as_str()))
    }
}

/// A code changed: the guests about to arrive or on site (the platform picks them) get an email
/// pointing to their booklet — never the code itself.
#[portaki_sdk::email(
    id = "code-changed",
    audience = EmailAudience::PropertyEligibleGuests,
    requires_guest_email,
    description_key = "email.code-changed.description"
)]
#[portaki_sdk::command(name = "onConfigUpdated")]
pub fn on_config_updated(ctx: Context, args: ConfigUpdatedArgs) -> Result<()> {
    if !args.changes_a_code() {
        return Ok(());
    }
    email::send(&SendEmailArgs {
        email_id: "code-changed".into(),
        audience: EmailAudience::PropertyEligibleGuests,
        content: ModuleEmailSdui {
            subject: crate::email_i18n::text("email.codeChanged.subject"),
            eyebrow: Some(crate::email_i18n::text("email.codeChanged.eyebrow")),
            title: Some(crate::email_i18n::text("email.codeChanged.title")),
            body: crate::email_i18n::text("email.codeChanged.body"),
            cta: Some(ModuleEmailCta {
                label: crate::email_i18n::text("email.codeChanged.cta"),
                url: None,
                portaki_action: Some("open-module:access-guide:default".into()),
            }),
        },
        stay_id: None,
        property_id: Some(args.property_id.unwrap_or(ctx.property_id)),
        action_url: None,
    })
}
