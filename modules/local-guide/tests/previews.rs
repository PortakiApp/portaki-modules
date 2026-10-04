//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use local_guide::{
    render_explore_activity, render_explore_detail, render_explore_item, render_upcoming_card,
};

use serde_json::json;

/// Trois adresses d'exemple autour du logement fictif des fixtures (Cannes). Activités et
/// billets restent éteints, comme chez un hôte qui vient d'installer le module.
fn sample_config() -> serde_json::Value {
    json!({
        "spots": [
            {
                "id": "boulangerie",
                "title": { "fr": "La boulangerie du village", "en": "The village bakery" },
                "category": "Boulangerie",
                "distance": "350 m",
                "detail": { "fr": "Pains au levain et croissants au beurre, cuits sur place.", "en": "Sourdough and butter croissants, baked on site." },
                "note": { "fr": "Venez avant 9 h : les croissants partent vite le dimanche.", "en": "Come before 9am: the croissants go fast on Sundays." },
                "perk": { "fr": "Un café offert avec deux croissants, sur présentation de cette page.", "en": "A free coffee with two croissants, on showing this page." },
                "price": "€",
                "hours": "07:00 – 13:00 · 16:00 – 19:00",
                "opening": "Fermée le lundi",
                "parking": "Place de l'église, en face",
                "phone": "+33 4 93 00 00 01",
                "address": "Place de l'église, Cannes",
                "lat": 43.5531, "lng": 7.0152
            },
            {
                "id": "marche",
                "title": { "fr": "Marché provençal", "en": "Provençal market" },
                "category": "Marché",
                "distance": "1,2 km",
                "tag": "Samedi matin",
                "detail": { "fr": "Fruits, fromages et fleurs, le samedi matin.", "en": "Fruit, cheese and flowers, on Saturday mornings." },
                "lat": 43.5552, "lng": 7.0214
            },
            {
                "id": "lac",
                "title": { "fr": "Lac de Saint-Cassien", "en": "Lake Saint-Cassien" },
                "category": "Baignade",
                "distance": "9 km",
                "detail": { "fr": "Eau calme et plage de galets, idéal avec des enfants.", "en": "Calm water and a pebble beach, great with kids." },
                "lat": 43.5663, "lng": 6.8511
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
                "phone": "+33 6 22 33 44 55"
            }
        ],
        "disclaimer": { "fr": "Suggestions de votre hôte, sans partenariat.", "en": "Your host's picks, no partnership." }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("surface"));
    let item = context.clone().run(|mut ctx| {
        ctx.input = json!({ "spotId": "boulangerie" });
        render_explore_item(ctx).expect("surface")
    });
    let activity = context.clone().run(|mut ctx| {
        ctx.input = json!({ "activityId": "voilier" });
        render_explore_activity(ctx).expect("surface")
    });
    let upcoming = context.run(|ctx| render_upcoming_card(ctx).expect("surface"));
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.activity", activity),
            ("explore.detail", detail),
            ("explore.item", item),
            ("upcoming.card", upcoming),
        ],
    );
}
