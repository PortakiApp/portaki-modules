//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use serde::{Deserialize, Serialize};

/// The keys are the names of the host form fields: the platform takes `updateConfig`.
/// Nothing is required on its own: « keypad code, or remote unlock (Nuki Web key + lock ID) »
/// spans the connector, so `publishReadiness` carries it. The platform does not trim: read
/// through the `*_trimmed` accessors.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(label = "host.smartlockId.label")]
    pub smartlock_id: String,
    #[field(secret, label = "host.keypadCode.label")]
    pub keypad_code: String,
    #[field(label = "host.deviceName.label")]
    pub device_name: String,
}

impl ModuleConfig {
    pub fn keypad_code_trimmed(&self) -> &str {
        self.keypad_code.trim()
    }

    pub fn smartlock_id_trimmed(&self) -> &str {
        self.smartlock_id.trim()
    }
}

impl ModuleConfig {
    /// Ce qui ne va pas, champ par champ — sous le champ, et dans `publishReadiness` (spec Nuki
    /// §2) : le code clavier fait six chiffres sans 0 en tête, le nom 40 caractères au plus.
    pub fn problems(&self) -> Vec<(&'static str, portaki_sdk::contracts::i18n::I18nText)> {
        let mut problems = Vec::new();
        let code = self.keypad_code_trimmed();
        let valid_code =
            code.len() == 6 && code.bytes().all(|b| b.is_ascii_digit()) && !code.starts_with('0');
        if !code.is_empty() && !valid_code {
            problems.push(("keypad_code", crate::i18n::text("host.keypadCode.invalid")));
        }
        if let Some(error) = portaki_sdk::config::check::max_chars(&self.device_name, 40) {
            problems.push(("device_name", error));
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<portaki_sdk::contracts::i18n::I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| *name == field)
            .map(|(_, error)| error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Six chiffres, sans 0 en tête : la règle du clavier Nuki. Vide n'est pas une erreur.
    #[test]
    fn the_keypad_code_follows_the_nuki_rule() {
        let with = |code: &str| ModuleConfig {
            keypad_code: code.into(),
            ..ModuleConfig::default()
        };
        assert!(with("").problems().is_empty());
        assert!(with("482913").problems().is_empty());
        for refused in ["048291", "4829", "48291a", "4829131"] {
            assert_eq!(with(refused).problems().len(), 1, "{refused}");
        }
    }
}
