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
            label: crate::i18n::text(field_label(&field)),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "card_limit" => "host.cardLimit.label",
        "title" => "host.facility.name",
        "note" => "host.facility.note",
        "opens_at" => "host.facility.opensAt",
        "closes_at" => "host.facility.closesAt",
        "break_from" => "host.facility.breakFrom",
        "break_to" => "host.facility.breakTo",
        "season_from" => "host.facility.season.from",
        "season_to" => "host.facility.season.to",
        _ => "host.facilities.title",
    }
}
