//! Compile-time host email locale bundles (`email_i18n/*.json`).

use portaki_sdk::host::email::LocalizedEmailText;

const BUNDLES: &[(&str, &str)] = &[
    ("en", include_str!("../email_i18n/en.json")),
    ("fr", include_str!("../email_i18n/fr.json")),
];

pub fn text(key: &str) -> LocalizedEmailText {
    LocalizedEmailText::from_i18n_key(BUNDLES.iter().copied(), key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundles_expose_sync_failed_keys() {
        let subject = text("email.syncFailed.subject");
        assert!(!subject.fr.is_empty());
        assert!(!subject.en.is_empty());
        let body = LocalizedEmailText::from_i18n_key_with_vars(
            BUNDLES.iter().copied(),
            "email.syncFailed.body",
            &[
                ("property", "Chalet"),
                ("source", "Booking"),
                ("lastSuccess", "11 juil. 2026"),
                ("error", "404"),
            ],
        );
        assert!(body.fr.contains("Chalet"));
        assert!(body.en.contains("Chalet"));
    }
}
