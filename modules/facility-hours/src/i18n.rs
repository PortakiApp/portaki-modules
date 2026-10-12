//! Text sent to the platform in both languages at once (field errors, readiness), read from the
//! bundles.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::email::LocalizedEmailText;

const BUNDLES: &[(&str, &str)] = &[
    ("fr", include_str!("../i18n/fr-FR.json")),
    ("en", include_str!("../i18n/en-US.json")),
];

pub fn text(key: &str) -> I18nText {
    let text = LocalizedEmailText::from_i18n_key_with_vars(BUNDLES.iter().copied(), key, &[]);
    I18nText::new(text.fr, text.en)
}

/// « Deux plages se chevauchent le samedi. », chaque langue avec son nom du jour.
pub fn overlap_on(day: chrono::Weekday) -> I18nText {
    let one = |lang: &str, bundle: &'static str| {
        // Minuscule en français (« le samedi »), majuscule en anglais (« on Saturday »).
        let name = portaki_sdk::host::time::weekday_name(day, lang);
        let name = if lang == "fr" {
            name.to_lowercase()
        } else {
            name.to_string()
        };
        LocalizedEmailText::from_i18n_key_with_vars(
            [(lang, bundle)],
            "host.facility.ranges.overlapOn",
            &[("day", name.as_str())],
        )
        .resolve(lang)
        .to_string()
    };
    I18nText::new(one("fr", BUNDLES[0].1), one("en", BUNDLES[1].1))
}
