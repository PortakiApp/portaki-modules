//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let tz = PropertyTz::parse(&ctx.timezone);
    let today = time::now()
        .ok()
        .map(|now| crate::schedule::local_time(now, tz.as_ref()).date());
    // Les erreurs bloquent ; une date passée ou des horaires en texte libre se signalent seulement.
    let check = |level: PublishLevel| {
        move |(field, hint): (String, _)| PublishCheck {
            label: crate::i18n::text(field_label(&field)),
            id: format!("config.{field}"),
            level,
            ok: false,
            hint,
        }
    };
    let items = config
        .problems()
        .into_iter()
        .map(check(PublishLevel::Required))
        .chain(
            config
                .warnings(today)
                .into_iter()
                .map(check(PublishLevel::Recommended)),
        )
        .collect();
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "card_limit" => "host.cardLimit.label",
        "title" => "host.facility.name",
        "note" => "host.facility.note",
        "range_1" => "host.facility.range.1",
        "range_2" => "host.facility.range.2",
        "range_3" => "host.facility.range.3",
        "week" => "host.facility.week",
        "season_from" => "host.facility.season.from",
        "season_to" => "host.facility.season.to",
        "date" => "host.exception.date",
        "range" => "host.exception.range",
        "reason" => "host.exception.reason",
        "hours" => "host.facility.hours",
        _ => "host.facilities.title",
    }
}
