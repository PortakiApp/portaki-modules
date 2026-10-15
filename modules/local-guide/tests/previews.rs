//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use local_guide::{
    map_markers, render_explore_activity, render_explore_detail, render_explore_item,
    render_explore_link, render_home_card, render_upcoming_card,
};

use serde_json::json;

/// Trois adresses réelles autour du logement de la démo (Cap d'Antibes), positions relevées sur
/// OpenStreetMap : le marché du vieil Antibes, la plage au bout du chemin, le Fort Carré — assez
/// loin pour que la fiche ne propose plus que la voiture. Activités et billets restent éteints,
/// comme chez un hôte qui vient d'installer le module.
fn sample_config() -> serde_json::Value {
    json!({
        "spots": [
            {
                "id": "marche",
                "title": { "fr": "Marché provençal", "en": "Provençal market" },
                "category": "Marché",
                "distance": "3,3 km",
                "tag": "Le matin",
                "detail": { "fr": "Fruits, fromages, olives et fleurs sous la halle du cours Masséna, dans le vieil Antibes.", "en": "Fruit, cheese, olives and flowers under the covered hall of Cours Masséna, in old Antibes." },
                "note": { "fr": "Venez avant 10 h : après, la halle est noire de monde.", "en": "Come before 10am: after that, the hall is packed." },
                "hours": "06:00 – 13:00",
                "photos": ["portaki-file:marche-1"],
                "opening": "Fermé le lundi hors saison",
                "parking": "Parking du Pré aux Pêcheurs, à 300 m",
                "address": "Cours Masséna, Antibes",
                "lat": 43.58093, "lng": 7.12758
            },
            {
                "id": "garoupe",
                "title": { "fr": "Plage de la Garoupe", "en": "La Garoupe beach" },
                "category": "Baignade",
                "distance": "850 m",
                "detail": { "fr": "Sable fin et eau calme au fond de l'anse, idéal avec des enfants.", "en": "Fine sand and calm water at the back of the cove, great with kids." },
                "address": "Chemin de la Garoupe, Antibes",
                "lat": 43.55455, "lng": 7.1357
            },
            {
                "id": "fort-carre",
                "title": { "fr": "Fort Carré", "en": "Fort Carré" },
                "category": "Patrimoine",
                "distance": "4,4 km",
                "detail": { "fr": "Le fort en étoile qui veille sur le port Vauban, et la vue jusqu'aux Alpes par temps clair.", "en": "The star-shaped fort watching over Port Vauban, with views to the Alps on a clear day." },
                "address": "Quai Éric Tabarly, Antibes",
                "lat": 43.59034, "lng": 7.12762
            }
        ],
        // Une activité que l'hôte propose lui-même (§2.13) : pas un lien partenaire, une sortie
        // qu'on réserve au téléphone avec son prestataire.
        "host_activities": [
            {
                "id": "voilier",
                "title": { "fr": "Sortie voilier au coucher du soleil", "en": "Sunset sailing trip" },
                "provider": "Marc, skipper au port",
                "price": "60 € / pers.",
                "duration": "3 h",
                "meet": "Port Vauban, ponton C",
                "languages": "Français, anglais",
                "cancel": { "fr": "Annulation à voir directement avec Marc.", "en": "Cancellation to be arranged directly with Marc." },
                "included": {
                    "fr": "Boissons à bord\nArrêt baignade à la Garoupe",
                    "en": "Drinks on board\nSwim stop at La Garoupe"
                },
                "tip": {
                    "fr": "Dites-lui que vous venez d'ici : il vous garde le coucher de soleil.",
                    "en": "Tell him you are staying here: he will save you the sunset."
                },
                "phone": "+33 6 22 33 44 55",
                "photo": "portaki-file:voilier-1"
            }
        ],
        // Un lien collé par l'hôte et reconnu par le catalogue (§2.13, `origin: hostLink`) : c'est
        // lui qui donne la fiche `explore.link` de l'aperçu.
        "activities_enabled": true,
        "activities_destination": "Antibes",
        "activities": [
            {
                "id": "petit-train",
                "url": "https://www.viator.com/fr-FR/tours/Antibes/The-Little-Train-of-Antibes-Juan-les-Pins/d21941-273628P2",
                "label": { "fr": "Le petit train d'Antibes", "en": "The little train of Antibes" },
                "tip": {
                    "fr": "Asseyez-vous à droite : c'est le côté qui longe les remparts.",
                    "en": "Sit on the right: that side runs along the ramparts."
                }
            }
        ],
        "disclaimer": { "fr": "Suggestions de votre hôte, sans partenariat.", "en": "Your host's picks, no partnership." }
    })
}

/// La réponse Viator réelle du bac à sable, celle des tests : l'aperçu montre une vraie fiche.
const VIATOR: &str = include_str!("fixtures/viator-freetext.json");

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root)
        .with_config(&sample_config())
        .with_connector_response("viator", "search_products", VIATOR);
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("surface"));
    let item = context.clone().run(|mut ctx| {
        ctx.input = json!({ "spotId": "marche" });
        render_explore_item(ctx).expect("surface")
    });
    let activity = context.clone().run(|mut ctx| {
        ctx.input = json!({ "activityId": "voilier" });
        render_explore_activity(ctx).expect("surface")
    });
    let hostlink = context.clone().run(|mut ctx| {
        ctx.input = json!({ "productCode": "273628P2" });
        render_explore_link(ctx).expect("surface")
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
    // Chaque fiche que la démo peut ouvrir : celle de l'adresse, de l'activité ou du lien tapé,
    // pas toujours la même.
    let variant = |surface_id, input: serde_json::Value| {
        let render = match surface_id {
            "explore.activity" => render_explore_activity,
            "explore.link" => render_explore_link,
            _ => render_explore_item,
        };
        let tree = context.clone().run(|mut ctx| {
            ctx.input = input.clone();
            render(ctx).expect("surface")
        });
        (surface_id, input, tree)
    };
    let variants = ["marche", "garoupe", "fort-carre"]
        .into_iter()
        .map(|id| variant("explore.item", json!({ "spotId": id })))
        .chain([
            variant("explore.activity", json!({ "activityId": "voilier" })),
            variant("explore.link", json!({ "productCode": "273628P2" })),
        ])
        .collect();
    let home = context.run(render_home_card).expect("home");
    previews::check_all_with_variants(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.activity", activity),
            ("explore.detail", detail),
            ("explore.item", item),
            ("explore.link", hostlink),
            ("upcoming.card", upcoming),
        ],
        vec![("home.card", home)],
        Some(serde_json::to_value(markers).expect("repères")),
        Vec::new(),
        variants,
    );
}
