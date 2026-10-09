//! Module queries — publication readiness.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

/// A guest needs a keypad code, or remote unlock: the Nuki Web key (a connector the host adds
/// in Integrations) plus the lock ID. A rule across the config and a grant, which the declared
/// config cannot say.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let remote = crate::commands::has_nuki_byok(&ctx) && !config.smartlock_id_trimmed().is_empty();
    let mut items = vec![PublishCheck {
        id: "guest-access".into(),
        level: PublishLevel::Required,
        ok: !config.keypad_code_trimmed().is_empty() || remote,
        label: text("publish.access.label"),
        hint: text("publish.access.hint"),
    }];
    items.extend(
        config
            .problems()
            .into_iter()
            .map(|(field, error)| PublishCheck {
                id: format!("config.{field}"),
                level: PublishLevel::Required,
                ok: false,
                label: text(match field {
                    "keypad_code" => "host.keypadCode.label",
                    _ => "host.deviceName.label",
                }),
                hint: error,
            }),
    );
    Ok(PublishReadiness { items })
}
