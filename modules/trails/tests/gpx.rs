//! Lecture d'une trace GPS : ce qu'on en tire, et ce qu'on refuse d'en tirer.

use trails::gpx;

const TRACK: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Visorando">
  <trk><name>Cap Garoupe</name><trkseg>
    <trkpt lat="43.5500" lon="6.9400"><ele>10</ele></trkpt>
    <trkpt lat="43.5510" lon="6.9400"><ele>35</ele></trkpt>
    <trkpt lat="43.5520" lon="6.9400"><ele>20</ele></trkpt>
    <trkpt lat="43.5530" lon="6.9400"><ele>60</ele></trkpt>
  </trkseg></trk>
</gpx>"#;

#[test]
fn une_trace_donne_ses_points_sa_distance_et_sa_montee() {
    let track = gpx::read(TRACK).expect("une trace");

    // Quatre points alignés plein nord, 0,001° de latitude chacun ≈ 111 m.
    assert!(
        (track.distance_km - 0.333).abs() < 0.02,
        "{}",
        track.distance_km
    );
    // Dénivelé POSITIF : 25 puis 40 ; la descente de 15 ne compte pas.
    assert!(
        (track.elevation_m - 65.0).abs() < 0.5,
        "{}",
        track.elevation_m
    );
    assert_eq!(track.shape, "out_and_back");
    assert!(track.points.len() >= 2);
}

#[test]
fn une_boucle_se_reconnait_a_son_retour_au_depart() {
    let loop_track = TRACK.replace(
        r#"<trkpt lat="43.5530" lon="6.9400"><ele>60</ele></trkpt>"#,
        r#"<trkpt lat="43.5501" lon="6.9400"><ele>12</ele></trkpt>"#,
    );

    assert_eq!(gpx::read(&loop_track).expect("une trace").shape, "loop");
}

/// Un espace de noms ne change rien : beaucoup d'outils écrivent `<gpx:trkpt>`.
#[test]
fn un_espace_de_noms_ne_cache_pas_les_points() {
    let namespaced = TRACK
        .replace("trkpt", "gpx:trkpt")
        .replace("<ele>", "<gpx:ele>")
        .replace("</ele>", "</gpx:ele>");

    assert!(gpx::read(&namespaced).is_some());
}

#[test]
fn un_fichier_sans_deux_points_ne_donne_pas_de_trace() {
    assert!(gpx::read("<gpx></gpx>").is_none());
    assert!(
        gpx::read(r#"<gpx><trk><trkseg><trkpt lat="43.5" lon="6.9"/></trkseg></trk></gpx>"#)
            .is_none()
    );
    assert!(gpx::read("pas du tout du xml").is_none());
}

/// Une montre enregistre un point par seconde : des dizaines de milliers pour une journée.
/// Les envoyer tous au livret coûterait plus que la carte, et ne se verrait pas.
#[test]
fn une_trace_tres_dense_est_simplifiee_sous_le_plafond() {
    let mut xml = String::from("<gpx><trk><trkseg>");
    for i in 0..5_000 {
        let lat = 43.55 + f64::from(i) * 0.000_02;
        xml.push_str(&format!(
            r#"<trkpt lat="{lat:.6}" lon="6.9400"><ele>10</ele></trkpt>"#
        ));
    }
    xml.push_str("</trkseg></trk></gpx>");

    let track = gpx::read(&xml).expect("une trace");
    assert!(
        track.points.len() <= gpx::MAX_POINTS,
        "{} points",
        track.points.len()
    );
    // Simplifier ne doit pas raccourcir la randonnée : la distance vient des points d'origine.
    assert!(track.distance_km > 10.0, "{}", track.distance_km);
}

/// Les points gardés restent dans l'ordre du fichier : c'est un chemin, pas un nuage.
#[test]
fn la_simplification_garde_le_depart_et_l_arrivee() {
    let track = gpx::read(TRACK).expect("une trace");

    assert!((track.points[0].lat - 43.5500).abs() < 1e-9);
    assert!((track.points[track.points.len() - 1].lat - 43.5530).abs() < 1e-9);
}
