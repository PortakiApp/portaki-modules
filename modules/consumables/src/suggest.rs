//! L'emoji proposé depuis le nom d'un produit (spec Consommables §4).
//!
//! Un dictionnaire court, en mots entiers : « thé » propose 🍵, « théière » rien. Il ne fait que
//! proposer — l'emoji que l'hôte saisit passe toujours devant.

const DICTIONARY: &[(&[&str], &str)] = &[
    (&["café", "capsules", "dosettes", "coffee", "pods"], "☕"),
    (&["thé", "tisane", "infusion", "tea"], "🍵"),
    (&["papier toilette", "toilet paper", "rouleaux"], "🧻"),
    (
        &[
            "liquide vaisselle",
            "éponge",
            "éponges",
            "dish soap",
            "sponge",
        ],
        "🧽",
    ),
    (&["tablettes", "lave vaisselle", "dishwasher"], "🍽️"),
    (&["sel", "salt"], "🧂"),
    (&["poivre", "pepper"], "🌶️"),
    (&["huile", "oil"], "🫒"),
    (&["sucre", "sugar"], "🍬"),
    (&["savon", "soap"], "🧼"),
    (&["gel douche", "shampoing", "shampoo", "shower gel"], "🧴"),
    (&["lessive", "laundry", "detergent"], "🧺"),
    (&["essuie tout", "paper towels"], "🧻"),
    (
        &["sacs poubelle", "poubelle", "bin bags", "trash bags"],
        "🗑️",
    ),
    (&["eau", "water"], "💧"),
    (&["lait", "milk"], "🥛"),
    (&["bois", "bûches", "firewood"], "🪵"),
];

/// Le nom en mots minuscules entourés d'espaces : « Liquide-vaisselle » → « liquide vaisselle ».
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

/// L'emoji de l'hôte, sinon celui que le nom propose, sinon rien.
pub fn shown(emoji: &str, name: &str) -> String {
    match emoji.trim() {
        "" => self::emoji(name).unwrap_or_default().to_string(),
        chosen => chosen.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only() {
        assert_eq!(emoji("Capsules de café"), Some("☕"));
        assert_eq!(emoji("Thé vert"), Some("🍵"));
        assert_eq!(emoji("Théière"), None);
        assert_eq!(emoji("Liquide-vaisselle"), Some("🧽"));
        assert_eq!(emoji(""), None);
        assert_eq!(shown("🫖", "Thé"), "🫖");
        assert_eq!(shown(" ", "Sel"), "🧂");
    }
}
