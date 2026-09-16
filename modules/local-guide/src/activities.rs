//! Section « Activités & billets » — d'où vient la destination, et ce qu'en voit le voyageur.
//!
//! Rien n'est appelé à distance : la section se résout entièrement à partir du contexte
//! propriété déjà présent dans l'invocation et de la configuration hôte.

use crate::affiliate::{self, MAX_CURATED_LINKS};
use crate::config::ActivitiesConfig;

/// La section une fois résolue — ou rien, et alors elle ne s'affiche pas.
pub struct ActivitiesView {
    /// Nom du lieu tel qu'affiché au voyageur. Vide = le libellé neutre prend le relais.
    pub destination: String,
    /// Lien principal : recherche par destination, ou page GetYourGuide collée par l'hôte.
    pub destination_url: String,
    /// Phrase d'introduction de l'hôte. Vide = pas de phrase.
    pub intro: String,
    /// Liens choisis par l'hôte, dans son ordre.
    pub links: Vec<ActivityLink>,
}

/// Un lien choisi par l'hôte.
pub struct ActivityLink {
    /// Intitulé saisi par l'hôte. Vide = le rendu retombe sur une clé i18n.
    pub label: String,
    /// URL normalisée.
    pub url: String,
}

/// Résout la section, ou `None` quand il n'y a rien à afficher.
///
/// Trois façons de ne rien afficher : la section est coupée par l'hôte, aucune destination
/// n'est exploitable, ou la destination ne produit pas d'URL. Aucune n'est une erreur — un
/// logement sans adresse géocodée et sans saisie hôte n'a simplement pas de ville à proposer.
pub fn resolve(
    config: &ActivitiesConfig,
    address: Option<&str>,
    guest_locale: &str,
    property_locale: &str,
) -> Option<ActivitiesView> {
    if !config.enabled {
        return None;
    }
    let destination = destination(config, address)?;

    let links = config
        .links
        .iter()
        .filter_map(|row| {
            // Les URL sont déjà validées à l'enregistrement. On repasse par la
            // normalisation au rendu pour deux raisons : une entrée écrite hors du
            // formulaire ne l'a jamais traversée, et un lien stocké sous un ancien
            // identifiant partenaire doit repartir avec l'actuel.
            let url = affiliate::normalize_curated_url(&row.url).ok()?;
            Some(ActivityLink {
                label: row
                    .label
                    .pick_with_fallback(guest_locale, property_locale)
                    .trim()
                    .to_string(),
                url,
            })
        })
        .take(MAX_CURATED_LINKS)
        .collect();

    Some(ActivitiesView {
        destination: destination.label,
        destination_url: destination.url,
        intro: config
            .intro
            .pick_with_fallback(guest_locale, property_locale)
            .trim()
            .to_string(),
        links,
    })
}

/// La destination résolue : ce que lit le voyageur, et où l'emmène le bouton.
pub struct ResolvedDestination {
    /// Nom du lieu. Vide quand aucun n'est lisible — le rendu passe alors au libellé neutre.
    pub label: String,
    /// URL du bouton.
    pub url: String,
}

/// La destination : la saisie de l'hôte d'abord, la ville de l'adresse ensuite.
///
/// L'ordre se lit comme une règle de préséance et non comme une préférence de qualité :
/// la ville déduite de l'adresse couvre le cas courant sans configuration, et la saisie
/// hôte est là pour la corriger — « Antibes » quand le logement est à Juan-les-Pins, ou
/// quand l'adresse géocodée ne donne rien d'exploitable. Une correction qui ne gagnerait
/// pas ne corrigerait rien.
///
/// Cette saisie vaut nom de lieu **ou** URL de page GetYourGuide. Les deux mènent au même
/// bouton, mais pas au même lien : la recherche `?q=…` choisit ses résultats depuis l'IP du
/// visiteur et ignore la requête pour un voyageur qui arrive de l'étranger ou d'un VPN,
/// tandis que la page de destination collée par l'hôte est juste pour tout le monde. D'où
/// le champ qui accepte les deux, et la feuille hôte qui dit laquelle est fiable.
pub fn destination(
    config: &ActivitiesConfig,
    address: Option<&str>,
) -> Option<ResolvedDestination> {
    let host_value = config.destination.trim();
    if !host_value.is_empty() {
        if affiliate::looks_like_url(host_value) {
            // Refusée à l'enregistrement ; une valeur écrite hors du formulaire peut
            // encore arriver ici, et alors la section ne s'affiche pas.
            let url = affiliate::normalize_curated_url(host_value).ok()?;
            // Le slug porte le nom du lieu ; un lien court n'en a pas, et la ville de
            // l'adresse le remplace avant le libellé neutre.
            let label = affiliate::place_from_url(&url)
                .or_else(|| address.and_then(destination_from_address))
                .unwrap_or_default();
            return Some(ResolvedDestination { label, url });
        }
        return Some(ResolvedDestination {
            label: host_value.to_string(),
            url: affiliate::search_url(host_value)?,
        });
    }

    let city = address.and_then(destination_from_address)?;
    let url = affiliate::search_url(&city)?;
    Some(ResolvedDestination { label: city, url })
}

/// Extrait une ville d'une adresse formatée sur une ligne.
///
/// `PropertyContext` n'expose pas de ville : il y a `address`, une seule chaîne géocodée.
/// On la découpe donc à la virgule, on saute la ligne de voirie et le pays, et on retire
/// un code postal en tête. Ça couvre les formes courantes (`Cannes, France`,
/// `12 rue des Lilas, 06400 Cannes, France`) et se trompera sur d'autres — c'est
/// exactement ce que la saisie hôte est là pour rattraper.
pub fn destination_from_address(address: &str) -> Option<String> {
    let parts: Vec<&str> = address
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return None;
    }

    // Trois segments ou plus : le dernier est le pays, il ne nous intéresse pas.
    let searchable = if parts.len() >= 3 {
        &parts[..parts.len() - 1]
    } else {
        &parts[..]
    };

    searchable
        .iter()
        .filter(|part| !is_street_line(part))
        .find_map(|part| city_from_part(part))
}

/// `12 rue des Lilas` : un nombre en tête et assez de mots derrière pour être une voie.
///
/// Le seuil écarte `06400 Cannes`, qui commence aussi par un nombre mais n'est pas une
/// adresse de voirie.
fn is_street_line(part: &str) -> bool {
    let mut tokens = part.split_whitespace();
    let Some(first) = tokens.next() else {
        return false;
    };
    let leading_number = !first.is_empty() && first.chars().all(|c| c.is_ascii_digit());
    leading_number && tokens.count() >= 2
}

/// Retire un code postal en tête, et refuse ce qui n'a aucune lettre.
fn city_from_part(part: &str) -> Option<String> {
    let tokens: Vec<&str> = part.split_whitespace().collect();
    let without_postcode = match tokens.split_first() {
        Some((first, rest))
            if !rest.is_empty()
                && !first.is_empty()
                && first.chars().all(|c| c.is_ascii_digit()) =>
        {
            rest.join(" ")
        }
        _ => part.to_string(),
    };
    let trimmed = without_postcode.trim();
    if trimmed.chars().any(char::is_alphabetic) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ActivityRow, Localized};

    #[test]
    fn city_is_read_from_the_common_address_shapes() {
        for (address, expected) in [
            ("Cannes, France", "Cannes"),
            ("12 rue des Lilas, 06400 Cannes, France", "Cannes"),
            ("06400 Cannes, France", "Cannes"),
            ("12 rue des Lilas, Cannes", "Cannes"),
            ("Cannes", "Cannes"),
            ("  Saint-Jean-Cap-Ferrat , France ", "Saint-Jean-Cap-Ferrat"),
        ] {
            assert_eq!(
                destination_from_address(address).as_deref(),
                Some(expected),
                "{address}"
            );
        }
    }

    #[test]
    fn an_unusable_address_yields_no_city() {
        for address in ["", "   ", ",,", "06400", "12345, 67890"] {
            assert_eq!(destination_from_address(address), None, "{address:?}");
        }
    }

    fn activities(destination: &str) -> ActivitiesConfig {
        ActivitiesConfig {
            enabled: true,
            destination: destination.into(),
            ..ActivitiesConfig::default()
        }
    }

    #[test]
    fn the_host_override_beats_the_address() {
        let config = activities(" Antibes ");
        let resolved = destination(&config, Some("Cannes, France")).expect("destination");
        assert_eq!(resolved.label, "Antibes");
        assert_eq!(
            resolved.url,
            "https://www.getyourguide.com/s/?q=Antibes&partner_id=CLOQ42U"
        );
        // Et il tient tout seul quand l'adresse n'est pas géocodée.
        assert_eq!(
            destination(&config, None).expect("destination").label,
            "Antibes"
        );
    }

    #[test]
    fn a_pasted_url_becomes_the_link_and_names_its_place() {
        for (raw, expected_url, expected_label) in [
            (
                "https://www.getyourguide.com/cannes-l15/",
                "https://www.getyourguide.com/cannes-l15/?partner_id=CLOQ42U",
                "Cannes",
            ),
            // Relevée en https, et l'identifiant collé avec elle remplacé par le nôtre.
            (
                "www.getyourguide.com/aix-en-provence-l1234/?partner_id=someone",
                "https://www.getyourguide.com/aix-en-provence-l1234/?partner_id=CLOQ42U",
                "Aix en Provence",
            ),
        ] {
            let config = activities(raw);
            let resolved = destination(&config, Some("Cannes, France")).expect("destination");
            assert_eq!(resolved.url, expected_url, "{raw}");
            assert_eq!(resolved.label, expected_label, "{raw}");
        }
    }

    #[test]
    fn a_short_link_keeps_its_url_and_borrows_a_label() {
        let config = activities("https://gyg.me/aBcD12");
        // Le lien court repart intact : son identifiant est déjà dans le chemin.
        let resolved = destination(&config, Some("Cannes, France")).expect("destination");
        assert_eq!(resolved.url, "https://gyg.me/aBcD12");
        // Aucun nom de lieu dans l'URL : la ville de l'adresse le remplace.
        assert_eq!(resolved.label, "Cannes");

        // Et sans adresse, le libellé neutre prendra le relais au rendu.
        let resolved = destination(&config, None).expect("destination");
        assert_eq!(resolved.url, "https://gyg.me/aBcD12");
        assert!(resolved.label.is_empty());
    }

    #[test]
    fn a_destination_url_written_outside_the_form_is_dropped_at_render() {
        // Un domaine étranger est refusé à l'enregistrement ; s'il arrive quand même,
        // la section ne s'affiche pas plutôt que de sortir le lien.
        let config = activities("https://viator.com/paris");
        assert!(destination(&config, Some("Cannes, France")).is_none());
        assert!(resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").is_none());
    }

    #[test]
    fn free_text_still_produces_the_search_url() {
        let config = activities("Nîmes");
        let view = resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").expect("view");
        assert_eq!(view.destination, "Nîmes");
        assert_eq!(
            view.destination_url,
            "https://www.getyourguide.com/s/?q=N%C3%AEmes&partner_id=CLOQ42U"
        );
    }

    #[test]
    fn nothing_renders_without_a_destination() {
        let config = ActivitiesConfig {
            enabled: true,
            ..ActivitiesConfig::default()
        };
        assert!(resolve(&config, None, "fr-FR", "fr-FR").is_none());
        assert!(resolve(&config, Some("   "), "fr-FR", "fr-FR").is_none());
    }

    #[test]
    fn the_section_is_off_until_the_host_turns_it_on() {
        // Opt-in : l'adresse suffirait techniquement à bâtir la recherche, mais tant que
        // l'hôte n'a rien décidé, aucun lien d'affiliation ne part vers le livret.
        let config = ActivitiesConfig::default();
        assert!(!config.enabled);
        assert!(resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").is_none());
    }

    #[test]
    fn nothing_renders_when_the_host_switched_it_off() {
        let config = ActivitiesConfig {
            enabled: false,
            destination: "Antibes".into(),
            ..ActivitiesConfig::default()
        };
        assert!(resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").is_none());
    }

    #[test]
    fn links_keep_the_host_order_and_stop_at_the_cap() {
        let links = (0..MAX_CURATED_LINKS + 4)
            .map(|index| ActivityRow {
                url: format!("https://www.getyourguide.com/tour-{index}"),
                label: Localized::singleton("fr", format!("Sortie {index}")),
            })
            .collect();
        let config = ActivitiesConfig {
            enabled: true,
            links,
            ..ActivitiesConfig::default()
        };
        let view = resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").expect("view");
        assert_eq!(view.links.len(), MAX_CURATED_LINKS);
        // L'URL complète, plutôt qu'un suffixe : l'identifiant partenaire est en queue,
        // et « finit par tour-1 » se confondrait de toute façon avec `tour-10`.
        assert_eq!(
            view.links[0].url,
            "https://www.getyourguide.com/tour-0?partner_id=CLOQ42U"
        );
        assert_eq!(
            view.links[MAX_CURATED_LINKS - 1].url,
            format!(
                "https://www.getyourguide.com/tour-{}?partner_id=CLOQ42U",
                MAX_CURATED_LINKS - 1
            )
        );
        assert_eq!(view.links[3].label, "Sortie 3");
    }

    #[test]
    fn a_row_written_outside_the_form_is_dropped_at_render() {
        let config = ActivitiesConfig {
            enabled: true,
            links: vec![
                ActivityRow {
                    url: "https://viator.com/paris".into(),
                    label: Localized::default(),
                },
                ActivityRow {
                    url: "https://gyg.me/aBcD12".into(),
                    label: Localized::default(),
                },
            ],
            ..ActivitiesConfig::default()
        };
        let view = resolve(&config, Some("Cannes, France"), "fr-FR", "fr-FR").expect("view");
        assert_eq!(view.links.len(), 1);
        assert_eq!(view.links[0].url, "https://gyg.me/aBcD12");
    }
}
