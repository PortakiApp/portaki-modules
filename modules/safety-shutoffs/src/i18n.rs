//! Les textes que la plateforme reçoit dans les deux langues à la fois, lus dans les bundles.

use portaki_sdk::contracts::i18n::I18nText;

const BUNDLES: &[(&str, &str)] = &[
    ("fr", include_str!("../i18n/fr-FR.json")),
    ("en", include_str!("../i18n/en-US.json")),
];

pub fn text(key: &str) -> I18nText {
    I18nText::from_bundles(BUNDLES, key, &[])
}
