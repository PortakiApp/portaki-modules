//! Text sent to the platform in every language at once (tasks, tiles, templates), read from the
//! bundles.
//!
//! Les dix bundles sont ici, pas seulement `fr` et `en` : ce que ce module en tire part dans le
//! livret (le nom d'une liste sert de titre de groupe au voyageur), et deux langues y donnaient un
//! titre français à un voyageur japonais.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::email::LocalizedEmailText;

use crate::labels::Labels;

const BUNDLES: &[(&str, &str)] = &[
    ("fr", include_str!("../i18n/fr-FR.json")),
    ("en", include_str!("../i18n/en-US.json")),
    ("es", include_str!("../i18n/es-ES.json")),
    ("de", include_str!("../i18n/de-DE.json")),
    ("it", include_str!("../i18n/it-IT.json")),
    ("pt", include_str!("../i18n/pt-PT.json")),
    ("nl", include_str!("../i18n/nl-NL.json")),
    ("ja", include_str!("../i18n/ja-JP.json")),
    ("zh", include_str!("../i18n/zh-CN.json")),
    ("ar", include_str!("../i18n/ar-SA.json")),
];

fn localized(key: &str, vars: &[(&str, &str)]) -> LocalizedEmailText {
    LocalizedEmailText::from_i18n_key_with_vars(BUNDLES.iter().copied(), key, vars)
}

pub fn text(key: &str, vars: &[(&str, &str)]) -> I18nText {
    let text = localized(key, vars);
    let mut out = I18nText::new(text.fr, text.en);
    out.others = text.translations;
    out
}

/// La même clé, en carte de langues — la forme que les colonnes `*_fr` stockent.
pub fn labels(key: &str, vars: &[(&str, &str)]) -> Labels {
    let text = localized(key, vars);
    let mut out = Labels::new();
    let pairs = [("fr".to_string(), text.fr), ("en".to_string(), text.en)]
        .into_iter()
        .chain(text.translations);
    for (lang, value) in pairs {
        if !value.trim().is_empty() {
            out.insert(crate::labels::lang_code(&lang), value);
        }
    }
    out
}
