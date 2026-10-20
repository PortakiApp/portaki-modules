//! `seedDefaults` — le catalogue de départ.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::{Product, Products};

#[portaki_sdk::command(name = "seedDefaults", example(label = "Catalogue par défaut"))]
pub fn seed_defaults(ctx: Context, _args: EmptyArgs) -> Result<()> {
    let mut products = Products::load(&ctx)?;
    if !products.is_empty() {
        return Ok(());
    }
    // La liste de départ de la spec (§4), chacun avec son emoji : la grille du voyageur se lit
    // alors d'un coup d'œil.
    let starter = [
        ("Café", "Coffee", "☕"),
        ("Thé", "Tea", "🍵"),
        ("Papier toilette", "Toilet paper", "🧻"),
        ("Liquide vaisselle", "Dish soap", "🧽"),
        ("Sel", "Salt", "🧂"),
        ("Huile", "Oil", "🫒"),
    ];
    products
        .rows_mut()
        .extend(starter.into_iter().map(|(fr, en, emoji)| Product {
            id: Uuid::new_v4().to_string(),
            label: I18nText::new(fr, en),
            emoji: emoji.into(),
            ..Product::default()
        }));
    Products::save(&products)
}
