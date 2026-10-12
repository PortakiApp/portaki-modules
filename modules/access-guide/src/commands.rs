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
    /// Ignored: the email always goes to the invocation's property, never to one the payload names.
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
#[portaki_sdk::command(
    name = "onConfigUpdated",
    example(
        label = "Code de la boîte à clés modifié",
        input = r#"{"changedKeys":["keybox_code"]}"#
    ),
    example(
        label = "Autre réglage modifié",
        input = r#"{"changedKeys":["arrival_video_url"]}"#
    )
)]
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
            ..Default::default()
        },
        stay_id: None,
        property_id: Some(ctx.property_id),
        action_url: None,
    })
}

/// « Changer le code de ce séjour » (`stay-action`) : le code saisi, ou six chiffres tirés au
/// hasard. `code` vient du formulaire de la modale.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct StayCodeArgs {
    pub stay_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default)]
    pub generate: bool,
}

/// Le code d'entrée d'un séjour, « À chaque séjour » (spec Accès §1, §2.2). Hôte seulement ; la
/// plateforme vérifie que le séjour est de ce logement. Le voyageur le lit à sa prochaine visite,
/// sans « Publier ».
#[portaki_sdk::command(
    name = "setStayCode",
    example(
        label = "Code saisi par l'hôte",
        input = r#"{"stayId":"1a2b3c4d-0000-4000-8000-000000000000","code":"4821"}"#
    ),
    example(
        label = "Code généré",
        input = r#"{"stayId":"1a2b3c4d-0000-4000-8000-000000000000","generate":true}"#
    )
)]
pub fn set_stay_code(ctx: Context, args: StayCodeArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".into()));
    }
    if !crate::config::ModuleConfig::read(&ctx)?.code_per_stay {
        return Err(PortakiError::Host("code_per_stay_off".into()));
    }
    let code = if args.generate {
        crate::stay_code::generate()?
    } else {
        args.code
            .as_deref()
            .and_then(crate::stay_code::normalize)
            .ok_or_else(|| PortakiError::Host("stay_code_invalid".into()))?
    };
    portaki_sdk::host::stay_secrets::set(args.stay_id, crate::stay_code::KEY, &code)
}
