//! Compile-time host email locale bundles (`email_i18n/*.json`).

use portaki_sdk::host::email::LocalizedEmailText;

const BUNDLES: &[(&str, &str)] = &[
    ("en", include_str!("../email_i18n/en.json")),
    ("fr", include_str!("../email_i18n/fr.json")),
];

pub fn text(key: &str) -> LocalizedEmailText {
    LocalizedEmailText::from_i18n_key(BUNDLES.iter().copied(), key)
}
