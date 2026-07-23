//! Guest-email snippets for Portaki templates (`arrival`, `arrival-day`, `new-code`).
//!
//! Reveal policy is applied here — same rules as guest SDUI. Never return plaintext
//! codes while locked.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, DoorCodeTarget, MethodFields, ModuleConfig, PrimaryMethod};
use crate::reveal::{evaluate_reveal, format_available_from, locked_message, RevealDecision};

/// Arguments for `emailContext`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmailContextArgs {
    /// Portaki template key (`arrival`, `arrival-day`, `new-code`, …).
    #[serde(default)]
    pub template_key: Option<String>,
    /// Formatted check-in clock time for callout copy (`16:00`).
    #[serde(default)]
    pub checkin_time_formatted: Option<String>,
    /// Optional locale override (BCP-47). Falls back to `ctx.locale`.
    #[serde(default)]
    pub locale: Option<String>,
}

/// Email-ready access-guide contribution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
    let locale = args
        .locale
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(ctx.locale.as_str());
    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let now = time::now().unwrap_or_else(|_| Utc::now());
    let decision = evaluate_reveal(config.reveal_policy, now, checkin_at, &property_timezone);

    let template = args.template_key.as_deref().unwrap_or("").trim();
    let wants_callout = matches!(template, "arrival" | "new-code" | "");
    let wants_code = wants_callout || template == "arrival-day";

    if !wants_callout && !wants_code {
        return Ok(EmailContextResponse::empty(
            decision.revealed,
            decision.available_from,
        ));
    }

    let time_label = checkin_time_label(args.checkin_time_formatted.as_deref(), locale);
    let plaintext_code = if decision.revealed {
        extract_entry_code(&config)
    } else {
        None
    };
    let has_configured_code = extract_entry_code(&config).is_some();
    let label = if wants_code && (decision.revealed || has_configured_code) {
        Some(access_code_label(&config, locale))
    } else {
        None
    };

    let callout = if wants_callout {
        Some(arrival_callout(
            &config,
            locale,
            &time_label,
            plaintext_code.is_some(),
            &decision,
            &property_timezone,
        ))
    } else {
        None
    };

    Ok(EmailContextResponse {
        arrival_callout: callout.filter(|s| !s.trim().is_empty()),
        entry_access_code: if wants_code {
            plaintext_code
        } else {
            None
        },
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

fn checkin_time_label(formatted: Option<&str>, locale: &str) -> String {
    let trimmed = formatted.map(str::trim).unwrap_or("");
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    if locale_is_en(locale) {
        "check-in time".into()
    } else {
        "l'heure d'arrivée".into()
    }
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

fn access_code_label(config: &ModuleConfig, locale: &str) -> String {
    let en = locale_is_en(locale);
    match &config.method {
        MethodFields::Keybox { .. } => {
            if en {
                "Keybox code".into()
            } else {
                "Code boîte à clés".into()
            }
        }
        MethodFields::DoorCode { target, .. } => door_code_label(*target, en),
        MethodFields::SmartLock { .. } => {
            if en {
                "Access code".into()
            } else {
                "Code d'accès".into()
            }
        }
        _ => {
            if en {
                "Access code".into()
            } else {
                "Code d'accès".into()
            }
        }
    }
}

fn door_code_label(target: DoorCodeTarget, en: bool) -> String {
    match (en, target) {
        (true, DoorCodeTarget::Gate) => "Gate code".into(),
        (false, DoorCodeTarget::Gate) => "Code portail".into(),
        (true, DoorCodeTarget::Building) => "Building code".into(),
        (false, DoorCodeTarget::Building) => "Code immeuble".into(),
        (true, DoorCodeTarget::Apartment) => "Apartment code".into(),
        (false, DoorCodeTarget::Apartment) => "Code appartement".into(),
    }
}

fn arrival_callout(
    config: &ModuleConfig,
    locale: &str,
    time: &str,
    has_revealed_code: bool,
    decision: &RevealDecision,
    property_timezone: &str,
) -> String {
    let en = locale_is_en(locale);
    let method = config.method.primary_method();

    // Timed lock: keep method framing, never promise "the code below".
    if !decision.revealed && extract_entry_code(config).is_some() {
        let when = decision
            .available_from
            .map(|at| format_available_from(at, property_timezone, locale));
        let locked = locked_message(locale, when.as_deref());
        return match method {
            PrimaryMethod::Keybox | PrimaryMethod::DoorCode | PrimaryMethod::SmartLock => {
                if en {
                    format!(
                        "Self check-in from {time}. {locked} Full access details are on your stay page."
                    )
                } else {
                    format!(
                        "Arrivée autonome dès {time}. {locked} Les détails d'accès sont sur votre page de séjour."
                    )
                }
            }
            _ => method_callout(config, en, time, false),
        };
    }

    method_callout(config, en, time, has_revealed_code)
}

fn method_callout(config: &ModuleConfig, en: bool, time: &str, has_code: bool) -> String {
    match &config.method {
        MethodFields::Keybox { .. } => {
            if has_code {
                if en {
                    format!(
                        "Self check-in: the code below opens the keybox from {time}. No need to wait for the host."
                    )
                } else {
                    format!(
                        "Arrivée autonome : le code ci-dessous ouvre la boîte à clés dès {time}. Pas besoin d'attendre l'hôte."
                    )
                }
            } else if en {
                format!(
                    "Self check-in from {time}. Access instructions are on your stay page."
                )
            } else {
                format!(
                    "Arrivée autonome dès {time}. Les instructions d'accès sont sur votre page de séjour."
                )
            }
        }
        MethodFields::DoorCode { .. } => {
            if has_code {
                if en {
                    format!(
                        "Self check-in: use the code below from {time}. No need to wait for the host."
                    )
                } else {
                    format!(
                        "Arrivée autonome : utilisez le code ci-dessous dès {time}. Pas besoin d'attendre l'hôte."
                    )
                }
            } else if en {
                format!(
                    "Self check-in from {time}. Access details are on your stay page."
                )
            } else {
                format!(
                    "Arrivée autonome dès {time}. Retrouvez les détails d'accès sur votre page de séjour."
                )
            }
        }
        MethodFields::SmartLock { .. } => {
            if has_code {
                if en {
                    format!("Self check-in: the code below unlocks the door from {time}.")
                } else {
                    format!(
                        "Arrivée autonome : le code ci-dessous déverrouille la serrure dès {time}."
                    )
                }
            } else if en {
                format!(
                    "Self check-in via smart lock from {time}. Open your stay page to unlock."
                )
            } else {
                format!(
                    "Arrivée autonome via serrure connectée dès {time}. Ouvrez votre page de séjour pour déverrouiller."
                )
            }
        }
        MethodFields::InPerson { meeting_place, .. } => {
            let place = meeting_place.trim();
            if place.is_empty() {
                if en {
                    format!("In-person handover on arrival (from {time}).")
                } else {
                    format!("Accueil en personne à votre arrivée (dès {time}).")
                }
            } else if en {
                format!("Meet at « {place} » from {time} to collect the keys.")
            } else {
                format!("Rendez-vous à « {place} » dès {time} pour récupérer les clés.")
            }
        }
        MethodFields::BuildingStaff { .. } => {
            if en {
                format!("Check in at the building desk from {time} to get the keys.")
            } else {
                format!("Présentez-vous à l'accueil du bâtiment dès {time} pour obtenir les clés.")
            }
        }
        MethodFields::HostGreets { .. } => {
            if en {
                format!("Your host greets you on arrival (from {time}).")
            } else {
                format!("Votre hôte vous accueille à l'arrivée (dès {time}).")
            }
        }
        MethodFields::Other {} => {
            if has_code {
                if en {
                    format!("Here is your access code, valid from {time}.")
                } else {
                    format!("Voici votre code d'accès, valable dès {time}.")
                }
            } else if en {
                "Access details are on your stay page.".into()
            } else {
                "Retrouvez les infos d'accès sur votre page de séjour.".into()
            }
        }
    }
}

fn locale_is_en(locale: &str) -> bool {
    locale.to_ascii_lowercase().starts_with("en")
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

    #[test]
    #[serial_test::serial]
    fn revealed_keybox_returns_code_and_callout() {
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&["core.storage"])
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::Always)).expect("json"),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some("arrival".into()),
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
        let (mut ctx, host) = MockContext::guest()
            .with_capabilities(&["core.storage"])
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
        });

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some("arrival".into()),
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
        let (ctx, host) = MockContext::guest()
            .with_capabilities(&["core.storage"])
            .with_kv(
                "config",
                serde_json::to_vec(&keybox_config(RevealPolicy::Always)).expect("json"),
            )
            .build();

        with_host(host, ctx.clone(), || {
            let response = build_email_context(
                &ctx,
                &EmailContextArgs {
                    template_key: Some("arrival-day".into()),
                    checkin_time_formatted: Some("16:00".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("ok");
            assert!(response.arrival_callout.is_none());
            assert_eq!(response.entry_access_code.as_deref(), Some("4821"));
        });
    }
}
