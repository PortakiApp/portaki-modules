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

/// Le pays du logement, lu dans son fuseau horaire (`Europe/Paris` → `FR`) ; `None` quand cette
/// table ne le connaît pas : 112 seul.
///
/// **Le fuseau, jamais une locale.** `ctx.property.locale` n'est pas celle du logement : l'enveloppe
/// du SDK y recopie la locale de la requête, donc celle du lecteur. Un anglophone en France perdait
/// le 15, le 17, le 18 et le 114 ; un francophone en Espagne les aurait reçus. Le fuseau, lui, vient
/// du logement (`ModuleRuntimeInvocationAssembler`).
///
/// ponytail: un fuseau n'est pas un pays — la plateforme met `Europe/Paris` à un logement qui n'en a
/// pas, et la Belgique a le sien. Un vrai code pays dans le contexte le remplacera ; d'ici là, le
/// 112 reste juste partout et c'est lui qu'on affiche en premier.
pub fn country_of(timezone: &str) -> Option<&'static str> {
    match timezone.trim() {
        "Europe/Paris" => Some("FR"),
        _ => None,
    }
}

/// Les numéros à afficher pour ce logement, d'après son fuseau.
pub fn for_timezone(timezone: &str) -> &'static [EmergencyNumber] {
    match country_of(timezone) {
        Some("FR") => FRANCE,
        _ => std::slice::from_ref(&EU_GENERAL),
    }
}

/// La pharmacie de garde du pays (spec §2.3) : le 3237 en France, rien ailleurs.
pub fn on_duty_pharmacy(timezone: &str) -> Option<&'static str> {
    country_of(timezone).map(|_| "3237")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_france_porte_ses_cinq_numeros() {
        let numbers = for_timezone("Europe/Paris");
        let dialled: Vec<&str> = numbers.iter().map(|n| n.number).collect();
        assert_eq!(dialled, ["112", "15", "17", "18", "114"]);
        assert!(numbers[4].sms, "le 114 se joint par SMS");
        assert_eq!(on_duty_pharmacy("Europe/Paris"), Some("3237"));
        assert_eq!(on_duty_pharmacy("Europe/Madrid"), None);
        assert_eq!(numbers[0].number, "112");
        assert!(
            numbers[0].primary,
            "le 112 est celui qu'on compose sans savoir"
        );
    }

    /// Jamais de numéro inventé : composer un mauvais numéro en urgence coûte des minutes.
    #[test]
    fn un_pays_inconnu_ne_recoit_que_le_112() {
        for timezone in [
            "Europe/Madrid",
            "Europe/Brussels",
            "Asia/Tokyo",
            "fr-FR",
            "",
        ] {
            let numbers = for_timezone(timezone);
            assert_eq!(numbers.len(), 1, "{timezone}");
            assert_eq!(numbers[0].number, "112");
        }
    }
}
