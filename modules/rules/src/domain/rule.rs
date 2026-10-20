//! Ce qu'une règle sait d'elle-même : son icône, le thème que son texte évoque.

use crate::{Rule, Theme};

impl Rule {
    /// L'icône du livret, d'après le thème.
    pub fn icon(&self) -> &'static str {
        match self.theme {
            Theme::Noise => "clock-circle",
            Theme::Pets => "gift",
            Theme::Smoking => "x",
            Theme::Parties | Theme::Visitors => "users",
            _ => "check-circle",
        }
    }

    /// Le thème que le texte évoque, pour le suggérer à l'hôte (§4).
    pub fn suggested_theme(&self) -> Option<Theme> {
        let text = format!("{} {}", self.title.fr, self.detail.fr).to_lowercase()
            + &format!(" {} {}", self.title.en, self.detail.en).to_lowercase();
        THEME_WORDS
            .iter()
            .find(|(_, words)| words.iter().any(|word| text.contains(word)))
            .map(|(theme, _)| *theme)
    }
}

#[rustfmt::skip] // une table de mots, une ligne par thème
const THEME_WORDS: [(Theme, &[&str]); 7] = [
    (Theme::Noise, &["silence", "bruit", "calme", "musique", "noise", "quiet", "music"]),
    (Theme::Pets, &["animal", "animaux", "chien", "chat", "pet", "dog", "cat"]),
    (Theme::Smoking, &["fum", "tabac", "cigarette", "smok", "vap"]),
    (Theme::Parties, &["fête", "fete", "soirée", "party", "parties", "événement", "event"]),
    (Theme::Visitors, &["visiteur", "invité", "visitor", "guest"]),
    (Theme::Cleaning, &["ménage", "poubelle", "déchet", "vaisselle", "clean", "trash", "rubbish", "dishes"]),
    (Theme::Safety, &["sécurité", "feu", "bougie", "piscine", "safety", "fire", "candle", "pool"]),
];
