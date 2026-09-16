//! Liens d'affiliation GetYourGuide — identifiant partenaire et fabrication des URL.
//!
//! Le module ne parle jamais à GetYourGuide : pas de connecteur, pas de permission
//! supplémentaire, pas de script ni d'iframe côté livret. On ne produit que des URL
//! `https` que le shell ouvre dans le navigateur du voyageur.

/// Identifiant d'affilié GetYourGuide de Portaki.
///
/// Il appartient à la plateforme, pas à l'hôte : la commission d'une réservation
/// revient à Portaki quelle que soit la propriété qui a affiché le lien. Il n'y en a
/// donc qu'un, ici, et non un champ de configuration par logement.
///
/// Il est **public par nature** — il voyage dans chaque lien ouvert par un voyageur et
/// s'affiche dans sa barre d'adresse. Ce n'est pas un secret : il n'a rien à faire dans
/// un coffre de credentials, et le lire dans ce fichier ne donne accès à rien.
///
/// En contrepartie, le changer est une **release du module** : la constante est compilée
/// dans le Wasm, republier est le seul moyen de la faire bouger sur les logements.
///
/// Il est **vide tant que le compte d'affiliation n'existe pas**. Vide, les liens partent
/// sans aucun paramètre partenaire : de simples liens GetYourGuide, aucune commission, et
/// rien d'autre ne change — ni la section, ni les liens choisis par l'hôte, ni la mention
/// d'affiliation, qui reste affichée parce qu'elle décrit l'intention du lien et non son
/// rendement du jour.
pub const PARTNER_ID: &str = "";

/// Nom du paramètre de requête qui porte [`PARTNER_ID`].
///
/// À confirmer dans le portail partenaire GetYourGuide le jour où le compte existe :
/// c'est la seule valeur de ce fichier qui dépend d'eux et non de nous.
pub const PARTNER_QUERY_PARAM: &str = "partner_id";

/// Recherche GetYourGuide par destination.
const SEARCH_BASE: &str = "https://www.getyourguide.com/s/";

/// Nombre maximum de liens choisis par l'hôte.
pub const MAX_CURATED_LINKS: usize = 10;

/// Domaine des liens longs.
const GYG_DOMAIN: &str = "getyourguide.com";

/// Domaine des liens courts. Ils portent déjà un identifiant : on n'y touche pas.
const GYG_SHORT_DOMAIN: &str = "gyg.me";

/// Pourquoi une URL proposée par l'hôte a été refusée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuratedUrlError {
    /// Champ laissé vide — la ligne est simplement ignorée, ce n'est pas une faute.
    Empty,
    /// Ni GetYourGuide ni un lien court `gyg.me` : refusé à l'enregistrement.
    NotGetYourGuide,
}

/// L'identifiant partenaire, ou `None` tant qu'il est vide.
pub fn partner_id() -> Option<&'static str> {
    let trimmed = PARTNER_ID.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// URL de recherche GetYourGuide pour une destination.
///
/// `None` quand la destination est vide — appelé ainsi, le module n'affiche pas la section
/// plutôt que de proposer une recherche sur rien.
pub fn search_url(destination: &str) -> Option<String> {
    let destination = destination.trim();
    if destination.is_empty() {
        return None;
    }
    let mut url = format!("{SEARCH_BASE}?q={}", percent_encode(destination));
    if let Some(id) = partner_id() {
        url.push('&');
        url.push_str(PARTNER_QUERY_PARAM);
        url.push('=');
        url.push_str(&percent_encode(id));
    }
    Some(url)
}

/// Valide et normalise une URL choisie par l'hôte.
///
/// Seuls `getyourguide.com` (tout sous-domaine, tout chemin) et le raccourcisseur `gyg.me`
/// passent. Les liens longs repartent avec [`PARTNER_QUERY_PARAM`] posé — ajouté s'il
/// manquait, remplacé s'il était là, retiré tant que [`PARTNER_ID`] est vide. Les liens
/// courts sortent tels quels : leur identifiant est déjà dans le chemin, le réécrire le
/// casserait.
///
/// La sortie est toujours en `https` : un lien sans schéma ou en `http` est relevé, tout
/// autre schéma est refusé.
pub fn normalize_curated_url(raw: &str) -> Result<String, CuratedUrlError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CuratedUrlError::Empty);
    }

    let lower = trimmed.to_ascii_lowercase();
    let rest = if let Some(stripped) = lower.strip_prefix("https://") {
        &trimmed[trimmed.len() - stripped.len()..]
    } else if let Some(stripped) = lower.strip_prefix("http://") {
        &trimmed[trimmed.len() - stripped.len()..]
    } else {
        // Pas de schéma explicite. Un `:` avant le premier `/` en est un (`javascript:`,
        // `ftp://`, …) : seuls http et https sont arrivés jusqu'ici, le reste est refusé.
        let head = trimmed.split(['/', '?', '#']).next().unwrap_or(trimmed);
        if head.contains(':') {
            return Err(CuratedUrlError::NotGetYourGuide);
        }
        trimmed
    };

    let host = host_of(rest).to_ascii_lowercase();
    let https = format!("https://{rest}");

    if is_domain_or_subdomain(&host, GYG_SHORT_DOMAIN) {
        return Ok(https);
    }
    if is_domain_or_subdomain(&host, GYG_DOMAIN) {
        return Ok(set_partner_param(&https));
    }
    Err(CuratedUrlError::NotGetYourGuide)
}

/// `host` est exactement `domain`, ou l'un de ses sous-domaines.
///
/// Le point est dans la comparaison : sans lui, `evil-getyourguide.com` passerait.
fn is_domain_or_subdomain(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Nom d'hôte d'une URL privée de son schéma.
///
/// L'autorité s'arrête au premier `/`, `?` ou `#`. Ce qui précède un `@` est un userinfo
/// (`https://getyourguide.com@ailleurs.example/` pointe vers `ailleurs.example`), ce qui
/// suit un `:` est un port.
fn host_of(rest: &str) -> &str {
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let after_userinfo = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    after_userinfo
        .split(':')
        .next()
        .unwrap_or(after_userinfo)
        .trim_matches(['[', ']'])
}

/// Repose [`PARTNER_QUERY_PARAM`] sur une URL, en retirant d'abord celui qui s'y trouvait.
///
/// Le fragment reste en queue : il n'appartient pas à la query.
fn set_partner_param(url: &str) -> String {
    let (head, fragment) = match url.split_once('#') {
        Some((head, fragment)) => (head, Some(fragment)),
        None => (url, None),
    };
    let (base, query) = match head.split_once('?') {
        Some((base, query)) => (base, query),
        None => (head, ""),
    };

    let mut out = String::with_capacity(url.len() + 32);
    out.push_str(base);
    let mut first = true;
    for pair in query
        .split('&')
        .filter(|pair| !pair.is_empty() && !is_partner_pair(pair))
    {
        out.push(if first { '?' } else { '&' });
        first = false;
        out.push_str(pair);
    }
    if let Some(id) = partner_id() {
        out.push(if first { '?' } else { '&' });
        out.push_str(PARTNER_QUERY_PARAM);
        out.push('=');
        out.push_str(&percent_encode(id));
    }
    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }
    out
}

fn is_partner_pair(pair: &str) -> bool {
    match pair.split_once('=') {
        Some((name, _)) => name == PARTNER_QUERY_PARAM,
        None => pair == PARTNER_QUERY_PARAM,
    }
}

/// Encodage pour-cent d'une valeur de query (RFC 3986).
///
/// Tout ce qui n'est pas non-réservé part en `%XX` sur les octets UTF-8 — l'espace inclus,
/// en `%20` et non en `+` : ce dernier n'est une convention que pour les formulaires.
fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Garde-fou : tout ce fichier décrit le comportement « identifiant vide », et les
    /// assertions ci-dessous seraient muettes si quelqu'un le remplissait sans les relire.
    #[test]
    fn partner_id_is_still_empty() {
        assert_eq!(PARTNER_ID, "");
        assert_eq!(partner_id(), None);
    }

    #[test]
    fn search_url_percent_encodes_the_destination() {
        assert_eq!(
            search_url("Aix-en-Provence"),
            Some("https://www.getyourguide.com/s/?q=Aix-en-Provence".to_string())
        );
        assert_eq!(
            search_url(" Saint-Jean-Cap-Ferrat "),
            Some("https://www.getyourguide.com/s/?q=Saint-Jean-Cap-Ferrat".to_string())
        );
        assert_eq!(
            search_url("Nîmes & Uzès"),
            Some("https://www.getyourguide.com/s/?q=N%C3%AEmes%20%26%20Uz%C3%A8s".to_string())
        );
    }

    #[test]
    fn search_url_needs_a_destination() {
        assert_eq!(search_url(""), None);
        assert_eq!(search_url("   "), None);
    }

    #[test]
    fn empty_partner_id_yields_plain_links() {
        // Le compte d'affiliation n'existe pas encore : aucun paramètre partenaire nulle part.
        let search = search_url("Antibes").expect("search url");
        assert_eq!(search, "https://www.getyourguide.com/s/?q=Antibes");
        assert!(!search.contains(PARTNER_QUERY_PARAM));

        let curated =
            normalize_curated_url("https://www.getyourguide.com/antibes-l1234/").expect("curated");
        assert_eq!(curated, "https://www.getyourguide.com/antibes-l1234/");
        assert!(!curated.contains(PARTNER_QUERY_PARAM));
    }

    #[test]
    fn empty_partner_id_strips_an_id_pasted_by_the_host() {
        // La commission revient à la plateforme, pas à l'hôte : un identifiant collé dans
        // l'URL ne survit pas à l'enregistrement.
        let out =
            normalize_curated_url("https://www.getyourguide.com/paris-l16/?partner_id=someone")
                .expect("curated");
        assert_eq!(out, "https://www.getyourguide.com/paris-l16/");
    }

    #[test]
    fn curated_url_keeps_its_other_query_params_and_fragment() {
        let out = normalize_curated_url(
            "https://www.getyourguide.com/paris-l16/eiffel-t1?cmp=ete&lc=fr#avis",
        )
        .expect("curated");
        assert_eq!(
            out,
            "https://www.getyourguide.com/paris-l16/eiffel-t1?cmp=ete&lc=fr#avis"
        );
    }

    #[test]
    fn curated_url_accepts_any_subdomain_and_upgrades_to_https() {
        for raw in [
            "https://getyourguide.com/x",
            "http://www.getyourguide.com/x",
            "www.getyourguide.com/x",
            "https://FR.GetYourGuide.com/x",
        ] {
            let out = normalize_curated_url(raw).unwrap_or_else(|_| panic!("{raw}"));
            assert!(out.starts_with("https://"), "{raw} -> {out}");
        }
    }

    #[test]
    fn short_links_are_left_alone() {
        let out = normalize_curated_url("https://gyg.me/aBcD12").expect("short link");
        assert_eq!(out, "https://gyg.me/aBcD12");
        assert!(!out.contains(PARTNER_QUERY_PARAM));
    }

    #[test]
    fn foreign_domains_are_refused() {
        for raw in [
            "https://example.com/tour",
            "https://viator.com/paris",
            // Le suffixe seul ne suffit pas — le point de séparation est vérifié.
            "https://evil-getyourguide.com/x",
            "https://getyourguide.com.evil.example/x",
            // Le userinfo ne déguise pas l'hôte réel.
            "https://www.getyourguide.com@evil.example/x",
            // Un autre schéma ne passe pas, même vers le bon domaine.
            "javascript:alert(1)",
            "ftp://www.getyourguide.com/x",
            "data:text/html,<script>",
        ] {
            assert_eq!(
                normalize_curated_url(raw),
                Err(CuratedUrlError::NotGetYourGuide),
                "{raw}"
            );
        }
    }

    #[test]
    fn blank_url_is_an_empty_row_not_a_mistake() {
        assert_eq!(normalize_curated_url("   "), Err(CuratedUrlError::Empty));
    }

    #[test]
    fn partner_param_is_set_then_replaced_when_an_id_exists() {
        // `PARTNER_ID` est vide aujourd'hui ; on vérifie ici la mécanique de pose et de
        // remplacement, qui est ce qui change le jour où le compte existe.
        assert_eq!(
            set_partner_param_with("https://www.getyourguide.com/x", Some("portaki")),
            "https://www.getyourguide.com/x?partner_id=portaki"
        );
        assert_eq!(
            set_partner_param_with(
                "https://www.getyourguide.com/x?partner_id=ancien&lc=fr",
                Some("portaki")
            ),
            "https://www.getyourguide.com/x?lc=fr&partner_id=portaki"
        );
        assert_eq!(
            set_partner_param_with("https://www.getyourguide.com/x?partner_id=ancien", None),
            "https://www.getyourguide.com/x"
        );
    }

    /// Variante paramétrable de [`set_partner_param`] — la vraie lit la constante, qui est
    /// vide, et ne pourrait donc pas montrer le remplacement.
    fn set_partner_param_with(url: &str, id: Option<&str>) -> String {
        let (head, fragment) = match url.split_once('#') {
            Some((head, fragment)) => (head, Some(fragment)),
            None => (url, None),
        };
        let (base, query) = match head.split_once('?') {
            Some((base, query)) => (base, query),
            None => (head, ""),
        };
        let mut out = String::from(base);
        let mut first = true;
        for pair in query
            .split('&')
            .filter(|pair| !pair.is_empty() && !is_partner_pair(pair))
        {
            out.push(if first { '?' } else { '&' });
            first = false;
            out.push_str(pair);
        }
        if let Some(id) = id {
            out.push(if first { '?' } else { '&' });
            out.push_str(PARTNER_QUERY_PARAM);
            out.push('=');
            out.push_str(&percent_encode(id));
        }
        if let Some(fragment) = fragment {
            out.push('#');
            out.push_str(fragment);
        }
        out
    }
}
