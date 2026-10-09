//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let items = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            label: crate::i18n::text(match field {
                "spot_label" => "host.spotLabel.label",
                "power_kw" => "host.power.label",
                "price" => "host.price.label",
                "instructions" => "host.instructions.label",
                _ => "host.bookingNote.label",
            }),
            hint: error,
        })
        .collect();
    Ok(PublishReadiness { items })
}
