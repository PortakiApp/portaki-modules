//! Host dashboard surface — design `editorReviews` / `reviews-editor-v1`.
//!
//! Choice cards for channel, Airbnb link + QR toggle, thank-you message.
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, Form, Page, Stack, TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, Localized};

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let thank_you_message = config.thank_you_message.get(&lang).to_string();

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.channel")
            .subtitle("i18n:host.section.channel.help")
            .icon("star")
            .children(vec![ChoiceList::new()
                .name("review_channel")
                .value(config.review_channel.as_str())
                .choices(vec![
                    ChoiceOption::new("airbnb", "i18n:host.channel.airbnb")
                        .description("i18n:host.channel.airbnb.desc")
                        .icon("star"),
                    ChoiceOption::new("portaki", "i18n:host.channel.portaki")
                        .description("i18n:host.channel.portaki.desc")
                        .icon("sparkles"),
                    ChoiceOption::new("both", "i18n:host.channel.both")
                        .description("i18n:host.channel.both.desc")
                        .icon("check-circle"),
                ])
                .into()])
            .into(),
        Card::new()
            .title("i18n:host.section.airbnb")
            .subtitle("i18n:host.section.airbnb.help")
            .icon("link")
            .children(vec![
                Field::new()
                    .name("airbnb_review_url")
                    .label("i18n:host.airbnb.label")
                    .child(
                        TextInput::new()
                            .name("airbnb_review_url")
                            .value(config.airbnb_review_url.clone())
                            .placeholder("i18n:host.airbnb.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("show_qr_code")
                    .label("i18n:host.qr.label")
                    .child(
                        Toggle::new()
                            .name("show_qr_code")
                            .checked(config.show_qr_code),
                    )
                    .into(),
            ])
            .into(),
        Card::new()
            .title("i18n:host.section.thanks")
            .subtitle("i18n:host.section.thanks.help")
            .icon("message")
            .children(vec![Field::new()
                .name("thank_you_message")
                .label("i18n:host.thanks.label")
                .child(
                    TextArea::new()
                        .name("thank_you_message")
                        .value(thank_you_message)
                        .placeholder("i18n:host.thanks.placeholder"),
                )
                .into()])
            .into(),
    ];

    // No Page title / Save — workspace tab owns chrome + footer Save.
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN)
}
