//! Host settings drawer (spec Votre avis §2.1): when to ask, the public review link (any
//! platform, detected from the domain), the private comment. The thank-you message stays.

use portaki_sdk::config::check;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, Page, Stack, Text, TextArea, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{AskFrom, ModuleConfig, ReviewPlatform};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Star
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    let mut url = Field::new()
        .name("review_url")
        .label("i18n:host.reviewUrl.label")
        .required(false)
        .child(Stack::new().children(vec![
                FieldHint::new().text("i18n:host.reviewUrl.help").into(),
                TextInput::new()
                    .name("review_url")
                    // L'ancien lien Airbnb s'affiche ici tant que `review_url` est vide : il est
                    // enregistré sous ce nom au prochain enregistrement.
                    .value(config.public_url().unwrap_or_default())
                    .placeholder("i18n:host.reviewUrl.placeholder")
                    .into(),
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
                    .value(config.ask_from.as_wire())
                    .choices(vec![
                        ChoiceOption::new(
                            AskFrom::Checkout.as_wire(),
                            "i18n:host.askFrom.label.checkout",
                        )
                        .icon(IconName::ClockCircle),
                        ChoiceOption::new(
                            AskFrom::NextDay10h.as_wire(),
                            "i18n:host.askFrom.label.next_day_10h",
                        )
                        .icon(IconName::ClockCircle),
                    ]),
            )
            .into(),
        url.into(),
    ];
    // Lecture seule, déduite du lien : sans `name`, le formulaire ne l'envoie pas.
    if let Some(platform) = config.platform() {
        children.push(
            Field::new()
                .label("i18n:host.platform.label")
                .child(
                    Text::new()
                        .text(platform_label(platform))
                        .variant(TextVariant::Body),
                )
                .into(),
        );
    }
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
    Ok(Surface::new(Page::new().child(Form::new().child(card))).with_id(MAIN))
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
