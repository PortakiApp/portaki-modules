//! Ce que les réglages des demandes veulent dire quand l'hôte n'a rien choisi.

use crate::{Channel, Requests};

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
