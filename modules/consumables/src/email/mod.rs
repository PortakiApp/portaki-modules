//! The module's e-mails: to the host when a guest asks, to the guest when the host replies.
//!
//! Compile-time email locale bundles (`email_i18n/*.json`): the host's e-mails in French and
//! English, the guest's reply in every booklet language.

pub mod bundles;
pub mod text;
pub mod to_guest;
pub mod to_host;

pub(crate) use bundles::{text, text_with};
