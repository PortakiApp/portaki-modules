//! L'emoji et la pièce proposés depuis le nom d'un appareil (spec Appareils §4).
//!
//! Un dictionnaire court, en mots entiers : « four » propose 🔥 et la cuisine, « fourchette » rien.
//! Il ne fait que proposer — l'emoji et la pièce restent ceux que l'hôte saisit.

/// `(mots, emoji, pièce)` : la pièce est une clé `host.room.*` du bundle, traduite à l'affichage.
const DICTIONARY: &[(&[&str], &str, Option<&str>)] = &[
    (
        &["lave linge", "machine à laver", "washing machine", "washer"],
        "🌀",
        Some("host.room.laundry"),
    ),
    (
        &["sèche linge", "dryer", "tumble dryer"],
        "🧺",
        Some("host.room.laundry"),
    ),
    (
        &["sèche cheveux", "hair dryer", "hairdryer"],
        "💇",
        Some("host.room.bathroom"),
    ),
    (
        &["lave vaisselle", "dishwasher"],
        "🍽️",
        Some("host.room.kitchen"),
    ),
    (
        &["micro ondes", "microwave"],
        "♨️",
        Some("host.room.kitchen"),
    ),
    (&["four", "oven"], "🔥", Some("host.room.kitchen")),
    (
        &["plaques", "plaque", "induction", "cooktop", "hob", "stove"],
        "🍳",
        Some("host.room.kitchen"),
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
        Some("host.room.kitchen"),
    ),
    (&["bouilloire", "kettle"], "🫖", Some("host.room.kitchen")),
    (&["grille pain", "toaster"], "🍞", Some("host.room.kitchen")),
    (
        &["réfrigérateur", "frigo", "fridge", "congélateur", "freezer"],
        "🧊",
        Some("host.room.kitchen"),
    ),
    (
        &["télévision", "télé", "tv", "television"],
        "📺",
        Some("host.room.livingRoom"),
    ),
    (&["climatisation", "clim", "air conditioning"], "❄️", None),
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
        None,
    ),
    (&["aspirateur", "vacuum"], "🧹", None),
    (&["fer à repasser", "iron"], "👕", Some("host.room.laundry")),
    (
        &["barbecue", "bbq", "plancha"],
        "🍖",
        Some("host.room.outdoor"),
    ),
    (
        &["jacuzzi", "spa", "piscine", "pool"],
        "🏊",
        Some("host.room.outdoor"),
    ),
    (&["box", "routeur", "router", "wifi"], "📶", None),
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

fn entry(
    name: &str,
) -> Option<&'static (&'static [&'static str], &'static str, Option<&'static str>)> {
    let name = words(name);
    DICTIONARY.iter().find(|(keys, _, _)| {
        keys.iter()
            .any(|key| name.contains(&format!(" {} ", words(key).trim())))
    })
}

/// L'emoji proposé pour ce nom.
pub fn emoji(name: &str) -> Option<&'static str> {
    entry(name).map(|(_, emoji, _)| *emoji)
}

/// La clé `host.room.*` de la pièce proposée pour ce nom.
pub fn room_key(name: &str) -> Option<&'static str> {
    entry(name).and_then(|(_, _, room)| *room)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only() {
        assert_eq!(emoji("Four Bosch"), Some("🔥"));
        assert_eq!(room_key("four"), Some("host.room.kitchen"));
        assert_eq!(emoji("Fourchette"), None);
        assert_eq!(emoji("Lave-linge"), Some("🌀"));
        assert_eq!(room_key("Sèche-cheveux Dyson"), Some("host.room.bathroom"));
        assert_eq!(emoji("Smart TV"), Some("📺"));
        assert_eq!(room_key("Climatisation"), None);
        assert_eq!(emoji(""), None);
    }
}
