//! Host settings drawer (spec Votre avis §2.1, planche « Avis — réglages »): when to ask, the
//! public review link (any platform, detected from the domain), whether it follows the stay's
//! booking platform, the private comment. The thank-you message stays.

use portaki_sdk::bundle_text;
use portaki_sdk::config::check;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    Badge, Card, ChoiceList, Field, FieldHint, Form, InlineNotice, Page, Stack, TextArea,
    TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::commands::StoredReview;
use crate::config::{
    AskFrom, ChannelMode, ModuleConfig, ReviewPlatform, PUBLIC_REVIEWS_MAX, PUBLIC_REVIEWS_MIN,
};
use crate::email_text::clip_chars;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Star
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    // La plateforme, lue sur le domaine du lien enregistré : en lecture seule, sous le champ.
    let hint: Component = match config.platform() {
        Some(platform) => Stack::new()
            .direction(StackDirection::Horizontal)
            .children(vec![
                Badge::new().label(platform_label(platform)).into(),
                FieldHint::new().text("i18n:host.reviewUrl.detected").into(),
            ])
            .into(),
        None => FieldHint::new().text("i18n:host.reviewUrl.help").into(),
    };
    let mut url = Field::new()
        .name("review_url")
        .label("i18n:host.reviewUrl.label")
        .required(false)
        .child(Stack::new().children(vec![
            TextInput::new()
                .name("review_url")
                // L'ancien lien Airbnb s'affiche ici tant que `review_url` est vide : il est
                // enregistré sous ce nom au prochain enregistrement.
                .value(config.public_url().unwrap_or_default())
                .placeholder("i18n:host.reviewUrl.placeholder")
                .into(),
            hint,
        ]));
    if let Some(error) = check::https_url(config.review_url.trim()) {
        url = url.error(error.get(&ctx.locale).to_string());
    }
    let mut children: Vec<Component> = vec![
        Field::new()
            .name("ask_from")
            .label("i18n:host.askFrom.label")
            .child(
                ChoiceList::new()
                    .name("ask_from")
                    .layout(ChoiceListLayout::Segmented)
                    .value(config.ask_from.as_wire())
                    .choices(vec![
                        ChoiceOption::new(
                            AskFrom::Checkout.as_wire(),
                            "i18n:host.askFrom.label.checkout",
                        ),
                        ChoiceOption::new(
                            AskFrom::NextDay10h.as_wire(),
                            "i18n:host.askFrom.label.next_day_10h",
                        ),
                    ]),
            )
            .into(),
        url.into(),
        ToggleRow::new()
            .name("channel_mode")
            .label("i18n:host.channelMode.label")
            .description("i18n:host.channelMode.help")
            .checked(config.channel_mode == ChannelMode::Auto)
            .into(),
        // Spec §3 : le lien ne dépend jamais de la note.
        InlineNotice::new()
            .tone(Tone::Neutral)
            .message("i18n:host.reviewUrl.anyRating")
            .into(),
    ];
    children.push(
        ToggleRow::new()
            .name("private_comment")
            .label("i18n:host.privateComment.label")
            .description("i18n:host.privateComment.help")
            .checked(config.private_comment)
            .into(),
    );
    children.push(
        Field::new()
            .name("thank_you_message")
            .label("i18n:host.thanks.label")
            .required(false)
            .child(
                TextArea::new()
                    .name("thank_you_message")
                    .value(config.thank_you_message.host_value(&ctx).to_string())
                    .placeholder("i18n:host.thanks.placeholder"),
            )
            .into(),
    );

    let card = Card::new()
        .title("i18n:host.section.reviews")
        .children(children);
    let consented = crate::commands::consented_reviews()?;
    let public = public_card(&config, &consented, &ctx.locale);
    Ok(Surface::new(Page::new().child(Form::new().child(card).child(public))).with_id(MAIN))
}

/// Longest excerpt of a review in the picker, in chars.
const EXCERPT_MAX_CHARS: usize = 80;

/// Carte « Page publique » : l'interrupteur, puis le choix de 2 à 6 avis parmi les consentis
/// (prénom, date, extrait). Le champ reste rendu sans avis à choisir : le formulaire envoie
/// toujours les clés déclarées.
fn public_card(config: &ModuleConfig, consented: &[(String, StoredReview)], locale: &str) -> Card {
    let candidates: Vec<&(String, StoredReview)> = consented
        .iter()
        .filter(|(_, review)| !review.comment.trim().is_empty())
        .collect();
    let choices = candidates
        .iter()
        .map(|(id, review)| {
            let date = review
                .at
                .map(|at| portaki_sdk::host::time::long_date(at.date_naive(), locale));
            let label = [crate::guest::first_name(review).map(str::to_string), date]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · ");
            ChoiceOption::new(id.clone(), label).description(format!(
                "★ {} · {}",
                review.rating,
                clip_chars(review.comment.trim(), EXCERPT_MAX_CHARS).text
            ))
        })
        .collect();
    let hint = if candidates.is_empty() {
        "i18n:host.public.reviews.empty"
    } else {
        "i18n:host.public.reviews.help"
    };
    let mut reviews = Field::new()
        .name("public_reviews")
        .label("i18n:host.public.reviews.label")
        .required(false)
        .child(Stack::new().children(vec![
            FieldHint::new().text(hint).into(),
            ChoiceList::new()
                .name("public_reviews")
                .multi(true)
                .limit(PUBLIC_REVIEWS_MAX as u32)
                .choices(choices)
                .value(serde_json::to_string(&config.public_reviews).unwrap_or_default())
                .into(),
        ]));
    if let Some(key) = public_reviews_problem(config, consented) {
        reviews = reviews.error(bundle_text!(key).get(locale).to_string());
    }
    Card::new()
        .title("i18n:host.section.public")
        .child(
            ToggleRow::new()
                .name("public_enabled")
                .label("i18n:host.public.enabled.label")
                .description("i18n:host.public.enabled.help")
                .checked(config.public_enabled),
        )
        .child(reviews)
}

/// Ce qui empêche le bloc public de s'afficher, en clé de message : moins de deux avis choisis
/// encore consentis, ou plus de six choisis. Rien tant que la page publique est désactivée.
pub(crate) fn public_reviews_problem(
    config: &ModuleConfig,
    consented: &[(String, StoredReview)],
) -> Option<&'static str> {
    if !config.public_enabled {
        return None;
    }
    if config.public_reviews.len() > PUBLIC_REVIEWS_MAX {
        return Some("host.public.reviews.tooMany");
    }
    (crate::guest::chosen(config, consented).len() < PUBLIC_REVIEWS_MIN)
        .then_some("host.public.reviews.tooFew")
}

fn platform_label(platform: ReviewPlatform) -> &'static str {
    match platform {
        ReviewPlatform::Airbnb => "Airbnb",
        ReviewPlatform::Booking => "Booking.com",
        ReviewPlatform::Google => "Google",
        ReviewPlatform::Abritel => "Abritel / Vrbo",
        ReviewPlatform::Tripadvisor => "Tripadvisor",
        ReviewPlatform::Other => "i18n:host.platform.other",
    }
}
