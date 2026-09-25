//! `onConfigUpdated` — the platform's signal after a save: guests hear of a new code.

use portaki_sdk::host::email::{
    self, EmailAudience, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;

use crate::config::{HostConfig, PrimaryMethod};

/// The keys that switch which codes the guest uses.
const SWITCH_KEYS: &[&str] = &[
    "primary_method",
    "building_access_enabled",
    "parking_enabled",
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
    /// The code the guest must use changed, in the config as saved: an active code was edited, or
    /// the method / a layer switched and some active code is set.
    fn changes_the_guest_code(&self, config: &HostConfig) -> bool {
        let active = active_codes(config);
        let changed = |key: &str| self.changed_keys.iter().any(|k| k == key);
        active.iter().any(|(key, _)| changed(key))
            || (SWITCH_KEYS.iter().any(|key| changed(key))
                && active.iter().any(|(_, code)| !code.trim().is_empty()))
    }
}

/// The code keys the guest reads with this config — the method's, the layers switched on — and
/// their values.
fn active_codes(config: &HostConfig) -> Vec<(&'static str, &str)> {
    let method = match config.method() {
        Some(PrimaryMethod::Keybox) => Some(("keybox_code", &config.keybox_code)),
        Some(PrimaryMethod::DoorCode) => Some(("door_code", &config.door_code)),
        Some(PrimaryMethod::SmartLock) => {
            Some(("smart_lock_manual_code", &config.smart_lock_manual_code))
        }
        _ => None,
    };
    let building = config.building_access_enabled.then_some((
        "building_access_gate_code",
        &config.building_access_gate_code,
    ));
    let parking = config
        .parking_enabled
        .then_some(("parking_code", &config.parking_code));
    [method, building, parking]
        .into_iter()
        .flatten()
        .map(|(key, code)| (key, code.as_str()))
        .collect()
}

/// The guest's code changed: the guests about to arrive or on site (the platform picks them) get an email
/// pointing to their booklet — never the code itself.
#[portaki_sdk::email(
    id = "code-changed",
    audience = EmailAudience::PropertyEligibleGuests,
    requires_guest_email,
    description_key = "email.code-changed.description"
)]
#[portaki_sdk::command(name = "onConfigUpdated")]
pub fn on_config_updated(ctx: Context, args: ConfigUpdatedArgs) -> Result<()> {
    if args.changed_keys.is_empty() || !args.changes_the_guest_code(&HostConfig::load(&ctx)?) {
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
