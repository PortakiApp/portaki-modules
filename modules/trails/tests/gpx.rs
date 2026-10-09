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
    // Une valeur que le formulaire et la tuile « Type » connaissent.
    assert_eq!(track.shape, "round_trip");
    assert!(trails::SHAPES.contains(&track.shape));
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

/// La barre du bas suit ce que le voyageur fera : la trace d'abord, s'il en a une.
mod barre_du_bas {
    use portaki_sdk::capability;
    use portaki_test_utils::{MockContext, SurfaceAssertions};
    use serde_json::json;
    use trails::render_explore_item;

    const GPX_REF: &str = "portaki-file:6f1c1d2e-3a4b-4c5d-8e9f-0a1b2c3d4e5f";

    fn config(with_gpx: bool, with_link: bool) -> serde_json::Value {
        let mut trail = json!({
            "id": "t1",
            "title": { "fr": "Cap Garoupe" },
            "level": "easy",
            "lat": 43.55,
            "lng": 6.94,
        });
        if with_gpx {
            trail["gpx_file"] = json!(GPX_REF);
        }
        if with_link {
            trail["link_url"] = json!("https://www.visorando.com/randonnee-cap");
        }
        json!({ "trails": [trail] })
    }

    fn detail(with_gpx: bool, with_link: bool) -> String {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config(with_gpx, with_link))
            .run(|mut ctx| {
                ctx.input = json!({ "trailId": "t1" });
                let surface = render_explore_item(ctx).expect("surface");
                assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
                serde_json::to_string(&surface).expect("json")
            })
    }

    #[test]
    fn la_trace_part_en_reference_pas_en_url() {
        let json = detail(true, true);

        // Le module ne fabrique pas d'URL : la plateforme échange la référence contre une URL
        // signée au rendu, et retire le lien si le fichier n'est pas à ce logement.
        assert!(json.contains(GPX_REF), "la référence voyage telle quelle");
    }

    #[test]
    fn sans_trace_deposee_aucun_bouton_de_telechargement() {
        let json = detail(false, true);

        assert!(!json.contains("downloadGpx"));
        assert!(json.contains("openTrace"), "la fiche tierce prend la place");
    }
}

/// Le refus à l'import (§9 n° 6) : le message de la spec, pas un fichier ignoré en silence.
#[test]
fn une_trace_refusee_dit_pourquoi() {
    assert_eq!(gpx::refusal(TRACK.as_bytes()), None);
    // Pas un GPX, ou pas deux points.
    assert_eq!(
        gpx::refusal(b"pas du tout du xml"),
        Some("host.trails.gpx.invalid")
    );
    assert_eq!(
        gpx::refusal(b"<gpx></gpx>"),
        Some("host.trails.gpx.invalid")
    );
    // Des points, mais sans racine `<gpx>` : un autre format.
    let kml = TRACK.replace("<gpx ", "<kml ").replace("</gpx>", "</kml>");
    assert_eq!(
        gpx::refusal(kml.as_bytes()),
        Some("host.trails.gpx.invalid")
    );
    // Une entité déclarée : refusée même si les points se lisent.
    let entity = TRACK.replace(
        "<gpx ",
        "<!DOCTYPE gpx [<!ENTITY x SYSTEM \"file:///etc/passwd\">]>\n<gpx ",
    );
    assert!(gpx::read(&entity).is_some());
    assert_eq!(
        gpx::refusal(entity.as_bytes()),
        Some("host.trails.gpx.invalid")
    );
    // Au-delà de 5 Mo.
    let mut big = TRACK.as_bytes().to_vec();
    big.resize(gpx::MAX_BYTES + 1, b' ');
    assert_eq!(gpx::refusal(&big), Some("host.trails.gpx.tooLarge"));
}

/// Les deux messages, mot pour mot.
#[test]
fn les_messages_de_refus_sont_ceux_de_la_spec() {
    let fr: serde_json::Value =
        serde_json::from_str(include_str!("../i18n/fr-FR.json")).expect("lot fr");
    assert_eq!(
        fr["host.trails.gpx.invalid"],
        "Ce fichier n’est pas une trace GPX valide."
    );
    assert_eq!(fr["host.trails.gpx.tooLarge"], "5 Mo au maximum.");
}
