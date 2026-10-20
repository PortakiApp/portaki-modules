//! La limite de la carte d'accueil.

use crate::Display;

impl Display {
    /// Les règles de la carte : 4 tant que l'hôte n'a rien choisi.
    pub fn limit(&self) -> usize {
        if self.card_limit == 0 {
            4
        } else {
            self.card_limit as usize
        }
    }
}
