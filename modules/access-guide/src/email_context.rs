//! Guest-email snippets for Portaki templates (`arrival`, `arrival-day`, `new-code`).
//!
//! Reveal policy is applied here — same rules as guest SDUI. Never return plaintext
//! codes while locked.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::{load_config, DoorCodeTarget, MethodFields, ModuleConfig, PrimaryMethod};
use crate::reveal::{evaluate_reveal, format_available_from, locked_message, RevealDecision};

/// Arguments for `emailContext`.
#[portaki_sdk::wire]
#[derive(Default)]
pub struct EmailContextArgs {
    /// Portaki template key (`arrival`, `arrival-day`, `new-code`, …).
    #[serde(default)]
    pub template_key: Option<EmailTemplateKey>,
    /// Formatted check-in clock time for callout copy (`16:00`).
    #[serde(default)]
    pub checkin_time_formatted: Option<String>,
    /// Optional locale override (BCP-47). Kept for wire compat; copy uses host i18n.
    #[serde(default)]
    pub locale: Option<String>,
}

/// Email-ready access-guide contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrival_callout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code_label: Option<String>,
    pub secrets_revealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reveal_available_from: Option<DateTime<Utc>>,
}

impl EmailContextResponse {
    fn empty(revealed: bool, available_from: Option<DateTime<Utc>>) -> Self {
        Self {
            arrival_callout: None,
            entry_access_code: None,
            access_code_label: None,
            secrets_revealed: revealed,
            reveal_available_from: available_from,
        }
    }
}

/// Reveal-aware snippets for Portaki guest email templates.
#[portaki_sdk::query(name = "emailContext")]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(&ctx, &args)
}

/// Build reveal-aware email snippets from config + stay context.
pub fn build_email_context(ctx: &Context, args: &EmailContextArgs) -> Result<EmailContextResponse> {
    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let now = time::now().unwrap_or_else(|_| Utc::now());
    let decision = evaluate_reveal(config.reveal_policy, now, checkin_at, &property_timezone);

    // arrival / new-code: method callout + code when revealed.
    // arrival-day: code only (weather is a separate module).
    // stay-link / other: no access-guide contribution.
    let (wants_callout, wants_code) = match args.template_key {
        None | Some(EmailTemplateKey::Arrival) | Some(EmailTemplateKey::NewCode) => (true, true),
        Some(EmailTemplateKey::ArrivalDay) => (false, true),
        Some(_) => (false, false),
    };

    if !wants_callout && !wants_code {
        return Ok(EmailContextResponse::empty(
            decision.revealed,
            decision.available_from,
        ));
    }

    let time_label = checkin_time_label(args.checkin_time_formatted.as_deref())?;
    let plaintext_code = if decision.revealed {
        extract_entry_code(&config)
    } else {
        None
    };
    let has_configured_code = extract_entry_code(&config).is_some();
    let label = if wants_code && (decision.revealed || has_configured_code) {
        Some(access_code_label(&config)?)
    } else {
        None
    };

    let callout = if wants_callout {
        Some(arrival_callout(
            &config,
            &time_label,
            plaintext_code.is_some(),
            &decision,
            &property_timezone,
        )?)
    } else {
        None
    };

    Ok(EmailContextResponse {
        arrival_callout: callout.filter(|s| !s.trim().is_empty()),
        entry_access_code: if wants_code { plaintext_code } else { None },
        access_code_label: label.filter(|s| !s.trim().is_empty()),
        secrets_revealed: decision.revealed,
        reveal_available_from: decision.available_from,
    })
}

fn property_timezone(ctx: &Context) -> String {
    let from_property = ctx.property.timezone.trim();
    if !from_property.is_empty() {
        return from_property.to_string();
    }
    let from_ctx = ctx.timezone.trim();
    if !from_ctx.is_empty() {
        return from_ctx.to_string();
    }
    "Europe/Paris".to_string()
}

fn checkin_time_label(formatted: Option<&str>) -> Result<String> {
    let trimmed = formatted.map(str::trim).unwrap_or("");
    if !trimmed.is_empty() {
        return Ok(trimmed.to_string());
    }
    t!("email.checkinTime.fallback")
}

fn extract_entry_code(config: &ModuleConfig) -> Option<String> {
    match &config.method {
        MethodFields::Keybox { code: Some(c), .. } if !c.trim().is_empty() => {
            Some(c.trim().to_string())
        }
        MethodFields::DoorCode { code, .. } if !code.trim().is_empty() => {
            Some(code.trim().to_string())
        }
        MethodFields::SmartLock {
            manual_code: Some(c),
        } if !c.trim().is_empty() => Some(c.trim().to_string()),
        _ => config
            .building_access
            .as_ref()
            .and_then(|b| b.gate_code.as_deref())
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_string),
    }
}

fn access_code_label(config: &ModuleConfig) -> Result<String> {
    match &config.method {
        MethodFields::Keybox { .. } => t!("email.label.keybox"),
        MethodFields::DoorCode { target, .. } => door_code_label(*target),
        _ => t!("email.label.access"),
    }
}

fn door_code_label(target: DoorCodeTarget) -> Result<String> {
    match target {
        DoorCodeTarget::Gate => t!("email.label.gate"),
        DoorCodeTarget::Building => t!("email.label.building"),
        DoorCodeTarget::Apartment => t!("email.label.apartment"),
    }
}

fn arrival_callout(
    config: &ModuleConfig,
    time: &str,
    has_revealed_code: bool,
    decision: &RevealDecision,
    property_timezone: &str,
) -> Result<String> {
    let method = config.method.primary_method();

    // Timed lock: keep method framing, never promise "the code below".
    if !decision.revealed && extract_entry_code(config).is_some() {
        let when = decision
            .available_from
            .map(|at| format_available_from(at, property_timezone));
        let locked = locked_message(when.as_deref());
        return match method {
            PrimaryMethod::Keybox | PrimaryMethod::DoorCode | PrimaryMethod::SmartLock => {
                t!(
                    "email.callout.locked.selfCheckin",
                    time = time,
                    locked = locked
                )
            }
            _ => method_callout(config, time, false),
        };
    }

    method_callout(config, time, has_revealed_code)
}

fn method_callout(config: &ModuleConfig, time: &str, has_code: bool) -> Result<String> {
    match &config.method {
        MethodFields::Keybox { .. } => {
            if has_code {
                t!("email.callout.keybox.withCode", time = time)
            } else {
                t!("email.callout.keybox.withoutCode", time = time)
            }
        }
        MethodFields::DoorCode { .. } => {
            if has_code {
                t!("email.callout.doorCode.withCode", time = time)
            } else {
                t!("email.callout.doorCode.withoutCode", time = time)
            }
        }
        MethodFields::SmartLock { .. } => {
            if has_code {
                t!("email.callout.smartLock.withCode", time = time)
            } else {
                t!("email.callout.smartLock.withoutCode", time = time)
            }
        }
        MethodFields::InPerson { meeting_place, .. } => {
            let place = meeting_place.trim();
            if place.is_empty() {
                t!("email.callout.inPerson.noPlace", time = time)
            } else {
                t!(
                    "email.callout.inPerson.withPlace",
                    place = place,
                    time = time
                )
            }
        }
        MethodFields::BuildingStaff { .. } => t!("email.callout.buildingStaff", time = time),
        MethodFields::HostGreets { .. } => t!("email.callout.hostGreets", time = time),
        MethodFields::Other {} => {
            if has_code {
                t!("email.callout.other.withCode", time = time)
            } else {
                t!("email.callout.other.withoutCode")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BuildingAccess, RevealPolicy};
    use chrono::TimeZone;
    use portaki_sdk::context::StayContext;
    use portaki_sdk::host::with_host;
    use portaki_test_utils::MockContext;
    use uuid::Uuid;

    fn keybox_config(policy: RevealPolicy) -> ModuleConfig {
        ModuleConfig {
            primary_method: PrimaryMethod::Keybox,
            method: MethodFields::Keybox {
                location: "Sous le pot".into(),
                code: Some("4821".into()),
            },
            building_access: Some(BuildingAccess {
                gate_code: Some("A17B".into()),
                intercom: None,
            }),
            reveal_policy: policy,
            ..ModuleConfig::default()
        }
    }

    fn fr_email_host() -> MockContext {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_translation("email.label.keybox", "Code boîte à clés")
            .with_translation(
                "email.callout.keybox.withCode",
                "Arrivée autonome : le code ci-dessous ouvre la boîte à clés dès {time}. Pas besoin d'attendre l'hôte.",
            )
            .with_translation(
                "email.callout.locked.selfCheckin",
                "Arrivée autonome dès {time}. {locked} Les détails d'accès sont sur votre page de séjour.",
            )
            .with_translation("reveal.locked.withWhen", "Disponible à partir du {when}")
            .with_translation(
                "reveal.availableFrom.datetime",
                "{day}/{month}/{year} à {hour}:{minute}",
            )
    }

    #[test]
    #[serial_test::serial]
    fn revealed_keybox_returns_code_and_callout() {
        let (ctx, host) = fr_email_host()
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::Always)).expect("json"),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    checkin_time_formatted: Some("16:00".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("ok");
            assert!(response.secrets_revealed);
            assert_eq!(response.entry_access_code.as_deref(), Some("4821"));
            assert_eq!(
                response.access_code_label.as_deref(),
                Some("Code boîte à clés")
            );
            assert!(response
                .arrival_callout
                .as_deref()
                .unwrap_or("")
                .contains("boîte à clés"));
            assert!(response
                .arrival_callout
                .as_deref()
                .unwrap_or("")
                .contains("16:00"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn locked_policy_omits_plaintext_code() {
        let (mut ctx, host) = fr_email_host()
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::AtCheckin)).expect("json"),
            )
            .build();
        ctx.timezone = "Europe/Paris".into();
        ctx.property.timezone = "Europe/Paris".into();
        ctx.stay = Some(StayContext {
            stay_id: Uuid::nil(),
            checkin_at: Some(
                Utc.with_ymd_and_hms(2099, 1, 1, 15, 0, 0)
                    .single()
                    .expect("dt"),
            ),
            checkout_at: None,
            booking_channel: None,
        });

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    checkin_time_formatted: Some("16:00".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("ok");
            assert!(!response.secrets_revealed);
            assert!(response.entry_access_code.is_none());
            assert!(response
                .arrival_callout
                .as_deref()
                .unwrap_or("")
                .contains("Disponible"));
            assert!(!response
                .arrival_callout
                .as_deref()
                .unwrap_or("")
                .contains("4821"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn arrival_day_returns_code_without_callout() {
        let (ctx, host) = fr_email_host()
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::Always)).expect("json"),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some(EmailTemplateKey::ArrivalDay),
                    checkin_time_formatted: Some("16:00".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("ok");
            assert!(response.arrival_callout.is_none());
            assert_eq!(response.entry_access_code.as_deref(), Some("4821"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn stay_link_returns_empty_access_fields() {
        let (ctx, host) = fr_email_host()
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::Always)).expect("json"),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some(EmailTemplateKey::StayLink),
                    checkin_time_formatted: Some("16:00".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("ok");
            assert!(response.arrival_callout.is_none());
            assert!(response.entry_access_code.is_none());
            assert!(response.access_code_label.is_none());
        });
    }
}
