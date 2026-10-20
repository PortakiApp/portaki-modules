//! The module's e-mails: to the host when a guest asks, to the guest when the host replies.
//!
//! Compile-time email locale bundles (`email_i18n/*.json`): the host's e-mails in French and
//! English, the guest's reply in every booklet language. Guest-typed text is quoted within a
//! fixed length (`host::email::quote_guest`) — the request keeps it whole.

pub mod bundles;
pub mod to_guest;
pub mod to_host;

use portaki_sdk::host::email::{LocalizedEmailText, ModuleEmailCta};

pub(crate) use bundles::{text, text_with};

/// Longest item label in the request lines, in chars.
const ITEM_ROW_MAX_CHARS: usize = 120;

/// « Voir plus » when a quote was cut, the email's own label otherwise.
fn cta(cut: bool, own: LocalizedEmailText, action: Option<&str>) -> ModuleEmailCta {
    ModuleEmailCta {
        label: if cut { text("email.cta.seeMore") } else { own },
        url: None,
        portaki_action: action.map(str::to_string),
    }
}
