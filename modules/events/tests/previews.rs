//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use events::{
    map_markers, render_explore_detail, render_explore_item, render_home_card, render_upcoming_card,
};
use serde_json::json;

/// Trois rendez-vous réels d'Antibes saisis par l'hôte pour la semaine du séjour d'exemple, aux
/// positions relevées sur OpenStreetMap. L'agenda OpenAgenda reste éteint : il demande un appel
/// réseau, et l'aperçu montre ce que l'hôte maîtrise.
///
/// Datés depuis le jour où ils sont rendus — le concert le soir de `today`, le marché le matin et
/// le musée le soir de `tomorrow`. La veille du départ : ce soir, demain matin, demain soir. Un rendu figé ne dit ainsi que « Ce soir »
/// et « Demain », justes quel que soit le jour réel ; un rendez-vous à deux jours ou plus se lirait
/// « Samedi » ou « 6 juin », que le jour où l'on ouvre la démo dément.
///
/// La démo n'a pas un agenda mais trois livrets, chacun figé à sa propre heure : chaque rendu
/// reçoit les dates de son jour.
fn sample_config((today, tomorrow): (&str, &str)) -> serde_json::Value {
    json!({
        "events": [
            {
                "id": "marche-provencal",
                "title": { "fr": "Marché provençal", "en": "Provençal market" },
                "place": { "fr": "Cours Masséna", "en": "Cours Masséna" },
                "starts_at": format!("{tomorrow}T07:00:00"),
                "ends_at": format!("{tomorrow}T13:00:00"),
                "note": { "fr": "Producteurs du pays sous la halle du vieil Antibes : fruits, fromages, olives, fleurs.", "en": "Local growers under the covered hall of old Antibes: fruit, cheese, olives, flowers." },
                "price": "Entrée libre",
                "tips": {
                    "fr": "Venez avant 10 h, la halle se remplit vite.\nLa socca se vend chaude au bout de la halle.",
                    "en": "Come before 10am, the hall fills up fast.\nSocca is sold hot at the end of the hall."
                },
                "access": {
                    "fr": "Halle couverte, accès de plain-pied\nParking du Pré aux Pêcheurs à 300 m",
                    "en": "Covered hall, step-free access\nPré aux Pêcheurs car park 300 m away"
                },
                "address": "Cours Masséna, Antibes",
                "lat": 43.58093, "lng": 7.12758
            },
            {
                "id": "picasso",
                "title": { "fr": "Les collections du musée Picasso", "en": "The Picasso museum collections" },
                "place": { "fr": "Château Grimaldi", "en": "Château Grimaldi" },
                "starts_at": format!("{tomorrow}T19:00:00"),
                "ends_at": format!("{tomorrow}T22:00:00"),
                "note": { "fr": "Les œuvres que Picasso a peintes ici en 1946, dans le château qui domine la mer.", "en": "The works Picasso painted here in 1946, in the castle above the sea." },
                "address": "Place Mariejol, Antibes",
                "lat": 43.58071, "lng": 7.12826
            },
            {
                "id": "concert",
                "title": { "fr": "Concert au théâtre Anthéa", "en": "Concert at Anthéa theatre" },
                "place": { "fr": "Anthéa, théâtre d'Antibes", "en": "Anthéa, Antibes theatre" },
                "starts_at": format!("{today}T20:30:00"),
                "url": "https://www.anthea-antibes.fr/",
                "address": "260 avenue Jules Grec, Antibes",
                "lat": 43.59031, "lng": 7.11821
            }
        ],
        "disclaimer": { "fr": "Programme indicatif, à vérifier auprès des organisateurs.", "en": "Schedule for guidance, check with the organisers." },
        "nearby_enabled": false
    })
}

/// Le lendemain de [`previews::NOW`], deux fois : la veille de l'arrivée, les trois rendez-vous
/// sont demain, le jour où l'on arrive. Un concert « ce soir » s'y lirait sous un livret qui dit
/// « Demain, vous y êtes ».
const EVE_OF_ARRIVAL: (&str, &str) = ("2026-06-01", "2026-06-01");
/// Le jour de [`previews::DURING`] et son lendemain.
const EVE_OF_DEPARTURE: (&str, &str) = ("2026-06-07", "2026-06-08");
/// La veille de [`previews::AFTER`], deux fois : après le départ, les trois rendez-vous d'hier.
/// Ceux de la veille du départ, rendus le lendemain du départ, se liraient « 7 juin ».
const DAY_OF_DEPARTURE: (&str, &str) = ("2026-06-08", "2026-06-08");

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config(EVE_OF_ARRIVAL));
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("surface"));
    let item = context.clone().run(|mut ctx| {
        ctx.input = json!({ "eventId": "marche-provencal" });
        render_explore_item(ctx).expect("surface")
    });
    let upcoming = context
        .clone()
        .run(|ctx| render_upcoming_card(ctx).expect("surface"));
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    // Les points que ce module pose sur la carte du livret (§3). La démo les servait vides.
    let markers = context
        .clone()
        .run(|ctx| map_markers(ctx).expect("repères"));
    // Chaque fiche : la démo ouvre celle de l'événement tapé, pas toujours la même.
    let variants = ["marche-provencal", "picasso", "concert"]
        .into_iter()
        .map(|id| {
            let input = json!({ "eventId": id });
            let tree = context.clone().run(|mut ctx| {
                ctx.input = input.clone();
                render_explore_item(ctx).expect("surface")
            });
            ("explore.item", input, tree)
        })
        .collect();
    let home = context.run(render_home_card).expect("home");
    /*
     * Les mêmes surfaces aux deux autres heures de la démo. Un événement se date depuis l'heure
     * où il est rendu (« Ce soir », « Demain ») : rendu la veille d'arrivée et servi la veille du
     * départ, il annoncerait le mauvais soir. Après le départ, la fenêtre du séjour est close.
     */
    let at = |instant: &str| {
        let days = if instant == previews::AFTER {
            DAY_OF_DEPARTURE
        } else {
            EVE_OF_DEPARTURE
        };
        previews::guest_at(root, instant).with_config(&sample_config(days))
    };
    let lists = |instant: &str| {
        vec![
            (
                "explore.detail",
                at(instant).run(|ctx| render_explore_detail(ctx).expect("surface")),
            ),
            (
                "upcoming.card",
                at(instant).run(|ctx| render_upcoming_card(ctx).expect("surface")),
            ),
            (
                "home.card",
                at(instant).run(render_home_card).expect("home"),
            ),
        ]
    };
    let stay_items = ["marche-provencal", "picasso", "concert"]
        .into_iter()
        .map(|id| {
            let input = json!({ "eventId": id });
            let tree = at(previews::DURING).run(|mut ctx| {
                ctx.input = input.clone();
                render_explore_item(ctx).expect("surface")
            });
            ("stay", "explore.item", input, tree)
        })
        .collect();
    previews::check_all_with_phased_variants(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.detail", detail),
            ("explore.item", item),
            ("upcoming.card", upcoming),
        ],
        vec![("home.card", home)],
        Some(serde_json::to_value(markers).expect("repères")),
        vec![
            ("stay", lists(previews::DURING)),
            ("post-stay", lists(previews::AFTER)),
        ],
        variants,
        stay_items,
    );
}
