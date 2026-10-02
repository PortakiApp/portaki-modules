//! Ce que les sections qui appellent un fournisseur (Tiqets, Viator) partagent : leur cache KV,
//! le constat « appelé sans clé » montré à l'hôte, et la langue de contenu demandée.

use portaki_sdk::host;
use portaki_sdk::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

/// Durée d'un constat de clé manquante : passé ce délai, l'écran de l'hôte cesse de le dire.
const NOTE_SECS: u32 = 24 * 60 * 60;

pub(crate) fn read<T: DeserializeOwned>(key: &str) -> Option<T> {
    let bytes = host::kv::get(key).ok()??;
    serde_json::from_slice(&bytes).ok()
}

/// Le KV oublie de lui-même l'entrée au bout de `ttl_secs` — ce que le fournisseur interdit de garder.
pub(crate) fn write<T: Serialize>(key: &str, value: &T, ttl_secs: i64) -> Result<()> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| PortakiError::Storage(format!("{key} serialize: {error}")))?;
    host::kv::set(key, &bytes, Some(ttl_secs as u32))
}

/// L'appel a manqué de clé (`connector_credential_missing`) : noté pour l'hôte, pour un jour.
pub(crate) fn note_missing_key(key: &str) {
    let _ = host::kv::set(key, b"1", Some(NOTE_SECS));
}

pub(crate) fn clear_missing_key(key: &str) {
    let _ = host::kv::delete(key);
}

pub(crate) fn missing_key(key: &str) -> bool {
    matches!(host::kv::get(key), Ok(Some(_)))
}

/// `fr` pour `fr-FR` ; `fr` quand la locale est vide.
pub(crate) fn lang_code(locale: &str) -> String {
    match locale.trim().split(['-', '_']).next() {
        Some(code) if !code.is_empty() => code.to_ascii_lowercase(),
        _ => "fr".to_string(),
    }
}

/// La langue du livret si le fournisseur la sert, l'anglais sinon.
pub(crate) fn served_lang(locale: &str, served: &[&str]) -> String {
    let code = lang_code(locale);
    if served.contains(&code.as_str()) {
        code
    } else {
        "en".to_string()
    }
}
