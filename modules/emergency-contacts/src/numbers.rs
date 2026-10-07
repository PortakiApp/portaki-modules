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
}

const EU_GENERAL: EmergencyNumber = EmergencyNumber {
    number: "112",
    label_key: "guest.numbers.general",
    primary: true,
};

const FRANCE: &[EmergencyNumber] = &[
    EU_GENERAL,
    EmergencyNumber {
        number: "15",
        label_key: "guest.numbers.medical",
        primary: false,
    },
    EmergencyNumber {
        number: "18",
        label_key: "guest.numbers.fire",
        primary: false,
    },
];

/// Le pays, lu dans le sous-tag de région de la locale du logement (`fr-FR` → `FR`).
///
/// **La locale du logement, jamais celle du lecteur.** Les surfaces passaient `ctx.locale`, qui est
/// celle du voyageur quand il en a choisi une : un francophone en Espagne recevait le 15 et le 18,
/// qui ne sonnent nulle part là-bas, et un anglophone en France perdait les deux.
///
/// ponytail: la locale est ce que le contexte donne ; un vrai code pays sur la propriété serait
/// plus sûr — un hôte belge qui garde `fr-FR` reçoit les numéros français. Le 112 reste juste
/// dans les deux cas, et c'est lui qu'on affiche en premier.
fn region_of(locale: &str) -> Option<&str> {
    locale.split(['-', '_']).nth(1).filter(|r| r.len() == 2)
}

/// Les numéros à afficher pour ce logement.
pub fn for_locale(locale: &str) -> &'static [EmergencyNumber] {
    match region_of(locale).map(str::to_ascii_uppercase).as_deref() {
        Some("FR") => FRANCE,
        _ => std::slice::from_ref(&EU_GENERAL),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_france_porte_ses_trois_numeros() {
        let numbers = for_locale("fr-FR");
        assert_eq!(numbers.len(), 3);
        assert_eq!(numbers[0].number, "112");
        assert!(
            numbers[0].primary,
            "le 112 est celui qu'on compose sans savoir"
        );
    }

    /// Jamais de numéro inventé : composer un mauvais numéro en urgence coûte des minutes.
    #[test]
    fn un_pays_inconnu_ne_recoit_que_le_112() {
        for locale in ["es-ES", "de-DE", "ja-JP", "en-US", "xx", ""] {
            let numbers = for_locale(locale);
            assert_eq!(numbers.len(), 1, "{locale}");
            assert_eq!(numbers[0].number, "112");
        }
    }

    #[test]
    fn la_region_se_lit_quel_que_soit_le_separateur() {
        assert_eq!(for_locale("fr_FR").len(), 3);
        assert_eq!(for_locale("fr-fr").len(), 3);
    }
}
