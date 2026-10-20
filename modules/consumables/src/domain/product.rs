//! Un produit du catalogue : son id, son nom, et le catalogue tel que le livret le montre.

use portaki_sdk::{Context, Result};
use uuid::Uuid;

use crate::{Product, Products};

impl Product {
    /// L'id des demandes qui visent ce produit.
    pub fn uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }

    /// Le nom, dans la langue demandée, sinon une autre.
    pub fn name(&self, locale: &str) -> String {
        self.label.get(locale).trim().to_string()
    }
}

/// Le catalogue tel que le livret le montre : sans les produits sans nom.
pub fn named_products(ctx: &Context) -> Result<Vec<Product>> {
    Ok(Products::load(ctx)?
        .rows()
        .iter()
        .filter(|p| !p.label.is_blank())
        .cloned()
        .collect())
}
