//! `publishReadiness` — the rules of the drawer, checked again before « Publier ». Each point
//! names its field (`config.networks[1].password`) for the dashboard to scroll to.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(PublishReadiness {
        items: config
            .problems()
            .into_iter()
            .map(|problem| PublishCheck {
                id: problem.id,
                level: if problem.blocking {
                    PublishLevel::Required
                } else {
                    PublishLevel::Recommended
                },
                ok: false,
                label: text(problem.label),
                hint: problem.message,
            })
            .collect(),
    })
}
