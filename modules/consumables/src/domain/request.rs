//! Ce qu'une demande enregistrée dit d'elle-même : son produit, si elle attend encore, quand elle
//! a été livrée.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Entry, Status};

pub trait Asked {
    /// Le produit demandé.
    fn product(&self) -> Uuid;
    /// Pas encore livrée (à traiter ou prévue) : le produit manque toujours.
    fn pending(&self) -> bool;
    /// La dernière fois qu'elle est passée « Livré ».
    fn restocked_at(&self) -> Option<DateTime<Utc>>;
    /// La précision du voyageur, s'il en a écrit une.
    fn note(&self) -> Option<&str>;
}

impl Asked for Entry {
    fn product(&self) -> Uuid {
        Uuid::parse_str(&self.item.item_id).unwrap_or_default()
    }

    fn pending(&self) -> bool {
        self.status != Status::Restocked
    }

    fn restocked_at(&self) -> Option<DateTime<Utc>> {
        self.reached_at(Status::Restocked)
            .filter(|_| self.status == Status::Restocked)
    }

    fn note(&self) -> Option<&str> {
        Some(self.item.note.trim()).filter(|n| !n.is_empty())
    }
}
