//! OpenAgenda nearby fetch, KV cache, and merge with host-curated events.

use chrono::{DateTime, Duration, Utc};
use portaki_connectors::open_agenda::{bbox_from_radius, NearbyEventsArgs, OpenAgenda};
use portaki_sdk::host::{self, time};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use portaki_sdk::contracts::i18n::I18nText;

use crate::config::{EventRow, ModuleConfig};

const CACHE_KEY: &str = "nearby_cache";
const CACHE_TTL_SECS: i64 = 60 * 60;
const MAX_NEARBY: usize = 12;
/// Combien d'événements à venir un appel rapporte : de quoi remplir le livret (douze dans la
/// fenêtre du séjour) et les mois de l'onglet Calendrier, pour un seul appel par heure.
// ponytail: le fournisseur ne filtre que « à venir », pas par mois : au-delà des 100 prochains,
// un mois lointain de l'onglet reste incomplet. Paginer le jour où un logement s'en plaint.
const FETCH_SIZE: u32 = 100;
/// Deux événements à moins d'une heure d'écart sont le même (§3).
const DUPLICATE_WINDOW_SECS: i64 = 60 * 60;
/// Deux lieux à moins de 500 m sont le même.
const SAME_PLACE_KM: f64 = 0.5;
const HOME_NEARBY_CAP: usize = 3;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
pub(crate) fn host_now() -> DateTime<Utc> {
    time::now().unwrap_or_else(|_| DateTime::<Utc>::from_timestamp(0, 0).expect("epoch is valid"))
}

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
    let now = host_now();
    // §0.7 : de la veille de l'arrivée au lendemain du départ, et rien au-delà. Sans la borne
    // haute, un séjour de trois nuits affichait le festival du mois prochain.
    let window = crate::time_format::stay_window(ctx.stay.as_ref());
    let tz = ctx.property_tz();
    let anchored = |e| crate::time_format::anchored(e, tz.as_ref());
    let all_manual: Vec<EventRow> = config.parse_events().into_iter().map(anchored).collect();
    // Un import double le vôtre à n'importe laquelle de ses dates du séjour, pas seulement la
    // prochaine : le marché de chaque mardi double l'import du deuxième mardi aussi.
    let own_dates = occurrences_within(&all_manual, window, tz.as_ref());
    let mut manual = crate::time_format::sort_events_by_start(all_manual);
    manual = crate::time_format::events_within(&manual, window, tz.as_ref());
    if for_home_card {
        manual = crate::time_format::events_for_home_card(&manual, now, tz.as_ref());
    }

    if !config.nearby_enabled || !has_open_agenda(ctx) {
        return Ok(manual);
    }

    // Pas de position (logement non géocodé) : pas d'agenda alentour, et aucun appel.
    let Some(point) = ctx.property.coordinates.as_ref() else {
        return Ok(manual);
    };

    let nearby = load_nearby(ctx, config, point.lat, point.lng).unwrap_or_else(|_| Vec::new());
    let nearby = nearby.into_iter().map(anchored).collect();
    let mut nearby = crate::time_format::sort_events_by_start(nearby);
    nearby = crate::time_format::events_within(&nearby, window, tz.as_ref());
    if for_home_card {
        nearby = crate::time_format::events_for_home_card(&nearby, now, tz.as_ref());
        nearby.truncate(HOME_NEARBY_CAP);
    } else {
        nearby.truncate(MAX_NEARBY);
    }

    Ok(merge_manual_and_nearby(manual, nearby, &own_dates))
}

/// Vos événements à chacune de leurs dates dans la fenêtre ; tels quels sans fenêtre.
fn occurrences_within(
    events: &[EventRow],
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
    tz: Option<&time::PropertyTz>,
) -> Vec<EventRow> {
    let Some((from, to)) = window else {
        return events.to_vec();
    };
    let (first, last) = (
        crate::config::local(from, tz).date(),
        crate::config::local(to, tz).date(),
    );
    events
        .iter()
        .flat_map(|event| {
            event
                .occurrences_in(first, last, tz)
                .into_iter()
                .map(move |at| EventRow {
                    starts_at: at.to_rfc3339(),
                    ..event.clone()
                })
        })
        .collect()
}

/// Les événements d'OpenAgenda à venir autour du logement, à l'heure du logement — `None` quand
/// l'hôte ne les veut pas, que la capacité manque ou que le logement n'est pas placé ; `Err`
/// quand le fournisseur ne répond pas (§9 #6 : l'onglet le dit).
pub fn upcoming_nearby(ctx: &Context, config: &ModuleConfig) -> Result<Option<Vec<EventRow>>> {
    if !config.nearby_enabled || !has_open_agenda(ctx) {
        return Ok(None);
    }
    let Some(point) = ctx.property.coordinates.as_ref() else {
        return Ok(None);
    };
    let tz = ctx.property_tz();
    let events = load_nearby(ctx, config, point.lat, point.lng)?;
    Ok(Some(
        events
            .into_iter()
            .map(|e| crate::time_format::anchored(e, tz.as_ref()))
            .collect(),
    ))
}

/// Le même événement saisi et importé (§3) : même nom, moins d'une heure d'écart, même lieu.
/// Le lieu se compare par la position ; sans position d'un côté, nom et heure suffisent.
pub fn is_duplicate(own: &EventRow, imported: &EventRow) -> bool {
    let close_in_time = match (
        crate::time_format::parse_starts_at(&own.starts_at),
        crate::time_format::parse_starts_at(&imported.starts_at),
    ) {
        (Some(a), Some(b)) => (a - b).num_seconds().abs() <= DUPLICATE_WINDOW_SECS,
        _ => own.starts_at.trim() == imported.starts_at.trim(),
    };
    let same_place = match (own.lat.zip(own.lng), imported.lat.zip(imported.lng)) {
        (Some((lat, lng)), Some((other_lat, other_lng)))
            if own.has_coords() && imported.has_coords() =>
        {
            portaki_sdk::config::check::within_km(lat, lng, other_lat, other_lng, SAME_PLACE_KM)
        }
        _ => true,
    };
    let title = normalized_title(own);
    close_in_time && same_place && !title.is_empty() && title == normalized_title(imported)
}

/// Invalidates the nearby KV cache (host refresh).
pub fn invalidate_nearby_cache() -> Result<()> {
    host::kv::delete(CACHE_KEY)
}

fn load_nearby(ctx: &Context, config: &ModuleConfig, lat: f64, lng: f64) -> Result<Vec<EventRow>> {
    let now = time::now()?;
    let locale = match ctx.locale.trim().split(['-', '_']).next() {
        Some(code) if !code.is_empty() => code.to_ascii_lowercase(),
        _ => "fr".to_string(),
    };
    let radius_km = config.normalized_radius_km();
    if let Some(cached) = read_cache()? {
        if cache_valid(&cached, lat, lng, radius_km, &locale, now) {
            return Ok(cached.events);
        }
    }

    let fetched = fetch_from_api(lat, lng, radius_km, &locale)?;
    let _ = write_cache(NearbyCache {
        lat,
        lng,
        radius_km,
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
        size: FETCH_SIZE,
        monolingual: locale.to_string(),
    })?;

    Ok(response
        .events
        .into_iter()
        .map(|event| EventRow {
            id: event.id,
            title: I18nText::new(event.title.clone(), event.title),
            place: I18nText::new(event.place.clone(), event.place),
            starts_at: event.starts_at.unwrap_or_default(),
            ends_at: None,
            url: event.url,
            lat: event.lat,
            lng: event.lng,
            note: None,
            // Un événement trouvé à proximité n'a ni photo, ni prix, ni conseil : le fournisseur
            // n'en donne pas, et les inventer les ferait passer pour ceux de l'hôte.
            address: None,
            price: None,
            photo: String::new(),
            access: I18nText::default(),
            tips: I18nText::default(),
            ..EventRow::default()
        })
        .collect())
}

/// Vos événements, puis ceux d'OpenAgenda qui ne doublent aucune de vos dates (`own_dates`) :
/// le vôtre gagne (§3).
fn merge_manual_and_nearby(
    manual: Vec<EventRow>,
    nearby: Vec<EventRow>,
    own_dates: &[EventRow],
) -> Vec<EventRow> {
    let kept: Vec<EventRow> = nearby
        .into_iter()
        .filter(|imported| !own_dates.iter().any(|own| is_duplicate(own, imported)))
        .collect();
    crate::time_format::sort_events_by_start(manual.into_iter().chain(kept).collect())
}

fn normalized_title(event: &EventRow) -> String {
    let title = event.title.get("fr").trim().to_lowercase();
    if title.is_empty() {
        event.title.get("en").trim().to_lowercase()
    } else {
        title
    }
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
    let bytes = serde_json::to_vec(&cache)
        .map_err(|error| PortakiError::Storage(format!("nearby cache serialize: {error}")))?;
    host::kv::set(CACHE_KEY, &bytes, Some(CACHE_TTL_SECS as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_prefers_manual_on_duplicate_key() {
        let manual = vec![EventRow {
            id: "manual-1".into(),
            title: I18nText::new("Concert", ""),
            place: I18nText::new("Chez nous", ""),
            starts_at: "2099-07-25T18:00:00Z".into(),
            ends_at: None,
            url: None,
            lat: None,
            lng: None,
            note: None,
            ..Default::default()
        }];
        let nearby = vec![EventRow {
            id: "oa-1".into(),
            title: I18nText::new("Concert", ""),
            place: I18nText::new("OpenAgenda", ""),
            starts_at: "2099-07-25T18:00:00Z".into(),
            ends_at: None,
            url: Some("https://example.com".into()),
            lat: Some(1.0),
            lng: Some(2.0),
            note: None,
            ..Default::default()
        }];
        let merged = merge_manual_and_nearby(manual.clone(), nearby, &manual);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "manual-1");
        assert_eq!(merged[0].place.get("fr"), "Chez nous");
    }

    fn at(title: &str, starts_at: &str, lat: f64, lng: f64) -> EventRow {
        EventRow {
            title: I18nText::new(title, ""),
            starts_at: starts_at.into(),
            lat: Some(lat),
            lng: Some(lng),
            ..Default::default()
        }
    }

    /// §3 : même nom, ± 1 h, même lieu — sinon deux événements.
    #[test]
    fn a_duplicate_is_same_name_within_an_hour_at_the_same_place() {
        let own = at("Concert sur la plage", "2099-08-22T21:00:00Z", 43.55, 7.01);
        assert!(is_duplicate(
            &own,
            &at(
                "concert sur la plage",
                "2099-08-22T20:15:00Z",
                43.5502,
                7.0102
            )
        ));
        assert!(!is_duplicate(
            &own,
            &at("Concert sur la plage", "2099-08-22T19:30:00Z", 43.55, 7.01)
        ));
        assert!(!is_duplicate(
            &own,
            &at("Concert sur la plage", "2099-08-22T21:00:00Z", 43.60, 7.01)
        ));
        assert!(!is_duplicate(
            &own,
            &at("Concert au port", "2099-08-22T21:00:00Z", 43.55, 7.01)
        ));
    }

    /// Le calendrier veut toutes les occurrences du mois, pas la prochaine.
    #[test]
    fn occurrences_fill_the_month() {
        let first = chrono::NaiveDate::from_ymd_opt(2099, 2, 1).unwrap();
        let last = chrono::NaiveDate::from_ymd_opt(2099, 2, 28).unwrap();
        let days = |event: &EventRow| -> Vec<String> {
            event
                .occurrences_in(first, last, None)
                .iter()
                .map(|at| at.format("%d").to_string())
                .collect()
        };
        let monthly = EventRow {
            starts_at: "2099-01-31T10:00:00Z".into(),
            recurrence: "monthly".into(),
            ..Default::default()
        };
        assert!(days(&monthly).is_empty(), "pas de 31 février");
        let weekly = EventRow {
            starts_at: "2099-02-10T10:00:00Z".into(),
            recurrence: "weekly".into(),
            weekdays: vec!["tue".into(), "sat".into()],
            ..Default::default()
        };
        assert_eq!(days(&weekly), ["10", "14", "17", "21", "24", "28"]);
        let chosen = EventRow {
            starts_at: "2099-01-20T10:00:00Z".into(),
            recurrence: "dates".into(),
            dates: vec![
                crate::config::EventDate {
                    date: "2099-02-03".into(),
                },
                crate::config::EventDate {
                    date: "2099-03-03".into(),
                },
            ],
            ..Default::default()
        };
        assert_eq!(days(&chosen), ["03"]);
    }
}
