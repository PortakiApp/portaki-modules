//! Module queries — la porte de publication.

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::i18n::I18nText;
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
        label: bundle_text!("publish.shutoffs.label"),
        hint: bundle_text!("host.shutoffs.required"),
    }];

    // Une ligne sans emplacement ne s'affiche pas : « N emplacements vides ne seront pas
    // publiés. » (§3). Avertit, ne bloque pas.
    let incomplete = config.shutoffs_incomplete();
    if incomplete > 0 {
        items.push(PublishCheck {
            id: "incomplete".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: bundle_text!("publish.incomplete.label"),
            hint: empty_rows(incomplete),
        });
    }

    // Un emplacement manquant n'est pas bloquant : la ligne est écartée, et l'avertissement
    // ci-dessus le dit (§3). Les autres erreurs du formulaire bloquent.
    let missing_location = |field: &str| {
        field
            .strip_prefix("shutoffs.")
            .and_then(|rest| rest.strip_suffix(".location"))
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| config.rows().get(index))
            .is_some_and(|row| row.location.is_blank())
    };
    let blocking: Vec<_> = config
        .problems()
        .into_iter()
        .filter(|(field, _)| !missing_location(field))
        .collect();
    items.extend(blocking.into_iter().map(|(field, error)| PublishCheck {
        label: bundle_text!(field_label(&field)),
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
        "kind" => "host.kind.label",
        "title" => "host.shutoffs.rowTitle",
        "location" => "host.shutoffs.location",
        "instruction" => "host.shutoffs.instruction",
        _ => "publish.shutoffs.label",
    }
}

/// « 1 emplacement vide… » / « N emplacements vides… ».
pub(crate) fn empty_rows(count: usize) -> I18nText {
    let key = if count == 1 {
        "host.shutoffs.empty.one"
    } else {
        "host.shutoffs.empty.other"
    };
    bundle_text!(key, &[("count", &count.to_string())])
}
