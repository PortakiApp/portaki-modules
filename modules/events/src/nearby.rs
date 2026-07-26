//! OpenAgenda nearby fetch, KV cache, and merge with host-curated events.

use chrono::{DateTime, Duration, Utc};
use portaki_connectors::open_agenda::{bbox_from_radius, NearbyEventsArgs, OpenAgenda};
use portaki_sdk::host::{self, time};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{EventRow, Localized, ModuleConfig};

const CACHE_KEY: &str = "nearby_cache";
const CACHE_TTL_SECS: i64 = 60 * 60;
const MAX_NEARBY: usize = 12;
const HOME_NEARBY_CAP: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NearbyCache {
    lat: f64,
    lng: f64,
    radius_km: u32,
    locale: String,
    fetched_at: DateTime<Utc>,
    events: Vec<EventRow>,
}

/// Returns true when pool or BYOK OpenAgenda capability is granted.
pub fn has_open_agenda(ctx: &Context) -> bool {
    ctx.has_capability(capability::external::OPEN_AGENDA_POOL)
        || ctx.has_capability(capability::external::OPEN_AGENDA_BYOK)
}

/// Merges host-curated events with nearby OpenAgenda results when enabled.
pub fn resolve_events(
    ctx: &Context,
    config: &ModuleConfig,
    for_home_card: bool,
) -> Result<Vec<EventRow>> {
    let mut manual = crate::time_format::sort_events_by_start(config.parse_events());
    if for_home_card {
        manual = crate::time_format::events_for_home_card(&manual);
    }

    if !config.nearby_enabled || !has_open_agenda(ctx) {
        return Ok(manual);
    }

    let lat = ctx.property.lat;
    let lng = ctx.property.lng;
    if !coords_usable(lat, lng) {
        return Ok(manual);
    }

    let nearby = load_nearby(ctx, config, lat, lng).unwrap_or_else(|_| Vec::new());
    let mut nearby = crate::time_format::sort_events_by_start(nearby);
    if for_home_card {
        nearby = crate::time_format::events_for_home_card(&nearby);
        nearby.truncate(HOME_NEARBY_CAP);
    } else {
        nearby.truncate(MAX_NEARBY);
    }

    Ok(merge_manual_and_nearby(manual, nearby))
}

/// Invalidates the nearby KV cache (host refresh).
pub fn invalidate_nearby_cache() -> Result<()> {
    host::kv::delete(CACHE_KEY)
}

fn load_nearby(
    ctx: &Context,
    config: &ModuleConfig,
    lat: f64,
    lng: f64,
) -> Result<Vec<EventRow>> {
    let now = time::now().unwrap_or_else(|_| Utc::now());
    let locale = Localized::lang_code(&ctx.locale);
    if let Some(cached) = read_cache()? {
        if cache_valid(&cached, lat, lng, config.radius_km, &locale, now) {
            return Ok(cached.events);
        }
    }

    let fetched = fetch_from_api(lat, lng, config.radius_km, &locale)?;
    let _ = write_cache(NearbyCache {
        lat,
        lng,
        radius_km: config.radius_km,
        locale,
        fetched_at: now,
        events: fetched.clone(),
    });
    Ok(fetched)
}

fn fetch_from_api(lat: f64, lng: f64, radius_km: u32, locale: &str) -> Result<Vec<EventRow>> {
    let (ne_lat, ne_lng, sw_lat, sw_lng) = bbox_from_radius(lat, lng, f64::from(radius_km));
    let response = OpenAgenda::nearby_events(&NearbyEventsArgs {
        geo_ne_lat: ne_lat,
        geo_ne_lng: ne_lng,
        geo_sw_lat: sw_lat,
        geo_sw_lng: sw_lng,
        relative: "upcoming".to_string(),
        size: MAX_NEARBY as u32,
        monolingual: locale.to_string(),
    })?;

    Ok(response
        .events
        .into_iter()
        .map(|event| EventRow {
            id: event.id,
            title: Localized::singleton(locale, event.title),
            place: Localized::singleton(locale, event.place),
            starts_at: event.starts_at.unwrap_or_default(),
            ends_at: None,
            url: event.url,
            lat: event.lat,
            lng: event.lng,
            note: None,
        })
        .collect())
}

fn merge_manual_and_nearby(manual: Vec<EventRow>, nearby: Vec<EventRow>) -> Vec<EventRow> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for event in manual {
        seen.insert(normalize_key(&event));
        out.push(event);
    }
    for event in nearby {
        let key = normalize_key(&event);
        if seen.insert(key) {
            out.push(event);
        }
    }
    crate::time_format::sort_events_by_start(out)
}

fn normalize_key(event: &EventRow) -> String {
    let title = event.title.get("fr").trim().to_ascii_lowercase();
    let title = if title.is_empty() {
        event.title.get("en").trim().to_ascii_lowercase()
    } else {
        title
    };
    format!("{}|{}", event.starts_at.trim(), title)
}

fn coords_usable(lat: f64, lng: f64) -> bool {
    lat.abs() > f64::EPSILON || lng.abs() > f64::EPSILON
}

fn cache_valid(
    cache: &NearbyCache,
    lat: f64,
    lng: f64,
    radius_km: u32,
    locale: &str,
    now: DateTime<Utc>,
) -> bool {
    if cache.radius_km != radius_km || cache.locale != locale {
        return false;
    }
    if (cache.lat - lat).abs() > 0.0001 || (cache.lng - lng).abs() > 0.0001 {
        return false;
    }
    now < cache.fetched_at + Duration::seconds(CACHE_TTL_SECS)
}

fn read_cache() -> Result<Option<NearbyCache>> {
    let Some(bytes) = host::kv::get(CACHE_KEY)? else {
        return Ok(None);
    };
    match serde_json::from_slice::<NearbyCache>(&bytes) {
        Ok(cache) => Ok(Some(cache)),
        Err(_) => Ok(None),
    }
}

fn write_cache(cache: NearbyCache) -> Result<()> {
    let bytes = serde_json::to_vec(&cache).map_err(|error| {
        PortakiError::Storage(format!("nearby cache serialize: {error}"))
    })?;
    host::kv::set(CACHE_KEY, &bytes, Some(CACHE_TTL_SECS as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_prefers_manual_on_duplicate_key() {
        let manual = vec![EventRow {
            id: "manual-1".into(),
            title: Localized::singleton("fr", "Concert"),
            place: Localized::singleton("fr", "Chez nous"),
            starts_at: "2099-07-25T18:00:00Z".into(),
            ends_at: None,
            url: None,
            lat: None,
            lng: None,
            note: None,
        }];
        let nearby = vec![EventRow {
            id: "oa-1".into(),
            title: Localized::singleton("fr", "Concert"),
            place: Localized::singleton("fr", "OpenAgenda"),
            starts_at: "2099-07-25T18:00:00Z".into(),
            ends_at: None,
            url: Some("https://example.com".into()),
            lat: Some(1.0),
            lng: Some(2.0),
            note: None,
        }];
        let merged = merge_manual_and_nearby(manual, nearby);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "manual-1");
        assert_eq!(merged[0].place.get("fr"), "Chez nous");
    }
}
