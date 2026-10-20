//! Compile-time email locale bundles (`email_i18n/*.json`): the host's e-mails in French and
//! English, the guest's reply in every booklet language.

use portaki_sdk::host::email::LocalizedEmailText;

const BUNDLES: &[(&str, &str)] = &[
    ("en", include_str!("../../email_i18n/en.json")),
    ("fr", include_str!("../../email_i18n/fr.json")),
    ("es", include_str!("../../email_i18n/es.json")),
    ("de", include_str!("../../email_i18n/de.json")),
    ("it", include_str!("../../email_i18n/it.json")),
    ("pt", include_str!("../../email_i18n/pt.json")),
    ("nl", include_str!("../../email_i18n/nl.json")),
    ("zh", include_str!("../../email_i18n/zh.json")),
    ("ja", include_str!("../../email_i18n/ja.json")),
    ("ar", include_str!("../../email_i18n/ar.json")),
];

pub(crate) fn text(key: &str) -> LocalizedEmailText {
    LocalizedEmailText::from_i18n_key(BUNDLES.iter().copied(), key)
}

pub(crate) fn text_with(key: &str, vars: &[(&str, &str)]) -> LocalizedEmailText {
    LocalizedEmailText::from_i18n_key_with_vars(BUNDLES.iter().copied(), key, vars)
}
