//! La carte d'accueil : combien d'appareils elle met en avant.

use crate::Display;

/// Les appareils de la carte d'accueil : 4 tant que l'hôte n'a rien choisi.
pub const DEFAULT_FEATURED: usize = 4;

impl Display {
    /// Combien d'appareils la carte d'accueil met en avant, borné à 2..=6.
    pub fn limit(&self) -> usize {
        match self.featured_limit {
            0 => DEFAULT_FEATURED,
            n => (n as usize).clamp(2, 6),
        }
    }
}
