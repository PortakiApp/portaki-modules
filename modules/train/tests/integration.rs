//! Le tableau réel, par le connecteur mocké.
//!
//! Le module n'a plus aucun horaire écrit en dur : tout ce qui est vérifié ici sort des réponses
//! de `tests/fixtures/`, de la forme que Navitia documente.

use portaki_test_utils::{MockContext, MockContextBuilder, Property, SurfaceAssertions};
use serde_json::json;

use train::{render_explore_detail, render_explore_item, render_home_card, render_upcoming_card};

const PLACES: &str = include_str!("fixtures/sncf-places.json");
const DEPARTURES: &str = include_str!("fixtures/sncf-departures.json");

/// Un logement dont l'hôte a donné sa gare, et un fournisseur qui répond.
fn wired() -> MockContextBuilder {
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "station": "Antibes" }))
        .with_connector_response("sncf", "find_place", PLACES)
        .with_connector_response("sncf", "departures", DEPARTURES)
        .with_connector_response("sncf", "arrivals", DEPARTURES)
}

#[test]
fn the_board_comes_from_the_provider_not_from_the_module() {
    wired().run(|ctx| {
        let card = render_home_card(ctx).expect("render");
        assert!(SurfaceAssertions::new(&card).contains_type("Card"));
        // Des lignes, pas des `TimedEntry` : un départ s'ouvre et porte son état.
        assert!(SurfaceAssertions::new(&card).contains_type("ListItem"));
        assert!(!SurfaceAssertions::new(&card).contains_type("TimedEntry"));

        let json = serde_json::to_string(&card).expect("json");
        assert!(json.contains("\"type\":\"openOverlay\""));
        assert!(json.contains("explore.detail"));
        // Le nom officiel de la gare, pas celui tapé par l'hôte.
        assert!(json.contains("Gare d'Antibes"));
        assert!(json.contains("Nice-Ville (Nice)"));
        assert!(json.contains("08:12"));
        assert!(json.contains("TER"));
        // Les deux lignes illisibles de la réponse — l'une sans horaire, l'autre sans
        // direction — sont écartées, pas dessinées à moitié.
        assert!(!json.contains("Sans horaire"));
        assert!(!json.contains("sans direction"));
    });
}

#[test]
fn the_destinations_offered_come_from_the_board_itself() {
    wired().run(|ctx| {
        let detail = render_explore_detail(ctx).expect("render");
        // Le sens en segmenté, la gare en champ de recherche : les deux questions qu'on se pose
        // sur un quai, et que des pastilles de filtre ne posaient pas.
        assert!(SurfaceAssertions::new(&detail).contains_type("ChoiceList"));
        assert!(SurfaceAssertions::new(&detail).contains_type("ListItem"));
        assert!(!SurfaceAssertions::new(&detail).contains_type("FilterChip"));

        let json = serde_json::to_string(&detail).expect("json");
        assert!(json.contains("\"segmented\""));
        assert!(json.contains("\"combobox\""));
        assert!(json.contains("Nice-Ville (Nice)"));
        assert!(json.contains("Cannes (Cannes)"));
        // Aucune gare inventée : la liste ne contient que ce que le tableau a servi.
        assert!(!json.contains("Monaco"));
        assert!(!json.contains("Grasse"));
        // La fiche d'un départ est au bout de chaque ligne.
        assert!(json.contains("train/20261004-0812-"));
    });
}

#[test]
fn a_chosen_destination_filters_the_board_without_emptying_the_list() {
    wired().run(|mut ctx| {
        ctx.input = json!({ "dest": "Cannes (Cannes)" });
        let detail = render_explore_detail(ctx).expect("render");
        let json = serde_json::to_string(&detail).expect("json");
        assert!(json.contains("\"value\":\"Cannes (Cannes)\""));
        assert!(json.contains("08:42"));
        // Filtrer sur Cannes ne fait pas disparaître Nice de la liste des destinations.
        assert!(json.contains("Nice-Ville (Nice)"));
        // Mais son départ de 08:12, lui, n'est plus au tableau.
        assert!(!json.contains("08:12"));
    });
}

#[test]
fn an_unknown_destination_leaves_an_empty_state_rather_than_another_train() {
    wired().run(|mut ctx| {
        ctx.input = json!({ "dest": "Marseille" });
        let detail = render_explore_detail(ctx).expect("render");
        assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
        let json = serde_json::to_string(&detail).expect("json");
        assert!(!json.contains("08:12"));
    });
}

/// La nuit sans train : le premier résultat des 24 h suivantes est celui du matin (§2.17).
///
/// L'horloge est celle du logement, pas UTC : `Property::default()` est à Paris, donc 18 h UTC
/// est 20 h sur place, le 4 — et le train de 5 h 21 est bien celui du lendemain.
#[test]
fn a_departure_on_another_day_says_so() {
    wired()
        .with_now("2026-10-04T18:00:00Z".parse().expect("une heure du soir"))
        .with_translation("explore.detail.laterDay", "Demain")
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            let json = serde_json::to_string(&detail).expect("json");
            assert!(json.contains("05:21"), "le premier train du matin est là");
            assert!(json.contains("Demain"), "et il est annoncé pour demain");
        });
}

/// Passé minuit sur place, ce même train est redevenu « aujourd'hui » : rien ne l'annonce.
#[test]
fn after_midnight_the_morning_train_is_today_again() {
    wired()
        .with_now("2026-10-04T22:30:00Z".parse().expect("une heure de nuit"))
        .with_translation("explore.detail.laterDay", "Demain")
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            let json = serde_json::to_string(&detail).expect("json");
            assert!(json.contains("05:21"));
            assert!(!json.contains("Demain"));
        });
}

#[test]
fn the_compact_card_stays_a_single_line() {
    wired().run(|ctx| {
        let card = render_upcoming_card(ctx).expect("render");
        assert!(SurfaceAssertions::new(&card).contains_type("Card"));
        assert!(SurfaceAssertions::new(&card).contains_type("Text"));
        // Compact : pas de tableau complet sur la carte d'avant l'arrivée.
        assert!(!SurfaceAssertions::new(&card).contains_type("ListItem"));

        let json = serde_json::to_string(&card).expect("json");
        assert!(json.contains("upcoming.card"));
        assert!(json.contains("Gare d'Antibes → Nice-Ville (Nice) · 08:12"));
    });
}

#[test]
fn a_departure_sheet_opens_on_the_train_its_address_names() {
    wired().run(|mut ctx| {
        ctx.input = json!({ "departureId": "20261004-0812-17654" });
        let item = render_explore_item(ctx).expect("render");
        let json = serde_json::to_string(&item).expect("json");
        assert!(json.contains("Gare d'Antibes → Nice-Ville (Nice)"));
        assert!(json.contains("08:12"));
        assert!(json.contains("17654"));
    });
}

/// Un horaire passé, un lien gardé en favori : la fiche le dit.
#[test]
fn an_unknown_departure_sheet_says_so_rather_than_showing_the_next_train() {
    wired().run(|mut ctx| {
        ctx.input = json!({ "departureId": "20261004-2359-ter" });
        let item = render_explore_item(ctx).expect("render");
        assert!(SurfaceAssertions::new(&item).contains_type("EmptyState"));
        let json = serde_json::to_string(&item).expect("json");
        assert!(!json.contains("08:12"));
    });
}

/// Sans gare, c'est à l'hôte de la donner — pas au voyageur de comprendre une panne.
#[test]
fn without_a_station_the_booklet_says_what_is_missing() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
            assert!(!SurfaceAssertions::new(&detail).contains_type("ErrorState"));
        });
}

/// Un nom de gare qui ne correspond à rien n'est pas une panne du fournisseur.
#[test]
fn a_station_name_matching_nothing_says_so() {
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "station": "Gare de nulle part" }))
        .with_connector_response("sncf", "find_place", r#"{"places":[]}"#)
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
        });
}

/// Le fournisseur injoignable donne le gabarit d'erreur, jamais un tableau vide muet (§2.17).
#[test]
fn an_unreachable_provider_gives_the_error_template() {
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "station": "Antibes" }))
        .with_connector_error("sncf", "find_place", "upstream_unavailable")
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("ErrorState"));
        });
}

/// Une réponse que personne ne sait lire ne devient pas un tableau faux.
#[test]
fn a_body_nobody_can_read_is_not_a_board() {
    MockContext::guest()
        .with_property(Property::default())
        .with_config(&json!({ "station": "Antibes" }))
        .with_connector_response("sncf", "find_place", PLACES)
        .with_connector_response("sncf", "departures", r#"{"error":{"id":"unknown_object"}}"#)
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
            let json = serde_json::to_string(&detail).expect("json");
            assert!(!json.contains("08:12"));
        });
}
