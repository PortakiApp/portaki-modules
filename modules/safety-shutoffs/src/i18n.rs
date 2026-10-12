//! Les textes que la plateforme reçoit dans les deux langues à la fois, lus dans les bundles.

use portaki_sdk::contracts::i18n::I18nText;

const BUNDLES: &[(&str, &str)] = &[
    ("fr", include_str!("../i18n/fr-FR.json")),
    ("en", include_str!("../i18n/en-US.json")),
];

pub fn text(key: &str) -> I18nText {
    text_with(key, &[])
}

pub fn text_with(key: &str, vars: &[(&str, &str)]) -> I18nText {
    I18nText::from_bundles(BUNDLES, key, vars)
}

/// « 1 emplacement vide ne sera pas publié. » / « 3 emplacements vides ne seront pas publiés. »
pub fn empty_rows(count: usize) -> I18nText {
    let key = if count == 1 {
        "host.shutoffs.empty.one"
    } else {
        "host.shutoffs.empty.other"
    };
    text_with(key, &[("count", &count.to_string())])
}
