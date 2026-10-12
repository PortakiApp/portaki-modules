//! Text sent to the platform in both languages at once, read from the bundles.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::email::LocalizedEmailText;

const BUNDLES: &[(&str, &str)] = &[
    ("fr", include_str!("../i18n/fr-FR.json")),
    ("en", include_str!("../i18n/en-US.json")),
];

pub fn text(key: &str) -> I18nText {
    let text = LocalizedEmailText::from_i18n_key(BUNDLES.iter().copied(), key);
    I18nText::new(text.fr, text.en)
}

/// Comme [`text`], avec des variables (`{bin}`).
pub fn text_with(key: &str, vars: &[(&str, &str)]) -> I18nText {
    let text = LocalizedEmailText::from_i18n_key_with_vars(BUNDLES.iter().copied(), key, vars);
    I18nText::new(text.fr, text.en)
}

/// Comme [`text_with`], la variable prise dans la langue du texte : « Bac jaune », « Yellow bin ».
/// Écrite en minuscules, au milieu d'une phrase.
pub fn text_with_text(key: &str, var: &str, value: &I18nText) -> I18nText {
    let in_language = |language: &str| {
        let value = value.get(language).to_lowercase();
        text_with(key, &[(var, value.as_str())])
            .get(language)
            .to_string()
    };
    I18nText::new(in_language("fr"), in_language("en"))
}
