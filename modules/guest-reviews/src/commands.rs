//! Module commands — Portaki review submit.

use portaki_sdk::host;
use portaki_sdk::host::email::{
    self, EmailAudience, EmailBlock, EmailPair, LocalizedEmailText, ModuleEmailCta,
    ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use crate::email_text;

/// Longest guest name in the rating lines, in chars.
const GUEST_NAME_ROW_MAX_CHARS: usize = 80;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReviewArgs {
    /// La note, de 1 à 5. Lue en nombre comme en texte : voir [`deserialize_rating`].
    #[serde(deserialize_with = "deserialize_rating")]
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
    /// « Votre avis peut apparaître sur la page du logement » : décochée par défaut.
    #[serde(default, deserialize_with = "deserialize_consent")]
    pub public_consent: bool,
}

/// La case du formulaire : une case HTML cochée envoie `"on"`, décochée n'envoie rien (le
/// `#[serde(default)]` la lit alors `false`). Un booléen se lit aussi. Le reste vaut `false` : on
/// ne publie jamais un avis sur un doute.
fn deserialize_consent<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(checked) => checked,
        serde_json::Value::String(text) => {
            matches!(
                text.trim().to_ascii_lowercase().as_str(),
                "on" | "true" | "1"
            )
        }
        _ => false,
    })
}

/// La note, telle que le formulaire l'envoie : un nombre, ou le `Select` en texte (`"5"`).
///
/// Tout ce qui sort d'un formulaire HTML est une chaîne, et rien sur le chemin ne la convertit
/// d'après le type déclaré — ni le livret, qui poste ce que `FormData` lui donne, ni la
/// plateforme, qui passe les arguments tels quels. Un `u8` nu les faisait donc refuser par serde
/// (`wasm_params_invalid`), et le voyageur n'enregistrait jamais sa note. Le contrôle de 1 à 5 de
/// [`submit_review`] reste le seul juge de la valeur : ici on ne fait que la lire.
fn deserialize_rating<'de, D>(deserializer: D) -> std::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let not_a_rating =
        |raw: &dyn std::fmt::Display| serde::de::Error::custom(format!("not a rating: {raw}"));
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(number) => number
            .as_u64()
            .and_then(|number| u8::try_from(number).ok())
            .ok_or_else(|| not_a_rating(&number)),
        serde_json::Value::String(text) => text.trim().parse().map_err(|_| not_a_rating(&text)),
        other => Err(not_a_rating(&other)),
    }
}

/// Where every review used to go, one blob for the property: still read, never written.
const LEGACY_REVIEWS_KEY: &str = "reviews";
/// First per-stay key (`review:<stay_id>`), outside the platform's stay prefix: still read.
const LEGACY_REVIEW_KEY_PREFIX: &str = "review:";

/// One review per stay, under `stay:<stay_id>:` — the prefix the platform deletes with the stay.
/// Same format as `portaki_sdk::host::kv::stay_key`; switch to it once the SDK pin moves.
fn review_key(stay_id: Uuid) -> String {
    format!("stay:{stay_id}:review")
}

/// One review as stored in KV. Reviews stored before the date was kept have no `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReview {
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub guest_name: Option<String>,
    /// Le voyageur accepte que l'avis paraisse sur la page publique du logement. Absent sur les
    /// avis d'avant la case : jamais publiés.
    #[serde(default)]
    pub public_consent: bool,
}

/// L'avis déjà laissé pour ce séjour, s'il y en a un.
///
/// Les deux emplacements, comme le fait l'envoi : un avis d'avant la clé par séjour vit encore
/// sous l'ancien préfixe, et son auteur ne doit pas se voir proposer de noter une seconde fois.
pub(crate) fn review_for_stay(stay_id: Uuid) -> Result<Option<StoredReview>> {
    if let Some(review) = read_json::<StoredReview>(&review_key(stay_id))? {
        return Ok(Some(review));
    }
    read_json(&format!("{LEGACY_REVIEW_KEY_PREFIX}{stay_id}"))
}

/// Les avis que leur voyageur a laissés publier, avec leur séjour (l'identifiant que l'hôte
/// choisit), du plus ancien au plus récent. Seule la clé par séjour peut porter le
/// consentement : le blob `reviews` et les clés `review:` datent d'avant la case.
pub(crate) fn consented_reviews() -> Result<Vec<(String, StoredReview)>> {
    let mut reviews = Vec::new();
    for key in host::kv::list("stay:")? {
        let Some(stay_id) = key
            .strip_prefix("stay:")
            .and_then(|rest| rest.strip_suffix(":review"))
        else {
            continue;
        };
        if let Some(review) = read_json::<StoredReview>(&key)?.filter(|r| r.public_consent) {
            reviews.push((stay_id.to_string(), review));
        }
    }
    reviews.sort_by_key(|(_, review)| review.at);
    Ok(reviews)
}

/// Every review of the property, oldest first.
pub fn load_reviews() -> Result<Vec<StoredReview>> {
    let mut reviews: Vec<StoredReview> = read_json(LEGACY_REVIEWS_KEY)?.unwrap_or_default();
    let per_stay = host::kv::list("stay:")?
        .into_iter()
        .filter(|key| key.ends_with(":review"));
    for key in per_stay.chain(host::kv::list(LEGACY_REVIEW_KEY_PREFIX)?) {
        reviews.extend(read_json::<StoredReview>(&key)?);
    }
    reviews.sort_by_key(|review| review.at);
    Ok(reviews)
}

fn read_json<T: serde::de::DeserializeOwned>(key: &str) -> Result<Option<T>> {
    Ok(host::kv::get(key)?.and_then(|bytes| serde_json::from_slice(&bytes).ok()))
}

/// The guest has checked in — or the stay has no check-in date to say otherwise. Before
/// arrival there is no stay to review.
pub(crate) fn has_arrived(ctx: &Context) -> Result<bool> {
    match ctx.stay.as_ref().and_then(|stay| stay.checkin_at) {
        Some(checkin_at) => Ok(host::time::now()? >= checkin_at),
        None => Ok(true),
    }
}

#[portaki_sdk::command(
    name = "submitReview",
    guest,
    example(
        label = "Cinq étoiles",
        input = r#"{"rating":5,"comment":"Appartement lumineux et très bien situé, on reviendra !"}"#
    ),
    example(label = "Note sans commentaire", input = r#"{"rating":3}"#)
)]
pub fn submit_review(ctx: Context, args: SubmitReviewArgs) -> Result<()> {
    // La note se propose toujours (spec §2.1) : plus d'interrupteur à vérifier.
    if !has_arrived(&ctx)? {
        return Err(PortakiError::Host("review_before_arrival".into()));
    }
    let Some(stay_id) = ctx.stay.as_ref().map(|stay| stay.stay_id) else {
        return Err(PortakiError::Host("review_needs_stay".into()));
    };
    let key = review_key(stay_id);
    // ponytail: read-then-write, no compare-and-set in KV — two submits racing within the same
    // instant could both pass; a table with a unique stay_id closes it.
    if host::kv::get(&key)?.is_some()
        || host::kv::get(&format!("{LEGACY_REVIEW_KEY_PREFIX}{stay_id}"))?.is_some()
    {
        return Err(PortakiError::Host("review_already_submitted".into()));
    }

    if !(1..=5).contains(&args.rating) {
        return Err(PortakiError::Host(format!(
            "rating must be 1-5, got {}",
            args.rating
        )));
    }

    let comment = args.comment.trim().to_string();
    let guest_name = ctx
        .guest
        .as_ref()
        .and_then(|g| g.display_name.clone())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());
    let review = StoredReview {
        rating: args.rating,
        comment: comment.clone(),
        at: Some(host::time::now()?),
        guest_name: guest_name.clone(),
        public_consent: args.public_consent,
    };
    let bytes = serde_json::to_vec(&review)
        .map_err(|error| PortakiError::Storage(format!("review serialize: {error}")))?;
    host::kv::set(&key, &bytes, None)?;

    let guest_name = guest_name.unwrap_or_else(|| "Voyageur".to_string());

    // Guest text is quoted within a fixed length; the stored review keeps it whole.
    let quoted_name = email_text::quote_guest_text(&guest_name);
    let quoted_comment = email_text::quote_guest_text(&comment);
    let truncated = quoted_name.truncated || quoted_comment.truncated;

    let stars = "★".repeat(args.rating as usize) + &"☆".repeat(5 - args.rating as usize);
    let rating = format!("{stars} ({}/5)", args.rating);
    let body = if comment.is_empty() {
        format!("{} — {rating}", quoted_name.text)
    } else {
        quoted_comment.text.clone()
    };
    // A block line holds 200 chars: the name is cut shorter than the body quote.
    let short_name = email_text::clip_chars(&guest_name, GUEST_NAME_ROW_MAX_CHARS);
    let blocks = vec![EmailBlock::rows([
        EmailPair::new(
            LocalizedEmailText::new("Voyageur", "Guest"),
            LocalizedEmailText::both(short_name.text),
        ),
        EmailPair::new(
            LocalizedEmailText::new("Note", "Rating"),
            LocalizedEmailText::both(rating),
        ),
    ])];

    // The review is saved: a refused email is logged, it does not fail the guest's submit.
    let sent = email::send(&SendEmailArgs {
        email_id: "review-submitted".into(),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Vous avez reçu un nouvel avis",
                "You received a new review",
            ),
            eyebrow: Some(LocalizedEmailText::both("Avis")),
            title: Some(LocalizedEmailText::new(
                "Nouvel avis voyageur",
                "New guest review",
            )),
            body: LocalizedEmailText::both(body),
            cta: Some(ModuleEmailCta {
                // No URL: with `property_id` set, the platform links the property page.
                label: email_text::cta_label(
                    truncated,
                    LocalizedEmailText::new("Voir le logement", "View property"),
                ),
                url: None,
                portaki_action: None,
            }),
            blocks,
            ..Default::default()
        },
        stay_id: Some(stay_id),
        property_id: Some(ctx.property_id),
        action_url: None,
    });
    if let Err(error) = sent {
        email_text::log_send_failure("guest_reviews_host_email_failed", &error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Le `Select` du formulaire envoie la note comme texte : `"5"` doit se lire.
    ///
    /// Avec un `u8` nu, serde refusait les arguments — `wasm_params_invalid` — et la note du
    /// voyageur n'arrivait jamais au module. Le contrôle de 1 à 5 de [`submit_review`] reste le
    /// seul juge de la valeur.
    #[test]
    fn a_rating_sent_as_text_still_reads() {
        let parsed = |value| serde_json::from_value::<SubmitReviewArgs>(value).expect("args");
        assert_eq!(
            parsed(json!({ "rating": "5", "comment": "Super" })).rating,
            5
        );
        assert_eq!(parsed(json!({ "rating": " 3 " })).rating, 3);
        // Un nombre reste lisible : les exemples du manifeste en envoient.
        assert_eq!(parsed(json!({ "rating": 4 })).rating, 4);
    }

    /// La case décochée n'envoie rien, cochée elle envoie `"on"`.
    #[test]
    fn the_consent_box_reads_as_html_sends_it() {
        let consent = |value| {
            serde_json::from_value::<SubmitReviewArgs>(value)
                .expect("args")
                .public_consent
        };
        assert!(!consent(json!({ "rating": 5 })));
        assert!(consent(json!({ "rating": 5, "public_consent": "on" })));
        assert!(consent(json!({ "rating": 5, "public_consent": true })));
        assert!(!consent(json!({ "rating": 5, "public_consent": "off" })));
        assert!(!consent(json!({ "rating": 5, "public_consent": false })));
    }

    /// Un avis d'avant la case n'a pas le champ : il n'est pas consenti.
    #[test]
    fn an_old_review_is_not_consented() {
        let old: StoredReview = serde_json::from_value(
            json!({ "rating": 5, "comment": "Super", "guest_name": "Claire Martin" }),
        )
        .expect("old review");
        assert!(!old.public_consent);
    }

    /// Ce qui n'est pas une note est refusé à la lecture, et non lu comme zéro : un champ vide
    /// n'est pas une note d'une étoile.
    #[test]
    fn what_is_not_a_rating_is_refused() {
        for value in [
            json!({ "rating": "" }),
            json!({ "rating": "cinq" }),
            json!({ "rating": 2.5 }),
        ] {
            assert!(
                serde_json::from_value::<SubmitReviewArgs>(value.clone()).is_err(),
                "{value}"
            );
        }
    }
}
