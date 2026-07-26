//! Guest home card — inline post-stay review (no overlay).
//!
//! Only selected *and* feasible platforms are offered (Airbnb needs a URL).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, QRCode, Select, Stack, Text, TextArea,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    let mut children = Vec::new();

    let prompt = t!("guest.prompt", property = data.property_name.as_str())
        .unwrap_or_else(|_| data.property_name.clone());
    children.push(Component::Text(
        Text::new().text(prompt).variant(TextVariant::Title),
    ));

    if !data.thank_you.is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(data.thank_you.clone())
                .variant(TextVariant::Body),
        ));
    } else {
        children.push(Component::Text(
            Text::new()
                .text("i18n:guest.defaultThanks")
                .variant(TextVariant::Body),
        ));
    }

    if data.show_airbnb {
        let url = data.airbnb_url.clone().unwrap_or_default();
        let action = Action::external(url.clone());
        children.push(Component::Button(
            Button::new().label("i18n:guest.airbnbCta").action(action),
        ));
        if data.show_qr && !url.is_empty() {
            children.push(Component::QRCode(QRCode::new().value(url).size(144.0)));
            children.push(Component::Text(
                Text::new()
                    .text("i18n:guest.scanQr")
                    .variant(TextVariant::Caption),
            ));
        }
    }

    if data.show_portaki {
        if data.show_airbnb {
            children.push(Component::Text(
                Text::new()
                    .text("i18n:guest.orPortaki")
                    .variant(TextVariant::Caption),
            ));
        }

        let submit_action = crate::ids::module_id().command(
            crate::ids::SUBMIT_REVIEW,
            crate::commands::SubmitReviewArgs {
                rating: 5,
                comment: String::new(),
            },
        );

        children.push(Component::Form(
            Form::new()
                .child(
                    Field::new()
                        .name("rating")
                        .label("i18n:guest.rating")
                        .child(
                            Select::new()
                                .name("rating")
                                .options(vec![
                                    ChoiceOption::new("1", "★"),
                                    ChoiceOption::new("2", "★★"),
                                    ChoiceOption::new("3", "★★★"),
                                    ChoiceOption::new("4", "★★★★"),
                                    ChoiceOption::new("5", "★★★★★"),
                                ])
                                .value("5"),
                        ),
                )
                .child(
                    Field::new()
                        .name("comment")
                        .label("i18n:guest.comment")
                        .child(
                            TextArea::new()
                                .name("comment")
                                .placeholder("i18n:guest.commentPlaceholder"),
                        ),
                )
                .child(
                    Button::new()
                        .label("i18n:guest.submit")
                        .action(submit_action),
                ),
        ));
    }

    Surface::new(
        Card::new()
            .icon("star")
            .title("i18n:home.card.title")
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
}
