//! Ce que le module sait de lui-même : les demandes (entité, stockage, niveaux, statuts), l'emoji
//! proposé, et ce que les réglages veulent dire.

pub mod entities;
pub mod ids;
pub mod level;
pub mod status;
pub mod storage;
pub mod suggest;

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Collection, Problem};
use uuid::Uuid;

use crate::{Channel, Product, Products, Requests};

/// Demandes par séjour tant que l'hôte n'a rien choisi.
pub const DEFAULT_MAX_REQUESTS: u32 = 5;

impl Requests {
    /// Absent : oui, comme avant ce réglage.
    pub fn enabled(&self) -> bool {
        self.requests_enabled.unwrap_or(true)
    }

    /// Absent : e-mail et notification, le défaut de la spec.
    pub fn notify(&self) -> Vec<Channel> {
        self.notify
            .clone()
            .unwrap_or_else(|| vec![Channel::Email, Channel::App])
    }

    pub fn notifies_by_email(&self) -> bool {
        self.notify().contains(&Channel::Email)
    }

    /// Borné à 1..=20 pour le voyageur ; hors bornes, l'hôte le voit sous le champ.
    pub fn max(&self) -> u32 {
        self.max_requests
            .map_or(DEFAULT_MAX_REQUESTS, |n| n.clamp(1, 20))
    }
}

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

/// Demandes ouvertes sans canal : personne ne serait prévenu.
pub fn problems(products: &Collection<Product, Requests>) -> Vec<Problem> {
    let requests = products.meta();
    let silent = requests.enabled() && requests.notify.as_ref().is_some_and(Vec::is_empty);
    silent
        .then(|| Problem {
            row: None,
            field: "notify",
            message: portaki_sdk::bundle_text!("host.notify.required"),
        })
        .into_iter()
        .collect()
}
