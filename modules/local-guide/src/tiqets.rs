//! Section « Billets & activités (Tiqets) » — produits Tiqets autour du logement.
//!
//! Là où la section GetYourGuide n'appelle personne et ne produit que des liens, celle-ci
//! interroge l'API Content de Tiqets par le connecteur `tiqets` : titre, image, prix et note des
//! billets à proximité. Lecture seule — la clé partenaire ne couvre que le contenu et les
//! disponibilités. Le voyageur réserve chez Tiqets, en suivant `product_url`, qui porte déjà le
//! code d'affiliation de la clé qui a fait l'appel : celle de Portaki (pool), ou celle de l'hôte
//! (BYOK). Le module ne le réécrit jamais.
//!
//! Les résultats sont gardés en KV, une entrée par langue de livret :
//! - fraîche pendant [`FRESH_SECS`] (24 h) — un logement fait donc au plus un appel par jour et
//!   par langue, loin sous les 15 requêtes par seconde de Tiqets et le quota mensuel du pool ;
//! - encore servie, si Tiqets ne répond pas, jusqu'à [`STALE_MAX_SECS`] (14 jours) : Tiqets
//!   exige de rafraîchir les images au moins tous les 14 jours (crédits changés, images
//!   dépubliées). Au-delà, la section disparaît plutôt que de montrer une image périmée.
//!
//! Sans horloge de l'hôte, rien n'est affiché : juger la fraîcheur sur une fausse heure
//! servirait indéfiniment un cache que Tiqets interdit de garder.

use portaki_connectors::tiqets::{NearbyProductsArgs, Tiqets, TiqetsProduct};
use portaki_sdk::host::{self, log, time};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{valid_coords, Localized, TiqetsConfig};

/// Fraîcheur d'une entrée du cache.
pub const FRESH_SECS: i64 = 24 * 60 * 60;

/// Âge au-delà duquel une entrée n'est plus servie, même Tiqets injoignable.
pub const STALE_MAX_SECS: i64 = 14 * 24 * 60 * 60;

/// Produits demandés, et gardés, par appel.
pub const MAX_PRODUCTS: usize = 12;

/// Produits montrés sur la carte d'accueil, qui reste un aperçu.
pub const HOME_PRODUCTS: usize = 3;

/// Devise des prix. Le lien Tiqets affiche ensuite celle du navigateur du voyageur.
const CURRENCY: &str = "EUR";

/// Langues de contenu que Tiqets sert ; toute autre retombe sur l'anglais, comme chez eux.
const TIQETS_LANGS: [&str; 17] = [
    "ca", "cs", "da", "de", "el", "en", "es", "fr", "it", "ja", "ko", "nl", "pl", "pt", "ru", "sv",
    "zh",
];

const CACHE_KEY_PREFIX: &str = "tiqets_cache.";

#[portaki_sdk::custom_connector(
    id = "tiqets",
    display_name_key = "connector.tiqets.name",
    base_url = "https://api.tiqets.com",
    credential_provider_id = "tiqets"
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleTiqets;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleTiqets {
    #[portaki_sdk::connector_op(method = "GET", path = "/v2/products", cache = "24h")]
    pub fn nearby_products() {}
}

/// La section une fois résolue.
#[derive(Debug, Clone, PartialEq)]
pub struct TiqetsView {
    pub products: Vec<TiqetsProduct>,
}

/// Clé partenaire disponible : celle de Portaki (plan) ou celle de l'hôte.
pub fn has_tiqets(ctx: &Context) -> bool {
    ctx.has_capability(capability::external::TIQETS_POOL)
        || ctx.has_capability(capability::external::TIQETS_BYOK)
}

/// Pourquoi la section ne peut pas s'afficher, pour le dire à l'hôte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiqetsStatus {
    Off,
    MissingKey,
    MissingCoordinates,
    Ready,
}

/// Où chercher : la position du logement, `None` tant qu'il n'est pas géocodé — jamais de repli.
fn property_position(ctx: &Context) -> Option<(f64, f64)> {
    let point = ctx.property.coordinates.as_ref()?;
    valid_coords(point.lat, point.lng)
}

pub fn status(ctx: &Context, config: &TiqetsConfig) -> TiqetsStatus {
    if !config.enabled {
        TiqetsStatus::Off
    } else if !has_tiqets(ctx) {
        TiqetsStatus::MissingKey
    } else if property_position(ctx).is_none() {
        TiqetsStatus::MissingCoordinates
    } else {
        TiqetsStatus::Ready
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TiqetsCache {
    lat: f64,
    lng: f64,
    radius_km: u32,
    #[serde(default)]
    min_rating: Option<u8>,
    lang: String,
    /// Secondes Unix, horloge de l'hôte.
    fetched_at: i64,
    products: Vec<TiqetsProduct>,
}

/// Résout la section pour le voyageur, ou `None` — éteinte, sans clé, sans position, sans
/// horloge, ou sans produit à montrer. Aucune de ces issues n'est une erreur de rendu.
pub fn resolve(ctx: &Context, config: &TiqetsConfig) -> Option<TiqetsView> {
    if status(ctx, config) != TiqetsStatus::Ready {
        return None;
    }
    let (lat, lng) = property_position(ctx)?;
    let now = time::now().ok()?.timestamp();
    let lang = tiqets_lang(&ctx.locale);
    let radius_km = config.normalized_radius_km();
    let min_rating = config.normalized_min_rating();

    let cached = read_cache(&lang).filter(|cache| {
        same_query(cache, lat, lng, radius_km, min_rating) && now - cache.fetched_at >= 0
    });
    if let Some(cache) = cached.as_ref() {
        if now - cache.fetched_at < FRESH_SECS {
            return view(cache.products.clone());
        }
    }

    let mut args =
        NearbyProductsArgs::new(lat, lng, radius_km, &lang, CURRENCY, MAX_PRODUCTS as u32);
    args.min_rating = min_rating;
    match Tiqets::nearby_products(&args) {
        Ok(response) => {
            let mut products = response.products;
            products.truncate(MAX_PRODUCTS);
            let _ = write_cache(&TiqetsCache {
                lat,
                lng,
                radius_km,
                min_rating,
                lang,
                fetched_at: now,
                products: products.clone(),
            });
            view(products)
        }
        Err(error) => {
            let mut fields = log::Fields::new();
            fields.insert("error", &error.to_string());
            let _ = log::warn("local_guide_tiqets_fetch_failed", &fields);
            // Tiqets injoignable, quota épuisé : l'entrée d'hier vaut mieux que rien, tant
            // qu'elle reste sous la limite de 14 jours.
            cached
                .filter(|cache| now - cache.fetched_at < STALE_MAX_SECS)
                .and_then(|cache| view(cache.products))
        }
    }
}

fn view(products: Vec<TiqetsProduct>) -> Option<TiqetsView> {
    if products.is_empty() {
        None
    } else {
        Some(TiqetsView { products })
    }
}

/// La langue du livret si Tiqets la sert, l'anglais sinon.
pub fn tiqets_lang(locale: &str) -> String {
    let code = Localized::lang_code(locale);
    if TIQETS_LANGS.contains(&code.as_str()) {
        code
    } else {
        "en".to_string()
    }
}

fn same_query(
    cache: &TiqetsCache,
    lat: f64,
    lng: f64,
    radius_km: u32,
    min_rating: Option<u8>,
) -> bool {
    cache.radius_km == radius_km
        && cache.min_rating == min_rating
        && (cache.lat - lat).abs() < 0.0001
        && (cache.lng - lng).abs() < 0.0001
}

fn cache_key(lang: &str) -> String {
    format!("{CACHE_KEY_PREFIX}{lang}")
}

fn read_cache(lang: &str) -> Option<TiqetsCache> {
    let bytes = host::kv::get(&cache_key(lang)).ok()??;
    serde_json::from_slice(&bytes).ok()
}

fn write_cache(cache: &TiqetsCache) -> Result<()> {
    let bytes = serde_json::to_vec(cache)
        .map_err(|error| PortakiError::Storage(format!("tiqets cache serialize: {error}")))?;
    // Le KV oublie de lui-même ce que Tiqets interdit de garder plus de 14 jours.
    host::kv::set(&cache_key(&cache.lang), &bytes, Some(STALE_MAX_SECS as u32))
}

/// « 22 € », « 22,50 € » en français ; « €22 », « €22.50 » ailleurs.
pub fn format_price(amount: f64, currency: &str, lang: &str) -> String {
    let rounded = (amount * 100.0).round() / 100.0;
    let number = if (rounded.fract()).abs() < f64::EPSILON {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.2}")
    };
    let symbol = match currency {
        "EUR" => "€",
        "USD" => "$",
        "GBP" => "£",
        other => other,
    };
    if Localized::lang_code(lang) == "fr" {
        format!("{} {symbol}", number.replace('.', ","))
    } else if symbol.len() == 3 && symbol.chars().all(|c| c.is_ascii_uppercase()) {
        format!("{number} {symbol}")
    } else {
        format!("{symbol}{number}")
    }
}

/// « 4,6 » en français, « 4.6 » ailleurs.
pub fn format_rating(average: f64, lang: &str) -> String {
    let text = format!("{:.1}", (average * 10.0).round() / 10.0);
    if Localized::lang_code(lang) == "fr" {
        text.replace('.', ",")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_booklet_language_is_kept_when_tiqets_serves_it() {
        assert_eq!(tiqets_lang("fr-FR"), "fr");
        assert_eq!(tiqets_lang("de_DE"), "de");
        assert_eq!(tiqets_lang("pt-BR"), "pt");
        // Tiqets ne sert pas l'ukrainien : l'anglais, comme leur propre défaut.
        assert_eq!(tiqets_lang("uk-UA"), "en");
    }

    #[test]
    fn prices_read_like_the_guest_language() {
        assert_eq!(format_price(22.0, "EUR", "fr-FR"), "22 €");
        assert_eq!(format_price(22.5, "EUR", "fr-FR"), "22,50 €");
        assert_eq!(format_price(22.0, "EUR", "en-US"), "€22");
        assert_eq!(format_price(19.99, "USD", "en-GB"), "$19.99");
        assert_eq!(format_price(10.0, "CHF", "en-US"), "10 CHF");
    }

    #[test]
    fn ratings_keep_one_decimal() {
        assert_eq!(format_rating(4.63, "fr-FR"), "4,6");
        assert_eq!(format_rating(4.0, "en-US"), "4.0");
    }

    #[test]
    fn the_cache_is_only_reused_for_the_same_query() {
        let cache = TiqetsCache {
            lat: 43.5513,
            lng: 7.0128,
            radius_km: 10,
            min_rating: None,
            lang: "fr".into(),
            fetched_at: 0,
            products: Vec::new(),
        };
        assert!(same_query(&cache, 43.5513, 7.0128, 10, None));
        assert!(!same_query(&cache, 43.5513, 7.0128, 20, None));
        assert!(!same_query(&cache, 43.5513, 7.0128, 10, Some(4)));
        assert!(!same_query(&cache, 43.6, 7.0128, 10, None));
    }
}
