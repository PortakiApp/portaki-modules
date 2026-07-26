//! Host dashboard surface — design `editorReviews` / `reviews-editor-v1`.
//!
//! Multi-select platform toggles; Airbnb link + QR only when Airbnb is selected.
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, Grid, InfoBanner, Page, Stack, TextArea, TextInput, Toggle, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, normalize_url, Localized};

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let thank_you_message = config.thank_you_message.get(&lang).to_string();

    let platform_airbnb = ctx.input_bool("platform_airbnb", config.platform_airbnb);
    let platform_portaki = ctx.input_bool("platform_portaki", config.platform_portaki);
    let draft_url = ctx
        .input_str("airbnb_review_url")
        .map(str::to_string)
        .unwrap_or_else(|| config.airbnb_review_url.clone());
    let airbnb_needs_url = platform_airbnb && normalize_url(&draft_url).is_none();

    let mut form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.channel")
            .subtitle("i18n:host.section.channel.help")
            .icon("star")
            .children(vec![Grid::new()
                .columns(2)
                .gap(8.0)
                .minColumnWidth(280.0)
                .children(vec![
                    platform_toggle(
                        "platform_airbnb",
                        "i18n:host.channel.airbnb",
                        "star",
                        platform_airbnb,
                    ),
                    platform_toggle(
                        "platform_portaki",
                        "i18n:host.channel.portaki",
                        "sparkles",
                        platform_portaki,
                    ),
                ])
                .into()])
            .into(),
    ];

    if !platform_airbnb && !platform_portaki {
        form_children.push(InfoBanner::new().message("i18n:host.platforms.none").into());
    }

    if platform_airbnb {
        let mut airbnb_children: Vec<Component> = Vec::new();
        if airbnb_needs_url {
            airbnb_children.push(
                InfoBanner::new()
                    .message("i18n:host.airbnb.urlRequired")
                    .into(),
            );
        }
        airbnb_children.push(
            Field::new()
                .name("airbnb_review_url")
                .label("i18n:host.airbnb.label")
                .child(
                    TextInput::new()
                        .name("airbnb_review_url")
                        .value(draft_url)
                        .placeholder("i18n:host.airbnb.placeholder"),
                )
                .into(),
        );
        airbnb_children.push(
            Field::new()
                .name("show_qr_code")
                .label("i18n:host.qr.label")
                .child(
                    Toggle::new()
                        .name("show_qr_code")
                        .checked(ctx.input_bool("show_qr_code", config.show_qr_code)),
                )
                .into(),
        );

        form_children.push(
            Card::new()
                .title("i18n:host.section.airbnb")
                .subtitle("i18n:host.section.airbnb.help")
                .icon("link")
                .children(airbnb_children)
                .into(),
        );
    }

    form_children.push(
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
    );

    // No Page title / Save — workspace tab owns chrome + footer Save.
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn platform_toggle(name: &str, label: &str, icon: &str, checked: bool) -> Component {
    ToggleRow::new()
        .name(name)
        .label(label)
        .icon(icon)
        .checked(checked)
        .into()
}
