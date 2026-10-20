//! `publishReadiness` : ce qui bloque la publication.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::Lists;

/// Ce que le SDK relève, plus l'avertissement d'un code écrit en clair dans une étape du livret :
/// il se lit sans calendrier de révélation (spec Checklist §3). Il ne bloque pas.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let lists = Lists::load(&ctx)?;
    let mut readiness = portaki_sdk::resource::readiness(&ctx, &lists)?;
    for list in lists.rows().iter().filter(|list| list.is_guest()) {
        let coded = list.steps.iter().any(|step| {
            step.label
                .by_language()
                .chain(step.detail.by_language())
                .any(|(_, text)| looks_like_code(text))
        });
        if coded {
            let name = list.name.get("fr").to_string();
            readiness.items.push(PublishCheck {
                id: format!("code.{}", list.id),
                level: PublishLevel::Recommended,
                ok: false,
                label: portaki_sdk::bundle_text!("publish.steps.label", &[("list", name.as_str())]),
                hint: portaki_sdk::bundle_text!("publish.code.hint"),
            });
        }
    }
    Ok(readiness)
}

/// « Code 4821 », « digicode : 1234A » : le mot, ou une suite de quatre chiffres et plus.
///
/// ponytail: heuristique — un numéro de rue à quatre chiffres déclenche aussi l'avertissement,
/// qui ne bloque pas ; un code en lettres passe. Assez pour le cas que la spec décrit.
pub(crate) fn looks_like_code(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("code") {
        return true;
    }
    let mut run = 0;
    for c in lower.chars() {
        run = if c.is_ascii_digit() { run + 1 } else { 0 };
        if run >= 4 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::looks_like_code;

    #[test]
    fn a_code_is_spotted_by_its_word_or_its_digits() {
        assert!(looks_like_code("Code 4821"));
        assert!(looks_like_code("Boîte à clés : 0512"));
        assert!(looks_like_code("Le digicode est sur la porte"));
        assert!(!looks_like_code("Sortir les 3 poubelles"));
        assert!(!looks_like_code(""));
    }
}
