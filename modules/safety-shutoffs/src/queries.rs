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

    // Une ligne commencée sans emplacement ne s'affiche pas : « N emplacements vides ne seront
    // pas publiés » (§3). Avertit ; l'erreur bloquante est sous le champ, plus bas.
    if config.shutoffs_incomplete() > 0 {
        items.push(PublishCheck {
            id: "incomplete".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("publish.incomplete.label"),
            hint: crate::i18n::text("publish.incomplete.hint"),
        });
    }

    // Un emplacement manquant n'est pas bloquant : la ligne est écartée, et l'avertissement
    // ci-dessus le dit (§3). Les autres erreurs du formulaire bloquent.
    let missing_location = |field: &str| {
        field
            .strip_prefix("shutoffs.")
            .and_then(|rest| rest.strip_suffix(".location"))
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| config.shutoffs.get(index))
            .is_some_and(|row| row.location.is_blank())
    };
    let blocking: Vec<_> = config
        .problems()
        .into_iter()
        .filter(|(field, _)| !missing_location(field))
        .collect();
    items.extend(blocking.into_iter().map(|(field, error)| PublishCheck {
        label: crate::i18n::text(field_label(&field)),
        id: format!("config.{field}"),
        level: PublishLevel::Required,
        ok: false,
        hint: error,
    }));

    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "general_note" => "host.note.label",
        "title" => "host.shutoffs.rowTitle",
        "location" => "host.shutoffs.location",
        "instruction" => "host.shutoffs.instruction",
        _ => "publish.shutoffs.label",
    }
}
