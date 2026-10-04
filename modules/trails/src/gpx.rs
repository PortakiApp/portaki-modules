//! Lire une trace GPS déposée par l'hôte : les points, et ce qu'on en déduit.
//!
//! Le fichier est servi tel quel au voyageur qui le télécharge, donc on ne le réécrit jamais ; on
//! en tire des points, une distance, un dénivelé et une forme, qui pré-remplissent les champs que
//! l'hôte reste libre de corriger (§2.23).
//!
//! Le parseur ne construit pas d'arbre XML : il balaie le texte à la recherche des `<trkpt>` et de
//! leurs `<ele>`. C'est assez pour un GPX, et ça ne peut pas se faire piéger par ce qu'un parseur
//! complet accepterait — entités, inclusions, imbrications profondes. La plateforme refuse déjà
//! toute déclaration de document au dépôt ; ceci est la seconde ceinture.

use portaki_sdk::sdui::GeoPoint;

/// Au-delà, on simplifie plus fort : une trace tient dans une carte, pas dans une base.
pub const MAX_POINTS: usize = 400;

/// Rayon terrestre moyen, en mètres.
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Une trace lue, et ce qu'elle dit d'elle-même.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// Les points, dans l'ordre, déjà simplifiés.
    pub points: Vec<GeoPoint>,
    /// Longueur cumulée, en kilomètres.
    pub distance_km: f64,
    /// Somme des montées seules, en mètres — le dénivelé positif, pas l'amplitude.
    pub elevation_m: f64,
    /// `loop` quand l'arrivée rejoint le départ, sinon `out_and_back`.
    pub shape: &'static str,
}

/// Une boucle se referme : moins de cette distance entre le départ et l'arrivée.
const LOOP_TOLERANCE_M: f64 = 120.0;

/// Les points d'un GPX, dans l'ordre du fichier.
///
/// Les attributs sont lus sur le `<trkpt>` lui-même ; une altitude vaut pour le point qui la
/// précède. Un point dont la latitude ou la longitude ne se lit pas est sauté plutôt que de
/// décaler toute la suite.
fn read_points(xml: &str) -> Vec<(GeoPoint, Option<f64>)> {
    let mut points = Vec::new();
    let mut rest = xml;
    while let Some(start) = find_tag(rest, "trkpt").or_else(|| find_tag(rest, "rtept")) {
        rest = &rest[start..];
        let Some(end) = rest.find('>') else { break };
        let open = &rest[..end];
        let lat = attribute(open, "lat");
        let lon = attribute(open, "lon");
        rest = &rest[end + 1..];
        let Some((lat, lon)) = lat.zip(lon) else {
            continue;
        };
        // L'altitude du point : le premier `<ele>` avant le point suivant.
        let until = find_tag(rest, "trkpt")
            .or_else(|| find_tag(rest, "rtept"))
            .unwrap_or(rest.len());
        let elevation = element(&rest[..until], "ele");
        points.push((GeoPoint::new(lat, lon), elevation));
    }
    points
}

/// La position d'une balise ouvrante `<name` ou `<ns:name`, insensible à la casse.
fn find_tag(haystack: &str, name: &str) -> Option<usize> {
    let lowered = haystack.to_lowercase();
    let mut from = 0;
    while let Some(at) = lowered[from..].find('<') {
        let at = from + at;
        let after = &lowered[at + 1..];
        let after = after.strip_prefix('/').unwrap_or(after);
        let after = match after.find(':') {
            Some(colon) if colon < 12 && !after[..colon].contains(['<', ' ', '>']) => {
                &after[colon + 1..]
            }
            _ => after,
        };
        if after.starts_with(name)
            && after[name.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
            && !lowered[at..].starts_with("</")
        {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// `lat="43.55"` → `43.55`. Les deux guillemets sont acceptés, comme en XML.
fn attribute(open_tag: &str, name: &str) -> Option<f64> {
    let lowered = open_tag.to_lowercase();
    let at = lowered.find(&format!("{name}="))? + name.len() + 1;
    let rest = open_tag[at..].trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value = &rest[1..];
    let end = value.find(quote)?;
    value[..end].trim().parse().ok()
}

/// Le contenu du premier `<name>…</name>`, en nombre.
fn element(xml: &str, name: &str) -> Option<f64> {
    let at = find_tag(xml, name)?;
    let open_end = xml[at..].find('>')? + at + 1;
    let close = xml[open_end..].find('<')? + open_end;
    xml[open_end..close].trim().parse().ok()
}

/// Distance orthodromique entre deux points, en mètres.
fn haversine_m(a: GeoPoint, b: GeoPoint) -> f64 {
    let (lat1, lat2) = (a.lat.to_radians(), b.lat.to_radians());
    let dlat = lat2 - lat1;
    let dlng = (b.lng - a.lng).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().asin()
}

/// Distance d'un point au segment `[start, end]`, en mètres — approximation plane, suffisante
/// aux échelles d'une randonnée.
fn perpendicular_m(point: GeoPoint, start: GeoPoint, end: GeoPoint) -> f64 {
    let scale = point.lat.to_radians().cos();
    let (px, py) = ((point.lng - start.lng) * scale, point.lat - start.lat);
    let (sx, sy) = ((end.lng - start.lng) * scale, end.lat - start.lat);
    let length2 = sx * sx + sy * sy;
    if length2 == 0.0 {
        return haversine_m(point, start);
    }
    let t = ((px * sx + py * sy) / length2).clamp(0.0, 1.0);
    let closest = GeoPoint::new(
        start.lat + t * sy,
        start.lng + t * sx / scale.max(f64::EPSILON),
    );
    haversine_m(point, closest)
}

/// Douglas-Peucker : garde les points qui changent la forme, jette ceux qui la répètent.
fn simplify(points: &[GeoPoint], tolerance_m: f64) -> Vec<GeoPoint> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let (index, worst) = points[1..points.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, p)| (i + 1, perpendicular_m(*p, first, last)))
        .fold(
            (0usize, 0.0f64),
            |acc, cur| if cur.1 > acc.1 { cur } else { acc },
        );

    if worst <= tolerance_m {
        return vec![first, last];
    }
    let mut left = simplify(&points[..=index], tolerance_m);
    let right = simplify(&points[index..], tolerance_m);
    left.pop();
    left.extend(right);
    left
}

/// Simplifie jusqu'à tenir sous [`MAX_POINTS`], en relâchant la tolérance.
///
/// Une trace d'un GPS de montre porte un point par seconde : des dizaines de milliers pour une
/// journée. Les envoyer tous au livret coûterait plus que la carte elle-même, et ne se verrait pas.
fn simplify_to_budget(points: &[GeoPoint]) -> Vec<GeoPoint> {
    let mut tolerance = 5.0;
    let mut kept = simplify(points, tolerance);
    while kept.len() > MAX_POINTS && tolerance < 2_000.0 {
        tolerance *= 2.0;
        kept = simplify(points, tolerance);
    }
    kept
}

/// Lit une trace GPX. `None` quand le fichier ne porte pas deux points lisibles — il n'y a alors
/// pas de tracé à dessiner, et la fiche montre son seul départ.
pub fn read(xml: &str) -> Option<Track> {
    let raw = read_points(xml);
    if raw.len() < 2 {
        return None;
    }

    let distance_m: f64 = raw
        .windows(2)
        .map(|pair| haversine_m(pair[0].0, pair[1].0))
        .sum();

    // Dénivelé positif : on ne somme que les montées, et on ignore les sautes du GPS sous le mètre.
    let mut climb = 0.0;
    let mut previous: Option<f64> = None;
    for (_, elevation) in &raw {
        if let Some(current) = *elevation {
            if let Some(before) = previous {
                let delta = current - before;
                if delta > 1.0 {
                    climb += delta;
                }
            }
            previous = Some(current);
        }
    }

    let start = raw[0].0;
    let finish = raw[raw.len() - 1].0;
    let shape = if haversine_m(start, finish) <= LOOP_TOLERANCE_M {
        "loop"
    } else {
        "out_and_back"
    };

    let points: Vec<GeoPoint> = raw.iter().map(|(point, _)| *point).collect();
    Some(Track {
        points: simplify_to_budget(&points),
        distance_km: distance_m / 1000.0,
        elevation_m: climb,
        shape,
    })
}
