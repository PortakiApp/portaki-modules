//! Module queries — la porte de publication.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Ce qu'il faut pour publier : un organe complet.
///
/// Requis, parce qu'un module « Coupures & sécurité » sans aucun organe promet au voyageur une
/// page qui ne dit rien. La vérification vit ici et non en `#[field(required)]` sur la liste : une
/// ligne vide remplit un champ obligatoire sans rien dire, et il faut savoir compter les complètes.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items = vec![PublishCheck {
        id: "shutoffs".into(),
        level: PublishLevel::Required,
        ok: !config.parse_shutoffs().is_empty(),
        label: crate::i18n::text("publish.shutoffs.label"),
        hint: crate::i18n::text("publish.shutoffs.hint"),
    }];

    // Un titre sans emplacement ne s'affiche pas : l'hôte a commencé une ligne et croit l'avoir
    // écrite. Recommandé, pas requis — le reste du module marche.
    if config.shutoffs_incomplete() > 0 {
        items.push(PublishCheck {
            id: "incomplete".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("publish.incomplete.label"),
            hint: crate::i18n::text("publish.incomplete.hint"),
        });
    }

    Ok(PublishReadiness { items })
}
