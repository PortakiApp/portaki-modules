//! L'emoji proposé depuis le nom d'un appareil (spec Appareils §4).
//!
//! Un dictionnaire court, en mots entiers : « four » propose 🔥, « fourchette » rien.
//! Il ne fait que proposer — l'emoji reste celui que l'hôte saisit.

/// `(mots, emoji)`.
const DICTIONARY: &[(&[&str], &str)] = &[
    (
        &["lave linge", "machine à laver", "washing machine", "washer"],
        "🌀",
    ),
    (&["sèche linge", "dryer", "tumble dryer"], "🧺"),
    (&["sèche cheveux", "hair dryer", "hairdryer"], "💇"),
    (&["lave vaisselle", "dishwasher"], "🍽️"),
    (&["micro ondes", "microwave"], "♨️"),
    (&["four", "oven"], "🔥"),
    (
        &["plaques", "plaque", "induction", "cooktop", "hob", "stove"],
        "🍳",
    ),
    (
        &[
            "cafetière",
            "machine à café",
            "coffee",
            "nespresso",
            "expresso",
        ],
        "☕",
    ),
    (&["bouilloire", "kettle"], "🫖"),
    (&["grille pain", "toaster"], "🍞"),
    (
        &["réfrigérateur", "frigo", "fridge", "congélateur", "freezer"],
        "🧊",
    ),
    (&["télévision", "télé", "tv", "television"], "📺"),
    (&["climatisation", "clim", "air conditioning"], "❄️"),
    (
        &[
            "chauffage",
            "radiateur",
            "chaudière",
            "heater",
            "heating",
            "boiler",
        ],
        "🌡️",
    ),
    (&["aspirateur", "vacuum"], "🧹"),
    (&["fer à repasser", "iron"], "👕"),
    (&["barbecue", "bbq", "plancha"], "🍖"),
    (&["jacuzzi", "spa", "piscine", "pool"], "🏊"),
    (&["box", "routeur", "router", "wifi"], "📶"),
];

/// Le nom en mots minuscules entourés d'espaces : « Lave-linge Bosch » → « lave linge bosch ».
fn words(name: &str) -> String {
    let lower = name.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    format!(" {} ", tokens.join(" "))
}

/// L'emoji proposé pour ce nom.
pub fn emoji(name: &str) -> Option<&'static str> {
    let name = words(name);
    DICTIONARY
        .iter()
        .find(|(keys, _)| {
            keys.iter()
                .any(|key| name.contains(&format!(" {} ", words(key).trim())))
        })
        .map(|(_, emoji)| *emoji)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only() {
        assert_eq!(emoji("Four Bosch"), Some("🔥"));
        assert_eq!(emoji("Fourchette"), None);
        assert_eq!(emoji("Lave-linge"), Some("🌀"));
        assert_eq!(emoji("Smart TV"), Some("📺"));
        assert_eq!(emoji(""), None);
    }
}
