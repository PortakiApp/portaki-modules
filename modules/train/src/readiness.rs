//! `publishReadiness` — the rules of the drawer, checked again before « Publier ».

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{ModuleConfig, MAX_DESTINATIONS};
use crate::i18n::text;
use crate::sncf::{self, BoardError};

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items = Vec::new();
    // Une gare que la source ne trouve pas : le voyageur n'aurait jamais de tableau. Une source
    // muette (clé absente, panne) ne bloque pas — ce n'est pas la faute du nom.
    match config.station_name().map(sncf::station) {
        Some(Err(BoardError::UnknownStation)) => items.push(blocking(
            "config.station",
            "config.station",
            "config.station.unknown",
        )),
        // Une gare loin du logement se publie quand même : l'hôte peut avoir ses raisons (§9 #4).
        Some(Ok(station)) if crate::access::too_far(&station, ctx.property.coordinates) => items
            .push(PublishCheck {
                id: "config.station.far".into(),
                level: PublishLevel::Recommended,
                ok: false,
                label: text("config.station"),
                hint: text("host.station.far.message"),
            }),
        _ => {}
    }
    if config.proposed_destinations().len() > MAX_DESTINATIONS {
        items.push(blocking(
            "config.destinations",
            "config.destinations",
            "config.destinations.max",
        ));
    }
    Ok(PublishReadiness { items })
}

fn blocking(id: &str, label: &str, hint: &str) -> PublishCheck {
    PublishCheck {
        id: id.into(),
        level: PublishLevel::Required,
        ok: false,
        label: text(label),
        hint: text(hint),
    }
}
