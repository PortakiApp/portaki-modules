//! Bloc « Avis » de la page publique du logement (`property.public`) : la note moyenne et le
//! nombre de séjours notés, sur les avis consentis, puis les 2 à 6 avis choisis par l'hôte.
//!
//! Rien d'autre ne sort : ni avis non consenti, ni nom de famille (le prénom seul), ni séjour.
//! Désactivé, ou moins de deux avis choisis encore consentis : une `Section` sans enfant, que la
//! plateforme masque.

use chrono::Datelike;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Badge, KeyValue, Section, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::commands::{consented_reviews, StoredReview};
use crate::config::{ModuleConfig, PUBLIC_REVIEWS_MAX, PUBLIC_REVIEWS_MIN};
use crate::email_text::clip_chars;

/// Longest quote on the public page, in chars.
const QUOTE_MAX_CHARS: usize = 280;

/// Les avis choisis par l'hôte encore consentis et porteurs d'un texte, dans son ordre, sans
/// doublon, six au plus. Un identifiant hors des avis consentis est ignoré.
pub(crate) fn chosen<'a>(
    config: &ModuleConfig,
    consented: &'a [(String, StoredReview)],
) -> Vec<&'a StoredReview> {
    let mut picked: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for id in &config.public_reviews {
        if out.len() == PUBLIC_REVIEWS_MAX || picked.contains(&id.as_str()) {
            continue;
        }
        if let Some((_, review)) = consented.iter().find(|(stay, review)| {
            stay == id && review.public_consent && !review.comment.trim().is_empty()
        }) {
            picked.push(id);
            out.push(review);
        }
    }
    out
}

/// Le prénom seul : le premier mot du nom affiché.
pub(crate) fn first_name(review: &StoredReview) -> Option<&str> {
    review.guest_name.as_deref()?.split_whitespace().next()
}

/// La langue à deux lettres du visiteur (`fr-FR` → `fr`).
fn lang(locale: &str) -> String {
    locale
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// « Claire · juillet 2026 », ou ce qu'il en reste.
fn byline(review: &StoredReview) -> String {
    let date = review.at.map(|at| {
        let month = t!(format!("public.month.{}", at.month()).as_str()).unwrap_or_default();
        format!("{month} {}", at.year()).trim().to_string()
    });
    [first_name(review).map(str::to_string), date]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// La note moyenne (« 4,8 ») et le nombre de séjours notés, sur les seuls avis consentis.
pub(crate) fn summary(consented: &[(String, StoredReview)], comma: bool) -> (String, usize) {
    let all: Vec<StoredReview> = consented
        .iter()
        .filter(|(_, review)| review.public_consent)
        .map(|(_, review)| review.clone())
        .collect();
    (
        crate::host::average(&all, comma).unwrap_or_default(),
        all.len(),
    )
}

/// L'arbre public, pur : testé sans hôte.
pub(crate) fn build(
    config: &ModuleConfig,
    consented: &[(String, StoredReview)],
    locale: &str,
) -> Surface {
    let mut section = Section::new()
        .title("i18n:public.title")
        .subtitle("i18n:public.eyebrow");
    let chosen = chosen(config, consented);
    if config.public_enabled && chosen.len() >= PUBLIC_REVIEWS_MIN {
        let comma = !matches!(lang(locale).as_str(), "en" | "ja" | "zh" | "ar");
        let (average, count) = summary(consented, comma);
        let count = count.to_string();
        let mut children: Vec<Component> = vec![
            KeyValue::new()
                .key("i18n:public.average")
                .value(format!("{average} / 5"))
                .icon(IconName::Star)
                .into(),
            Badge::new()
                .label(t!("public.count", count = count.as_str()).unwrap_or(count))
                .into(),
        ];
        for review in chosen {
            let quote = clip_chars(review.comment.trim(), QUOTE_MAX_CHARS).text;
            children.push(
                Stack::new()
                    .gap(4.0)
                    // Sans guillemets : le cadre du bloc cite déjà, et « » n'est pas de toutes les
                    // langues.
                    .child(Text::new().text(quote).variant(TextVariant::Body))
                    .child(
                        Text::new()
                            .text(byline(review))
                            .variant(TextVariant::Caption),
                    )
                    .into(),
            );
        }
        section = section.child(Stack::new().gap(16.0).children(children));
    }
    Surface::new(section).with_id(PROPERTY_PUBLIC)
}

/// Rendu pour un visiteur sans séjour, avec la config publiée.
#[portaki_sdk::surface(guest, id = "property.public")]
pub fn render_property_public(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    if !config.public_enabled {
        return Ok(build(&config, &[], &ctx.locale));
    }
    Ok(build(&config, &consented_reviews()?, &ctx.locale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn review(name: &str, rating: u8, comment: &str, consent: bool) -> StoredReview {
        StoredReview {
            rating,
            comment: comment.into(),
            at: Some("2026-07-14T10:00:00Z".parse().unwrap()),
            guest_name: Some(name.into()),
            public_consent: consent,
            link_offered: false,
        }
    }

    fn config(enabled: bool, ids: &[&str]) -> ModuleConfig {
        ModuleConfig {
            public_enabled: enabled,
            public_reviews: ids.iter().map(|id| id.to_string()).collect(),
            ..ModuleConfig::default()
        }
    }

    fn children(surface: &Surface) -> Value {
        serde_json::to_value(surface).unwrap()["root"]["children"].clone()
    }

    #[test]
    fn disabled_renders_an_empty_section() {
        let consented = vec![
            ("a".into(), review("Claire Martin", 5, "Parfait", true)),
            ("b".into(), review("Paul Durand", 4, "Très bien", true)),
        ];
        let surface = build(&config(false, &["a", "b"]), &consented, "fr-FR");
        let json = serde_json::to_value(&surface).unwrap();
        assert_eq!(json["root"]["type"], "Section", "{json}");
        assert_eq!(json["root"]["title"], "i18n:public.title");
        assert!(
            children(&surface).as_array().is_none_or(Vec::is_empty),
            "{json}"
        );
    }

    #[test]
    fn a_chosen_review_without_consent_never_appears() {
        // `consented_reviews` ne rend que les consentis ; même passé ici, il est écarté.
        let consented = vec![
            ("a".into(), review("Claire Martin", 5, "Parfait", true)),
            ("b".into(), review("Paul Durand", 4, "Très bien", true)),
            ("c".into(), review("Zoé Secret", 1, "Privé", false)),
        ];
        let surface = build(&config(true, &["a", "b", "c"]), &consented, "fr-FR");
        let text = serde_json::to_string(&surface).unwrap();
        assert!(
            text.contains("Parfait") && text.contains("Très bien"),
            "{text}"
        );
        assert!(!text.contains("Privé") && !text.contains("Zoé"), "{text}");
    }

    #[test]
    fn last_names_and_stay_ids_are_stripped() {
        let consented = vec![
            ("stay-1".into(), review("Claire Martin", 5, "Parfait", true)),
            ("stay-2".into(), review("Paul  Durand", 4, "Bien", true)),
        ];
        let text = serde_json::to_string(&build(
            &config(true, &["stay-1", "stay-2"]),
            &consented,
            "fr-FR",
        ))
        .unwrap();
        assert!(text.contains("Claire") && text.contains("Paul"), "{text}");
        assert!(
            !text.contains("Martin") && !text.contains("Durand"),
            "{text}"
        );
        assert!(!text.contains("stay-"), "{text}");
    }

    #[test]
    fn fewer_than_two_chosen_consented_renders_nothing() {
        let consented = vec![("a".into(), review("Claire Martin", 5, "Parfait", true))];
        let surface = build(&config(true, &["a", "gone"]), &consented, "fr-FR");
        assert!(children(&surface).as_array().is_none_or(Vec::is_empty));
    }

    #[test]
    fn average_and_count_cover_every_consented_review() {
        let consented = vec![
            ("a".into(), review("Claire", 5, "Parfait", true)),
            ("b".into(), review("Paul", 4, "Bien", true)),
            ("c".into(), review("Inès", 3, "", true)),
        ];
        let mut with_private = consented.clone();
        with_private.push(("d".into(), review("Zoé", 1, "Non", false)));
        assert_eq!(summary(&with_private, true), ("4,0".to_string(), 3));
        assert_eq!(summary(&with_private, false).0, "4.0");
        let json = serde_json::to_value(build(&config(true, &["a", "b"]), &with_private, "fr-FR"))
            .unwrap();
        let stack = &json["root"]["children"][0]["children"];
        assert_eq!(stack[0]["value"], "4,0 / 5", "{json}");
        // La note, le nombre, puis les deux avis choisis.
        assert_eq!(stack.as_array().unwrap().len(), 4, "{json}");
    }
}
