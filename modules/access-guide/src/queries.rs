//! Module queries — the publication check the declared config cannot express.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{MethodFields, ModuleConfig};
use crate::i18n::text;

/// A code-bearing access method needs its code — the platform already blocks on the method.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let host = crate::config::HostConfig::load(&ctx)?;
    let mut items: Vec<PublishCheck> = host
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            label: text(field_label(&field)),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();
    // Dès la réservation : le code se lit même si le séjour est annulé ensuite (§3).
    if host.reveal_policy.trim() == "always" {
        items.push(PublishCheck {
            id: "config.reveal_policy".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: text("config.revealPolicy"),
            hint: text("publish.reveal.always"),
        });
    }

    let config = ModuleConfig::read(&ctx)?;
    let ok = match &config.method {
        MethodFields::Keybox { .. } => config.keybox_code().is_some(),
        MethodFields::DoorCode { code, .. } => !code.trim().is_empty(),
        // A provider module issues the codes; the manual one is only a fallback.
        MethodFields::SmartLock { .. } => {
            config.smart_lock_manual_code().is_some()
                || config
                    .smart_lock_provider_module_id
                    .as_deref()
                    .is_some_and(|id| !id.trim().is_empty())
        }
        _ => return Ok(PublishReadiness { items }),
    };
    items.insert(
        0,
        PublishCheck {
            id: "entry-code".into(),
            level: PublishLevel::Required,
            ok,
            label: text("publish.entry-code.label"),
            hint: text("publish.entry-code.hint"),
        },
    );
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de l'étape.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "method_instructions" => "config.methodInstructions",
        "keybox_location" => "host.keybox.location",
        "global_note" => "config.globalNote",
        "late_arrival_note" => "config.lateArrivalNote",
        "parking_info" => "config.parkingInfo",
        "title" => "host.step.title",
        "detail" => "host.step.detail",
        _ => "host.steps.label",
    }
}
