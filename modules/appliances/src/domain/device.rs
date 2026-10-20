//! L'emoji qu'un appareil montre.

use super::suggest;
use crate::Device;

impl Device {
    /// L'emoji de l'appareil, ou celui que son nom propose (§4) ; vide → l'icône prise.
    pub fn shown_emoji(&self, name: &str) -> String {
        match self.emoji.trim() {
            "" => suggest::emoji(name).unwrap_or_default().to_string(),
            emoji => emoji.to_string(),
        }
    }
}
