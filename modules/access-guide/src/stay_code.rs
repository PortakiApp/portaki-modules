//! « À chaque séjour » (spec Accès §2.2) : le code d'entrée d'un séjour, saisi par l'hôte dans la
//! fiche du séjour, gardé par la plateforme (`host::stay_secrets`) — jamais dans le KV ni la
//! config : un code changé pour un séjour doit atteindre son voyageur tout de suite, sans
//! « Publier », et disparaît avec le séjour.
//!
//! Aucun code n'est inventé au premier affichage : une boîte à clés ou un digicode ne connaît que
//! le code que l'hôte y a programmé. Sans code, le voyageur lit « Votre hôte vous communiquera le
//! code » et l'hôte a sa tâche « Code d'entrée manquant » (§9 cas 2).

use portaki_sdk::host::{random, stay_secrets};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// La clé du code d'entrée dans les secrets du séjour.
pub const KEY: &str = "entry_code";

/// La longueur d'un code tiré au hasard par « Générer ».
const GENERATED_DIGITS: u32 = 6;

/// Le code du séjour à la place du code fixe, quand l'hôte a choisi « À chaque séjour ». Sans
/// séjour (un aperçu) ou sans code saisi : aucun code — jamais le code fixe d'avant.
pub fn fill(config: &mut ModuleConfig, stay: Option<Uuid>) {
    if !config.code_per_stay {
        return;
    }
    let code = stay.and_then(|stay| read(stay).ok().flatten());
    config.set_entry_code(code);
}

/// Le code de ce séjour, s'il est saisi.
pub fn read(stay: Uuid) -> Result<Option<String>> {
    Ok(stay_secrets::get(stay, KEY)?.filter(|code| !code.trim().is_empty()))
}

/// 3 à 12 caractères : chiffres, lettres, `#` ou `*` (§2.2) — le code nettoyé de ses espaces.
pub fn normalize(raw: &str) -> Option<String> {
    let code: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let ok = (3..=12).contains(&code.chars().count())
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '#' || c == '*');
    ok.then_some(code)
}

/// Six chiffres du générateur de l'hôte, jamais d'un pseudo-aléa du module.
pub fn generate() -> Result<String> {
    let code = random::u32_below(10u32.pow(GENERATED_DIGITS))?;
    Ok(format!("{code:0width$}", width = GENERATED_DIGITS as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_is_three_to_twelve_keypad_characters() {
        assert_eq!(normalize(" 48 21 ").as_deref(), Some("4821"));
        assert_eq!(normalize("A1#*").as_deref(), Some("A1#*"));
        assert_eq!(normalize("12"), None);
        assert_eq!(normalize("1234567890123"), None);
        assert_eq!(normalize("12-34"), None);
        assert_eq!(normalize("çà1"), None);
    }
}
