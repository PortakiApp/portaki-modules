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
