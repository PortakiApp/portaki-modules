//! Section « Activités (Viator) » — produits Viator dans la ville du logement.
//!
//! Comme la section Tiqets, elle appelle un fournisseur : la recherche libre de l'API Partner
//! de Viator, par le connecteur `viator`, avec la ville de l'adresse du logement pour terme —
//! la même que celle de la section GetYourGuide. Lecture seule : le voyageur réserve sur
//! viator.com en suivant `product_url`, qui porte déjà les paramètres d'affiliation. Le module
//! ne le réécrit jamais.
//!
//! ADR-0021 : le connecteur est déclaré ici. Sans clé d'hôte (`host_key = false`) : la licence
//! Viator interdit de montrer son contenu sous une autre clé que celle du titulaire du domaine,
//! et le livret vit sur celui de Portaki. Seule la clé éditeur de Portaki sert donc ; quand elle
//! manque (`connector_credential_missing`), le module le note et l'écran de l'hôte le dit.
//!
//! Les résultats sont gardés en KV, une entrée par langue de livret :
//! - fraîche pendant [`FRESH_SECS`] (24 h) — au plus un appel par jour et par langue ;
//! - encore servie, si Viator ne répond pas, jusqu'à [`STALE_MAX_SECS`] (7 jours). Au-delà,
//!   la section disparaît plutôt que de montrer des prix d'il y a deux semaines.
//!
//! Sans horloge de l'hôte, rien n'est affiché : juger la fraîcheur sur une fausse heure
//! servirait indéfiniment le même cache.

use portaki_connectors::viator::{FreetextProductsArgs, Viator, ViatorProduct};
use portaki_sdk::host::{log, time};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::activities::destination_from_address;
use crate::config::ViatorConfig;

/// Fraîcheur d'une entrée du cache.
pub const FRESH_SECS: i64 = 24 * 60 * 60;

/// Âge au-delà duquel une entrée n'est plus servie, même Viator injoignable.
pub const STALE_MAX_SECS: i64 = 7 * 24 * 60 * 60;

/// Produits demandés, et gardés, par appel.
pub const MAX_PRODUCTS: usize = 12;

/// Produits montrés sur la carte d'accueil, qui reste un aperçu.
pub const HOME_PRODUCTS: usize = 3;

/// Devise des prix. La page Viator affiche ensuite celle du voyageur.
const CURRENCY: &str = "EUR";

/// Langues de contenu que Viator sert ; toute autre retombe sur l'anglais.
const VIATOR_LANGS: [&str; 12] = [
    "da", "de", "en", "es", "fr", "it", "ja", "ko", "nl", "no", "pt", "sv",
];

const CACHE_KEY_PREFIX: &str = "viator_cache.";

#[portaki_sdk::custom_connector(
    id = "viator",
    display_name_key = "connector.viator.name",
    base_url = "https://api.viator.com",
    auth = "header:exp-api-key",
    header = "Accept: application/json;version=2.0",
    header_arg = "Accept-Language=lang",
    host_key = false
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleViator;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleViator {
    #[portaki_sdk::connector_op(
        connector = "viator",
        method = "POST",
        path = "/partner/search/freetext",
        fields = "lang, searchTerm, currency, searchTypes, productFiltering",
        sends = "property_city"
    )]
    pub fn search_products() {}
}

/// La section une fois résolue.
#[derive(Debug, Clone, PartialEq)]
pub struct ViatorView {
    pub products: Vec<ViatorProduct>,
}

/// Noté quand Viator a été appelé sans la clé de Portaki ; effacé au premier appel qui aboutit.
const KEY_MISSING_KEY: &str = "viator_key_missing";

/// Pourquoi la section ne peut pas s'afficher, pour le dire à l'hôte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViatorStatus {
    Off,
    MissingKey,
    MissingCity,
    Ready,
}

/// Le terme de recherche : la ville de l'adresse du logement, `None` sans adresse exploitable.
fn property_city(ctx: &Context) -> Option<String> {
    ctx.property
        .address
        .as_deref()
        .and_then(destination_from_address)
}

pub fn status(ctx: &Context, config: &ViatorConfig) -> ViatorStatus {
    if !config.enabled {
        ViatorStatus::Off
    } else if crate::provider::missing_key(KEY_MISSING_KEY) {
        ViatorStatus::MissingKey
    } else if property_city(ctx).is_none() {
        ViatorStatus::MissingCity
    } else {
        ViatorStatus::Ready
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ViatorCache {
    search_term: String,
    min_rating: u8,
    lang: String,
    /// Secondes Unix, horloge de l'hôte.
    fetched_at: i64,
    products: Vec<ViatorProduct>,
}

/// Résout la section pour le voyageur, ou `None` — éteinte, sans clé, sans ville, sans
/// horloge, ou sans produit à montrer. Aucune de ces issues n'est une erreur de rendu.
pub fn resolve(ctx: &Context, config: &ViatorConfig) -> Option<ViatorView> {
    // Une clé manquante n'empêche pas d'essayer : Portaki peut l'avoir posée depuis.
    if !config.enabled {
        return None;
    }
    let search_term = property_city(ctx)?;
    let now = time::now().ok()?.timestamp();
    let lang = viator_lang(&ctx.locale);
    let min_rating = config.normalized_min_rating();

    let cached = read_cache(&lang).filter(|cache| {
        cache.search_term == search_term
            && cache.min_rating == min_rating
            && now - cache.fetched_at >= 0
    });
    if let Some(cache) = cached.as_ref() {
        if now - cache.fetched_at < FRESH_SECS {
            return view(cache.products.clone());
        }
    }

    let args = FreetextProductsArgs::new(
        &search_term,
        &lang,
        CURRENCY,
        MAX_PRODUCTS as u32,
        min_rating,
    );
    match Viator::search_products(&args) {
        Ok(response) => {
            crate::provider::clear_missing_key(KEY_MISSING_KEY);
            let mut products = response.products;
            products.truncate(MAX_PRODUCTS);
            let _ = write_cache(&ViatorCache {
                search_term,
                min_rating,
                lang,
                fetched_at: now,
                products: products.clone(),
            });
            view(products)
        }
        Err(error) => {
            if error.to_string().contains("connector_credential_missing") {
                // Dit à l'hôte que la clé de Portaki manque ; oublié de lui-même au bout d'un jour.
                crate::provider::note_missing_key(KEY_MISSING_KEY);
            }
            let mut fields = log::Fields::new();
            fields.insert("error", &error.to_string());
            let _ = log::warn("local_guide_viator_fetch_failed", &fields);
            // Viator injoignable, quota épuisé : l'entrée d'hier vaut mieux que rien, tant
            // qu'elle reste sous la limite de 7 jours.
            cached
                .filter(|cache| now - cache.fetched_at < STALE_MAX_SECS)
                .and_then(|cache| view(cache.products))
        }
    }
}

fn view(products: Vec<ViatorProduct>) -> Option<ViatorView> {
    if products.is_empty() {
        None
    } else {
        Some(ViatorView { products })
    }
}

/// La langue du livret si Viator la sert, l'anglais sinon.
pub fn viator_lang(locale: &str) -> String {
    crate::provider::served_lang(locale, &VIATOR_LANGS)
}

/// « 2 h », « 1 h 30 », « 45 min ».
pub fn format_duration(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m:02}"),
    }
}

fn cache_key(lang: &str) -> String {
    format!("{CACHE_KEY_PREFIX}{lang}")
}

fn read_cache(lang: &str) -> Option<ViatorCache> {
    crate::provider::read(&cache_key(lang))
}

fn write_cache(cache: &ViatorCache) -> Result<()> {
    crate::provider::write(&cache_key(&cache.lang), cache, STALE_MAX_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_booklet_language_is_kept_when_viator_serves_it() {
        assert_eq!(viator_lang("fr-FR"), "fr");
        assert_eq!(viator_lang("nb-NO"), "en");
        assert_eq!(viator_lang("no_NO"), "no");
        assert_eq!(viator_lang("uk-UA"), "en");
    }

    #[test]
    fn durations_read_in_hours_and_minutes() {
        assert_eq!(format_duration(120), "2 h");
        assert_eq!(format_duration(90), "1 h 30");
        assert_eq!(format_duration(65), "1 h 05");
        assert_eq!(format_duration(45), "45 min");
    }
}
