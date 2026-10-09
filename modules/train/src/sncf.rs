//! Les horaires réels, par le connecteur `sncf` (API Navitia de SNCF).
//!
//! ADR-0021 : le connecteur est déclaré ici. Sans clé d'hôte (`host_key = false`) : une seule clé
//! éditeur sert tous les logements, comme pour Viator. Quand elle manque
//! (`connector_credential_missing`), le module le note et l'écran de l'hôte le dit.
//!
//! Deux appels, deux caches :
//! - la gare → son `stop_area`, gardé 30 jours : un nom de gare ne change pas ;
//! - le tableau des départs, gardé [`BOARD_TTL_SECS`] (2 min). C'est un tableau d'affichage, pas
//!   un catalogue : le garder une heure afficherait des trains partis.
//!
//! Sans horloge de l'hôte, rien n'est affiché : juger la fraîcheur sur une fausse heure servirait
//! indéfiniment le même tableau.
//!
//! ponytail: les types de réponse sont écrits d'après la documentation Navitia, pas d'après une
//! réponse capturée — Portaki n'a pas encore de clé. Tout champ est donc facultatif, et un corps
//! que personne ne sait lire donne le gabarit d'erreur plutôt qu'un tableau faux. Le premier appel
//! réel confirmera ou corrigera `tests/fixtures/sncf-departures.json`, en un seul endroit.

use portaki_sdk::host::{kv, log, time};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Fraîcheur d'un tableau d'affichage.
pub const BOARD_TTL_SECS: i64 = 2 * 60;

/// Durée de vie de la correspondance « nom de gare → `stop_area` ».
const STATION_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Départs demandés par appel. Navitia regarde les 24 h qui suivent, ce qui donne le premier train
/// du matin quand on consulte le livret la nuit.
pub const BOARD_COUNT: u32 = 10;

/// Noté quand l'API a été appelée sans la clé de Portaki ; effacé au premier appel qui aboutit.
pub const KEY_MISSING_KEY: &str = "sncf_key_missing";

const STATION_KEY_PREFIX: &str = "sncf_station.";
const BOARD_KEY_PREFIX: &str = "sncf_board.";

#[portaki_sdk::custom_connector(
    id = "sncf",
    display_name_key = "connector.sncf.name",
    base_url = "https://api.sncf.com/v1",
    // L'API SNCF attend la clé en nom d'utilisateur, mot de passe vide.
    //
    // La passerelle encode le secret **tel qu'il est au coffre** : `Basic base64(<secret>)`, sans
    // y ajouter de deux-points (`EgressAuth.Kind.BASIC`, le coffre garde « identifiant:mot de
    // passe » tel que saisi). Le secret à déposer est donc `<clé>:`, **deux-points final compris**.
    // Sans lui, l'appel part en `base64(<clé>)` et SNCF répond 401 sans rien dire d'utile.
    auth = "basic",
    host_key = false
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleSncf;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleSncf {
    /// Le nom de gare → les lieux qui lui correspondent. Gardé un jour côté passerelle aussi : le
    /// même logement redemande la même gare.
    #[portaki_sdk::connector_op(
        connector = "sncf",
        method = "GET",
        path = "/coverage/sncf/places",
        cache = "24h",
        fields = "q, count",
        // La gare que l'hôte a écrite, et rien d'autre : ni le voyageur, ni l'adresse du logement.
        sends = "module_config"
    )]
    pub fn find_place() {}

    /// Les prochains départs d'une gare.
    #[portaki_sdk::connector_op(
        connector = "sncf",
        method = "GET",
        path = "/coverage/sncf/stop_areas/{id}/departures",
        fields = "id, count, data_freshness",
        sends = "module_config"
    )]
    pub fn departures() {}

    /// Les prochaines arrivées — le sens « vers la gare » du livret.
    #[portaki_sdk::connector_op(
        connector = "sncf",
        method = "GET",
        path = "/coverage/sncf/stop_areas/{id}/arrivals",
        fields = "id, count, data_freshness",
        sends = "module_config"
    )]
    pub fn arrivals() {}
}

#[derive(Serialize)]
struct PlaceArgs<'a> {
    q: &'a str,
    count: u32,
}

#[derive(Serialize)]
struct BoardArgs<'a> {
    id: &'a str,
    count: u32,
    data_freshness: &'a str,
}

/// Ce que Navitia renvoie sur `/places` : on ne garde que les gares.
#[derive(Debug, Clone, Deserialize)]
struct PlacesResponse {
    #[serde(default)]
    places: Vec<Place>,
}

#[derive(Debug, Clone, Deserialize)]
struct Place {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    embedded_type: Option<String>,
    #[serde(default)]
    stop_area: Option<StopArea>,
}

#[derive(Debug, Clone, Deserialize)]
struct StopArea {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    /// Navitia écrit les coordonnées en chaînes : `{ "lat": "43.58", "lon": "7.12" }`.
    #[serde(default)]
    coord: Option<Coord>,
}

#[derive(Debug, Clone, Deserialize)]
struct Coord {
    #[serde(default)]
    lat: Option<String>,
    #[serde(default)]
    lon: Option<String>,
}

/// `/departures` et `/arrivals` ont la même forme, sous deux noms.
#[derive(Debug, Clone, Deserialize)]
struct BoardResponse {
    #[serde(default)]
    departures: Vec<BoardStop>,
    #[serde(default)]
    arrivals: Vec<BoardStop>,
}

#[derive(Debug, Clone, Deserialize)]
struct BoardStop {
    #[serde(default)]
    stop_date_time: Option<StopDateTime>,
    #[serde(default)]
    display_informations: Option<DisplayInformations>,
}

#[derive(Debug, Clone, Deserialize)]
struct StopDateTime {
    #[serde(default)]
    departure_date_time: Option<String>,
    #[serde(default)]
    arrival_date_time: Option<String>,
    /// L'horaire de la fiche horaire, quand le temps réel s'en écarte. C'est l'écart entre les
    /// deux qui fait le retard — Navitia ne sert aucun champ « retard ».
    #[serde(default)]
    base_departure_date_time: Option<String>,
    #[serde(default)]
    base_arrival_date_time: Option<String>,
    #[serde(default)]
    data_freshness: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DisplayInformations {
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    headsign: Option<String>,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    commercial_mode: Option<String>,
}

/// La gare résolue : son identifiant Navitia, le nom que Navitia lui donne, et où elle est.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    /// Le nom officiel, qui vaut mieux que celui tapé par l'hôte sur l'écran du voyageur.
    pub label: String,
    /// Où est la gare, pour le temps d'accès ; absent d'une gare mise en cache avant lui.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
}

/// Une ligne du tableau, telle que le livret la dessine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stop {
    /// L'heure locale de la gare, `HH:MM`.
    pub time: String,
    /// La date locale de la gare, `AAAA-MM-JJ` : c'est elle qui dit « demain ».
    pub date: String,
    /// La direction du train — la gare au bout de la ligne.
    pub direction: String,
    /// Le numéro de la mission (« TER 17654 »), quand Navitia le donne.
    pub headsign: Option<String>,
    /// « TER », « TGV INOUI », « Intercités ».
    pub mode: Option<String>,
    /// Le réseau, quand il diffère du mode.
    pub network: Option<String>,
    /// L'horaire est celui du temps réel, pas celui de la fiche horaire.
    pub realtime: bool,
    /// Les minutes de retard, quand le temps réel s'écarte de la fiche horaire.
    ///
    /// `None` sur un horaire théorique : sans temps réel, personne ne sait si le train est en
    /// retard, et `Some(0)` dirait « à l'heure » sans l'avoir vérifié.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_min: Option<i64>,
}

impl Stop {
    /// L'identifiant de route de cette ligne : sa date, son heure et sa mission.
    ///
    /// Ni un rang ni un index : un tableau se décale dès qu'un train part, et une fiche ouverte sur
    /// « le troisième » montrerait un autre train une minute plus tard.
    pub fn route_id(&self) -> String {
        let mission = self.headsign.as_deref().unwrap_or(&self.direction);
        format!(
            "{}-{}-{}",
            self.date.replace('-', ""),
            self.time.replace(':', ""),
            slug(mission)
        )
    }
}

/// Minuscules, et tout ce qui n'est pas une lettre ou un chiffre devient un tiret.
fn slug(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// Pourquoi le tableau ne peut pas s'afficher, pour le dire à l'hôte et au voyageur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    /// L'hôte n'a pas donné sa gare.
    NoStation,
    /// Le séjour n'a pas commencé, ou il est fini depuis plus d'un jour (§0.7).
    OutsideStay,
    /// La clé éditeur de Portaki manque.
    MissingKey,
    /// Le nom de gare ne correspond à aucune gare.
    UnknownStation,
    /// L'API n'a pas répondu, ou a répondu ce que personne ne sait lire.
    Unavailable,
}

/// Le sens que le voyageur regarde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// Les départs depuis la gare du logement.
    From,
    /// Les arrivées à la gare du logement.
    To,
}

impl Way {
    pub fn wire(self) -> &'static str {
        match self {
            Way::From => "from",
            Way::To => "to",
        }
    }

    /// Ramène un `dir` reçu dans les deux sens, le défaut sinon : on part du logement plus souvent
    /// qu'on y revient en train.
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some(value) if value.eq_ignore_ascii_case("to") => Way::To,
            _ => Way::From,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BoardCache {
    station: Station,
    /// Secondes Unix, horloge de l'hôte.
    fetched_at: i64,
    stops: Vec<Stop>,
}

/// Le tableau d'une gare dans un sens, ou pourquoi il n'y en a pas.
///
/// Le troisième membre est l'heure de lecture du tableau, en secondes Unix : c'est elle qui permet
/// de dire au voyageur de quand datent ces horaires.
pub fn board(
    station_name: &str,
    way: Way,
) -> std::result::Result<(Station, Vec<Stop>, i64), BoardError> {
    let now = time::now()
        .map_err(|_| BoardError::Unavailable)?
        .timestamp();
    let station = station(station_name)?;

    let key = board_key(&station.id, way);
    let cached: Option<BoardCache> = read(&key);
    if let Some(cache) = cached.as_ref() {
        if now - cache.fetched_at >= 0 && now - cache.fetched_at < BOARD_TTL_SECS {
            return Ok((cache.station.clone(), cache.stops.clone(), cache.fetched_at));
        }
    }

    let args = BoardArgs {
        id: &station.id,
        count: BOARD_COUNT,
        data_freshness: "realtime",
    };
    let operation = match way {
        Way::From => "departures",
        Way::To => "arrivals",
    };
    let fetched: std::result::Result<BoardResponse, PortakiError> =
        portaki_sdk::host::connectors::call("sncf", operation, &args);
    match fetched {
        Ok(response) => {
            clear_missing_key();
            let stops = parse_board(&response, way);
            let _ = write(
                &key,
                &BoardCache {
                    station: station.clone(),
                    fetched_at: now,
                    stops: stops.clone(),
                },
                BOARD_TTL_SECS,
            );
            Ok((station, stops, now))
        }
        Err(error) => Err(fetch_error(error, "train_board_fetch_failed")),
    }
}

/// Les lignes d'une réponse, l'heure du sens demandé, les illisibles écartées.
fn parse_board(response: &BoardResponse, way: Way) -> Vec<Stop> {
    let rows = match way {
        Way::From => &response.departures,
        Way::To => &response.arrivals,
    };
    // Navitia nomme la liste d'après l'appel ; un corps qui porte l'autre nom reste lisible.
    let rows = if rows.is_empty() {
        match way {
            Way::From => &response.arrivals,
            Way::To => &response.departures,
        }
    } else {
        rows
    };
    rows.iter().filter_map(|row| map_stop(row, way)).collect()
}

fn map_stop(row: &BoardStop, way: Way) -> Option<Stop> {
    let times = row.stop_date_time.as_ref()?;
    // L'heure du sens demandé d'abord ; l'autre plutôt que rien, un terminus n'ayant pas de départ.
    let raw = match way {
        Way::From => times
            .departure_date_time
            .as_deref()
            .or(times.arrival_date_time.as_deref()),
        Way::To => times
            .arrival_date_time
            .as_deref()
            .or(times.departure_date_time.as_deref()),
    }?;
    let (date, time) = split_navitia_datetime(raw)?;
    let realtime = times
        .data_freshness
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("realtime"));
    let base = match way {
        Way::From => times
            .base_departure_date_time
            .as_deref()
            .or(times.base_arrival_date_time.as_deref()),
        Way::To => times
            .base_arrival_date_time
            .as_deref()
            .or(times.base_departure_date_time.as_deref()),
    };
    let info = row.display_informations.as_ref();
    let direction = info
        .and_then(|i| i.direction.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(Stop {
        time,
        date,
        direction: direction.to_string(),
        headsign: text(info.and_then(|i| i.headsign.as_deref())),
        mode: text(info.and_then(|i| i.commercial_mode.as_deref())),
        network: text(info.and_then(|i| i.network.as_deref())),
        realtime,
        delay_min: realtime
            .then_some(base)
            .flatten()
            .and_then(|base| delay_minutes(base, raw)),
    })
}

/// Les minutes de retard entre la fiche horaire et le temps réel.
///
/// Rien quand le train est à l'heure ou en avance : « +0 min » n'est pas une information, et une
/// avance n'en est pas une non plus sur un quai — on attend le train, il ne partira pas plus tôt.
fn delay_minutes(base: &str, realtime: &str) -> Option<i64> {
    let minutes = |raw: &str| {
        let (date, time) = split_navitia_datetime(raw)?;
        let day = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
        let (hours, mins) = time.split_once(':')?;
        Some(
            day.and_hms_opt(hours.parse().ok()?, mins.parse().ok()?, 0)?
                .and_utc()
                .timestamp()
                / 60,
        )
    };
    let late = minutes(realtime)? - minutes(base)?;
    (late > 0).then_some(late)
}

fn text(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// `20160613T135400` → (`2016-06-13`, `13:54`).
///
/// L'heure est déjà celle de la gare : Navitia sert le fuseau de la couverture, pas UTC. On ne la
/// reconvertit donc pas — ce serait décaler un horaire juste.
pub fn split_navitia_datetime(raw: &str) -> Option<(String, String)> {
    let raw = raw.trim();
    let (date, time) = raw.split_once('T')?;
    if date.len() != 8 || time.len() < 4 || !date.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if !time.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some((
        format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8]),
        format!("{}:{}", &time[0..2], &time[2..4]),
    ))
}

/// La gare de l'hôte, résolue puis gardée.
/// La gare que l'hôte a nommée, résolue (et gardée en cache) — ou pourquoi elle ne l'est pas.
pub fn station(name: &str) -> std::result::Result<Station, BoardError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BoardError::NoStation);
    }
    let key = station_key(name);
    if let Some(station) = read::<Station>(&key) {
        return Ok(station);
    }
    let args = PlaceArgs { q: name, count: 10 };
    let fetched: std::result::Result<PlacesResponse, PortakiError> =
        portaki_sdk::host::connectors::call("sncf", "find_place", &args);
    match fetched {
        Ok(response) => {
            clear_missing_key();
            let station = first_station(&response).ok_or(BoardError::UnknownStation)?;
            let _ = write(&key, &station, STATION_TTL_SECS);
            Ok(station)
        }
        Err(error) => Err(fetch_error(error, "train_station_lookup_failed")),
    }
}

/// La première gare de la réponse.
///
/// Le filtre se fait ici plutôt qu'avec `type[]=stop_area` : ce nom de paramètre à crochets ne
/// passe pas par les champs nommés d'une opération, et Navitia classe déjà ses résultats par
/// pertinence.
fn first_station(response: &PlacesResponse) -> Option<Station> {
    response.places.iter().find_map(|place| {
        if place.embedded_type.as_deref() != Some("stop_area") {
            return None;
        }
        let area = place.stop_area.as_ref();
        let id = area
            .and_then(|a| a.id.as_deref())
            .or(place.id.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty())?;
        let label = area
            .and_then(|a| a.name.as_deref())
            .or(place.name.as_deref())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(id);
        let coord = area.and_then(|a| a.coord.as_ref());
        let parse = |raw: Option<&String>| raw.and_then(|v| v.trim().parse::<f64>().ok());
        Some(Station {
            id: id.to_string(),
            label: label.to_string(),
            lat: coord.and_then(|c| parse(c.lat.as_ref())),
            lng: coord.and_then(|c| parse(c.lon.as_ref())),
        })
    })
}

/// Une clé manquante se note pour l'hôte ; tout le reste est une indisponibilité.
fn fetch_error(error: PortakiError, event: &str) -> BoardError {
    if error.to_string().contains("connector_credential_missing") {
        note_missing_key();
        return BoardError::MissingKey;
    }
    let mut fields = log::Fields::new();
    fields.insert("error", &error.to_string());
    let _ = log::warn(event, &fields);
    BoardError::Unavailable
}

/// Durée d'un constat de clé manquante : passé ce délai, l'écran de l'hôte cesse de le dire.
const NOTE_SECS: u32 = 24 * 60 * 60;

fn note_missing_key() {
    let _ = kv::set(KEY_MISSING_KEY, b"1", Some(NOTE_SECS));
}

fn clear_missing_key() {
    let _ = kv::delete(KEY_MISSING_KEY);
}

pub fn missing_key() -> bool {
    matches!(kv::get(KEY_MISSING_KEY), Ok(Some(_)))
}

fn station_key(name: &str) -> String {
    format!("{STATION_KEY_PREFIX}{}", slug(name))
}

fn board_key(station_id: &str, way: Way) -> String {
    format!("{BOARD_KEY_PREFIX}{}.{}", slug(station_id), way.wire())
}

fn read<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    let bytes = kv::get(key).ok()??;
    serde_json::from_slice(&bytes).ok()
}

fn write<T: Serialize>(key: &str, value: &T, ttl_secs: i64) -> Result<()> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| PortakiError::Storage(format!("{key} serialize: {error}")))?;
    kv::set(key, &bytes, Some(ttl_secs as u32))
}
