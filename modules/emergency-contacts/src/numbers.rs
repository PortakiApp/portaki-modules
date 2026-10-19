//! Les numéros d'urgence du pays du logement (§2.16, « calculé »).
//!
//! Le 112 vaut dans toute l'Union européenne et sur tout mobile ; il est donc toujours là, en
//! premier, en ton danger. Les numéros nationaux, eux, ne se devinent pas : composer un mauvais
//! numéro en urgence coûte des minutes que personne n'a. **Un pays absent de cette table ne reçoit
//! que le 112** — c'est moins riche, et c'est juste.
//!
//! ponytail: table courte et explicite. L'élargir demande une source par pays, pas une
//! extrapolation ; tant qu'elle n'existe pas, mieux vaut une tuile de moins qu'une fausse.

/// Un numéro à composer, et ce qu'il appelle.
pub struct EmergencyNumber {
    pub number: &'static str,
    /// Clé i18n du libellé — « Urgences », « SAMU », « Pompiers ».
    pub label_key: &'static str,
    /// Le numéro à composer quand on ne sait pas lequel : il passe en ton danger.
    pub primary: bool,
    /// Un numéro qu'on joint par SMS, pas par la voix (le 114, pour qui ne peut pas parler).
    pub sms: bool,
}

const EU_GENERAL: EmergencyNumber = EmergencyNumber {
    number: "112",
    label_key: "guest.numbers.general",
    primary: true,
    sms: false,
};

const fn national(number: &'static str, label_key: &'static str) -> EmergencyNumber {
    EmergencyNumber {
        number,
        label_key,
        primary: false,
        sms: false,
    }
}

/// Spec Urgences §2.1 : « France : 112, 15, 17, 18, 114 ».
const FRANCE: &[EmergencyNumber] = &[
    EU_GENERAL,
    national("15", "guest.numbers.medical"),
    national("17", "guest.numbers.police"),
    national("18", "guest.numbers.fire"),
    EmergencyNumber {
        number: "114",
        label_key: "guest.numbers.sms",
        primary: false,
        sms: true,
    },
];

/// Outre-mer : le 112 et les numéros français, qui y valent aussi. Ni le 114 ni le 3237 : leur
/// couverture outre-mer n'est pas sourcée, et un numéro qui ne répond pas coûte plus qu'une tuile
/// de moins.
const FRENCH_OVERSEAS: &[EmergencyNumber] = &[
    EU_GENERAL,
    national("15", "guest.numbers.medical"),
    national("17", "guest.numbers.police"),
    national("18", "guest.numbers.fire"),
];

/// Le pays du logement que cette table connaît, d'après `ctx.property.country` (ISO 3166-1, venu
/// de sa commune) ; `None` sinon : 112 seul.
///
/// **Ni la locale ni le fuseau.** La langue est celle d'un lecteur ou d'un hôte, pas d'un lieu ; un
/// fuseau en couvre plusieurs (`Europe/Paris` à un logement qui n'en a pas, la Belgique a le sien).
pub fn country_of(country: Option<&str>) -> Option<&'static str> {
    match country?.trim().to_ascii_uppercase().as_str() {
        "FR" => Some("FR"),
        "GP" => Some("GP"),
        "MQ" => Some("MQ"),
        "GF" => Some("GF"),
        "RE" => Some("RE"),
        "YT" => Some("YT"),
        _ => None,
    }
}

/// Les numéros à afficher pour ce logement, d'après son pays.
pub fn for_country(country: Option<&str>) -> &'static [EmergencyNumber] {
    match country_of(country) {
        Some("FR") => FRANCE,
        Some(_) => FRENCH_OVERSEAS,
        None => std::slice::from_ref(&EU_GENERAL),
    }
}

/// La pharmacie de garde du pays (spec §2.3) : le 3237 en France métropolitaine, rien ailleurs.
pub fn on_duty_pharmacy(country: Option<&str>) -> Option<&'static str> {
    (country_of(country) == Some("FR")).then_some("3237")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialled(country: Option<&str>) -> Vec<&'static str> {
        for_country(country).iter().map(|n| n.number).collect()
    }

    #[test]
    fn la_france_porte_ses_cinq_numeros() {
        let numbers = for_country(Some("FR"));
        assert_eq!(dialled(Some("fr")), ["112", "15", "17", "18", "114"]);
        assert!(numbers[4].sms, "le 114 se joint par SMS");
        assert_eq!(on_duty_pharmacy(Some("FR")), Some("3237"));
        assert!(
            numbers[0].primary,
            "le 112 est celui qu'on compose sans savoir"
        );
    }

    #[test]
    fn l_outre_mer_a_les_numeros_francais_sans_le_114_ni_le_3237() {
        for code in ["GP", "MQ", "GF", "RE", "YT"] {
            assert_eq!(dialled(Some(code)), ["112", "15", "17", "18"], "{code}");
            assert_eq!(on_duty_pharmacy(Some(code)), None, "{code}");
        }
    }

    /// Jamais de numéro inventé : composer un mauvais numéro en urgence coûte des minutes.
    #[test]
    fn un_pays_inconnu_ne_recoit_que_le_112() {
        for country in [Some("ES"), Some("BE"), Some("JP"), Some(""), None] {
            assert_eq!(dialled(country), ["112"], "{country:?}");
            assert_eq!(on_duty_pharmacy(country), None);
        }
    }
}
